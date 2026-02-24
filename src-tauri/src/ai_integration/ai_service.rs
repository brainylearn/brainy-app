use std::sync::Arc;

use injector::injector_scope::InjectorScope;
use injector_derive::ScopeInjectable;
#[cfg(not(test))]
use rig::client::{Nothing, ProviderClient};
#[cfg(not(test))]
use rig::providers::ollama;
use rig::{
    agent::{Agent, MultiTurnStreamItem, StreamingError, Text},
    client::CompletionClient,
    completion::PromptError,
    streaming::{StreamedAssistantContent, StreamingChat},
};
use serde::Serialize;
use thiserror::Error;
use tokio::sync::Mutex;
use tokio_stream::StreamExt;

use crate::Guid;
#[cfg(test)]
use crate::ai_integration::clients::mock_client::MockClient;
use crate::ai_integration::stream_ai_request::StreamAiRequest;
use crate::{
    ai_integration::{
        ai_state::AiState,
        clients::multi_completion_client::{
            MultiCompletionClient, multi_completion_model::MultiCompletionModel,
        },
        entities::{
            chat::Chat,
            message::{Message, MessageContent},
        },
        json_schemas::generate_title::GenerateTitle,
        repositories::traits::ai_repository::AiRepository,
        state_cancellation_hook::StateCancellationHook,
        tools::create_flash_card::CreateFlashCard,
    },
    common::repository_error::RepositoryError,
    settings::Settings,
};

const DEFAULT_TEMPERATURE: f64 = 0.5;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum StreamLlmResponseEvent {
    CreatedChat(Chat),
    InProgress(String),
    ToolCalled,
    Finished,
    Error(String),
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum AiServiceError {
    #[error("{0}")]
    UnknownRepositoryError(#[from] RepositoryError),
    #[error("Ai is not enabled in settings!")]
    AiNotEnabled,
    #[error("Ollama model name is not filled in settings!")]
    #[cfg(not(test))]
    OllamaModelNameIsNotFilled,
    #[error("An unknown error has happened!")]
    UnknownError(String),
}

impl From<String> for AiServiceError {
    fn from(value: String) -> Self {
        AiServiceError::UnknownError(value)
    }
}

#[derive(ScopeInjectable)]
pub struct AiService {
    settings: Arc<Mutex<Settings>>,
    state: Arc<AiState>,
    ai_repository: Arc<dyn AiRepository>,
    #[cfg(test)]
    mock_client: Arc<MockClient>,
}

impl AiService {
    pub async fn stream<F>(
        &self,
        scope: &InjectorScope<'_>,
        request: StreamAiRequest,
        on_event: F,
    ) -> Result<(), AiServiceError>
    where
        F: Fn(StreamLlmResponseEvent) -> Result<(), String>,
    {
        let _ = self.state.start_generation().await;

        let messages;
        let current_chat_id;
        let mut chat_to_upsert = None;
        if let Some(chat_id) = request.chat_id {
            messages = self
                .ai_repository
                .get_chat_messages_ordered(chat_id)
                .await?;
            current_chat_id = chat_id;
        } else {
            chat_to_upsert = Some(self.create_chat(&request.prompt).await?);
            current_chat_id = chat_to_upsert.as_ref().unwrap().id();
            log::info!("Created new chat with id '{current_chat_id}'.");
            messages = Vec::new();
            on_event(StreamLlmResponseEvent::CreatedChat(
                chat_to_upsert.as_ref().unwrap().clone(),
            ))?;
        }

        let mut messages_to_upsert = Vec::new();
        messages_to_upsert.push(Message::new(
            None,
            current_chat_id,
            MessageContent::Human(request.prompt.clone()),
        ));

        let messages = messages
            .iter()
            .map(|message| message.clone().into())
            .collect();

        let agent = self.get_agent(scope, &request, current_chat_id).await?;
        let mut stream = agent
            .stream_chat(request.prompt, messages)
            .with_hook(StateCancellationHook::new(self.state.clone()))
            .await;

        let mut error_happened = false;
        let mut complete_ai_response = String::new();

        while let Some(content) = stream.next().await {
            #[cfg(debug_assertions)]
            log::info!("Received following answer from AI: {:?}", content);

            match content {
                Ok(content) => {
                    if let MultiTurnStreamItem::StreamAssistantItem(
                        StreamedAssistantContent::Text(Text { text }),
                    ) = content
                    {
                        complete_ai_response = format!("{complete_ai_response}{text}");
                        on_event(StreamLlmResponseEvent::InProgress(text))?;
                    } else if let MultiTurnStreamItem::StreamAssistantItem(
                        StreamedAssistantContent::ToolCall { .. },
                    ) = content
                    {
                        on_event(StreamLlmResponseEvent::ToolCalled)?;
                    }
                }
                Err(err) => {
                    let mut should_call_callback = true;

                    if let StreamingError::Prompt(ref prompt_error) = err
                        && matches!(**prompt_error, PromptError::PromptCancelled { .. })
                    {
                        should_call_callback = false;
                    }

                    if should_call_callback {
                        on_event(StreamLlmResponseEvent::Error(err.to_string()))?;
                        error_happened = true;
                    }
                    break;
                }
            };
        }

        // Only save AI message when an error does not happen.
        if !error_happened {
            log::info!("Error happened, not storing user message.");
            messages_to_upsert.push(Message::new(
                None,
                current_chat_id,
                MessageContent::Assistant(complete_ai_response),
            ));
        }

        // Delaying database operations to end to avoid locking anything else.
        if let Some(chat) = chat_to_upsert {
            self.ai_repository.upsert_chat(&chat).await?;
        }

        for message in messages_to_upsert {
            self.ai_repository.upsert_message(&message).await?;
        }

        on_event(StreamLlmResponseEvent::Finished)?;

        Ok(())
    }

    async fn create_chat(&self, prompt: &str) -> Result<Chat, AiServiceError> {
        let response = match self
            .get_multi_completion_client()
            .await?
            .extractor::<GenerateTitle>(self.get_model_name().await)
            .preamble(
                "You are a chat naming assistant. Your task is to \
                generate a concise, descriptive title for a conversation based \
                on the user's first message. Be specific and descriptive.",
            )
            .build()
            .extract(format!("User message: {}", prompt))
            .await
        {
            Ok(response) => response,
            Err(err) => return Err(AiServiceError::UnknownError(err.to_string())),
        };

        log::info!("Generated title for chat is '{}'.", response.title);
        Ok(Chat::new(None, response.title))
    }

    async fn get_agent(
        &self,
        scope: &InjectorScope<'_>,
        request: &StreamAiRequest,
        chat_id: Guid,
    ) -> Result<Agent<MultiCompletionModel>, AiServiceError> {
        let client = self.get_multi_completion_client().await?;
        let model_name = self.get_model_name().await;

        let builder = client
            .agent(&model_name)
            .temperature(DEFAULT_TEMPERATURE)
            .name("Main Agent")
            .description(
                "Acts as the user-facing tutor for explaining concepts and \
                managing the conversation.",
            )
            // TODO: some instruction are not important if creating tool agent is not important
            .preamble(
                "\
                You are the primary assistant for **Brainy**, an app designed \
                to help users master subjects through active learning and \
                flashcards. Your tone should be encouraging, concise, and \
                academically focused.
\
                **Your Responsibilities:**\n\
                1. **Tutor & Explain:** Answer user questions and explain \
                concepts clearly. Ensure the user actually understands a topic \
                before they try to memorize it.\n\
                2. **Identify Memorization Needs:** Pay attention to when a \
                user wants to study, remember, or drill specific information.\n\
                3. **Delegate:** When the user is ready to create study \
                materials, you must call the `Learning Content Agent` via your \
                available tools. Pass down the core text, facts, or topic the \
                user wants to learn.
\
                **Important Rule:** Do NOT generate the learning materials \
                yourself. You must always invoke the Learning Content Agent \
                to ensure the learning materials adhere to strict cognitive \
                science principles.",
            );

        if let Some(file_id) = request.file_id {
            // TODO: unit test
            Ok(builder
                .tool(
                    create_learning_content_agent(scope, &client, &model_name, file_id, chat_id)
                        .await,
                )
                .build())
        } else {
            Ok(builder.build())
        }
    }

    async fn get_multi_completion_client(&self) -> Result<MultiCompletionClient, AiServiceError> {
        let settings = self.settings.lock().await;
        if !settings.enable_ai {
            return Err(AiServiceError::AiNotEnabled);
        }

        #[cfg(test)]
        return Ok(MultiCompletionClient::Mock((*self.mock_client).clone()));

        #[cfg(not(test))]
        {
            if settings.ollama_model_name.is_none() {
                return Err(AiServiceError::OllamaModelNameIsNotFilled);
            }

            let client = MultiCompletionClient::Ollama(ollama::Client::from_val(Nothing));
            Ok(client)
        }
    }

    async fn get_model_name(&self) -> String {
        #[cfg(test)]
        return self.mock_client.model.clone().unwrap_or_default();

        #[cfg(not(test))]
        {
            let settings = self.settings.lock().await;
            let model_name = settings.ollama_model_name.as_ref().unwrap().clone();
            log::info!("Using the model with name '{model_name}'.");
            model_name
        }
    }
}

async fn create_learning_content_agent(
    scope: &InjectorScope<'_>,
    client: &MultiCompletionClient,
    model_name: &str,
    file_id: Guid,
    chat_id: Guid,
) -> Agent<MultiCompletionModel> {
    let builder = client
        .agent(model_name)
        .temperature(DEFAULT_TEMPERATURE)
        .name("Learning Content Agent")
        .description(
            "Transforms raw educational text or concepts into optimized, \
            active learning materials.",
        )
        .preamble(
            "\
                You are the **Learning Content Agent** for the Brainy app. You \
                receive raw text or concepts from the Main Agent and convert \
                them into optimized active learning tasks using your tools.
    \
                Always follow these principles:
                1. **Minimum Information:** Each item tests exactly *one* fact or idea.
                2. **Optimize Wording:** Strip redundant words. Keep questions \
                and answers as concise as possible.
                3. **No Enumerations:** Never ask users to list multiple items. \
                Break them into separate facts or cloze deletions.
                4. **Use Cloze Deletions:** Prefer fill-in-the-blank for \
                definitions and relationships (e.g., \"The capital of France is [...]\").
                5. **Provide Context:** Use short context cues to avoid \
                ambiguity (e.g., \"[Physics] Force equals mass times [...]\").
                6. **Target Interference:** Clearly distinguish similar \
                concepts to prevent confusion.

                **Output only structured learning materials via your tools.** \
                No conversational filler; output is passed back to the Main Agent.",
        )
        .tool(CreateFlashCard::new(file_id, chat_id, scope).await);

    builder.build()
}

// TOOD:
// #[cfg(test)]
// pub mod tests {
//     use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
//
//     use injector::{injector::Injector, register_scope};
//     use rig::{
//         OneOrMany,
//         completion::{CompletionError, CompletionResponse, Usage},
//         message::{AssistantContent, Message, UserContent},
//         streaming::RawStreamingChoice,
//     };
//
//     use crate::{
//         ai_integration::{
//             clients::multi_completion_client::multi_response::MultiResponse,
//             repositories::sqlite_ai_repository::SqliteAiRepository,
//         },
//         test_utils::create_test_injector,
//     };
//
//     use super::*;
//
//     async fn get_test_dependencies(mock_client: MockClient, state: Arc<AiState>) -> Injector {
//         let mut injector = create_test_injector().await;
//
//         let settings = Settings {
//             enable_ai: true,
//             ..Default::default()
//         };
//
//         injector.register_singleton(Arc::new(Mutex::new(settings)));
//         injector.register_singleton(Arc::new(mock_client));
//         injector.register_singleton(state);
//
//         register_scope!(injector, dyn AiRepository, SqliteAiRepository);
//         register_scope!(injector, AiService);
//
//         injector
//     }
//
//     #[tokio::test]
//     pub async fn stream_new_chat_created_new_chat_and_added_messages() {
//         // Arrange
//
//         let sent_stream_answer = AtomicBool::new(false);
//
//         let mock_client = MockClient {
//             model: None,
//             completion_fn: Arc::new(Some(Box::new(|request| {
//                 if let Message::User { content } = request.chat_history.last()
//                     && let UserContent::Text(text) = content.last()
//                     && text.text() == "User message: User prompt"
//                 {
//                     let tool_call = AssistantContent::tool_call(
//                         "1",
//                         "submit",
//                         serde_json::to_value(GenerateTitle {
//                             title: "Chat title".to_string(),
//                         })
//                         .unwrap(),
//                     );
//                     return CompletionResponse {
//                         choice: OneOrMany::one(tool_call),
//                         raw_response: MultiResponse::Mock,
//                         usage: Usage::default(),
//                         message_id: None,
//                     };
//                 }
//
//                 panic!()
//             }))),
//             stream_fn: Arc::new(Some(Box::new(move |request| {
//                 if let Message::User { content } = request.chat_history.last()
//                     && let UserContent::Text(text) = content.last()
//                     && text.text() == "User prompt"
//                     && !sent_stream_answer.load(Ordering::Relaxed)
//                 {
//                     sent_stream_answer.store(true, Ordering::Relaxed);
//                     return Ok(Some(RawStreamingChoice::Message("Bot answer".to_string())));
//                 }
//
//                 Ok(None)
//             }))),
//         };
//
//         let injector = get_test_dependencies(mock_client, Arc::new(AiState::default())).await;
//         let scope = injector.start_scope();
//         let service = scope.resolve::<AiService>().await;
//         let repository = scope.resolve::<dyn AiRepository>().await;
//
//         let received_create_chat = Arc::new(AtomicBool::new(false));
//         let received_in_progress = Arc::new(AtomicBool::new(false));
//         let received_finished = Arc::new(AtomicBool::new(false));
//
//         let request = StreamAiRequest {
//             prompt: "User prompt".to_string(),
//             ..Default::default()
//         };
//
//         // Act
//
//         service
//             .stream(request, |event| {
//                 match event {
//                     StreamLlmResponseEvent::CreatedChat(chat) => {
//                         received_create_chat
//                             .clone()
//                             .store(chat.title() == "Chat title", Ordering::Relaxed);
//                     }
//                     StreamLlmResponseEvent::InProgress(message) => {
//                         received_in_progress
//                             .clone()
//                             .store(message == "Bot answer", Ordering::Relaxed);
//                     }
//                     StreamLlmResponseEvent::Finished => {
//                         received_finished.clone().store(true, Ordering::Relaxed);
//                     }
//                     _ => (),
//                 }
//                 Ok(())
//             })
//             .await
//             .unwrap();
//
//         // Assert
//
//         assert!(received_create_chat.load(Ordering::Relaxed));
//         assert!(received_in_progress.load(Ordering::Relaxed));
//         assert!(received_finished.load(Ordering::Relaxed));
//
//         let chats = repository
//             .get_all_chats_sorted_by_date_desc()
//             .await
//             .unwrap();
//         assert_eq!(1, chats.len());
//         assert_eq!("Chat title", chats[0].title());
//
//         let messages = repository
//             .get_chat_messages_ordered(chats[0].id())
//             .await
//             .unwrap();
//         assert_eq!(2, messages.len());
//
//         assert_eq!(
//             MessageContent::Human("User prompt".to_string()),
//             *messages[0].content()
//         );
//
//         assert_eq!(
//             MessageContent::Assistant("Bot answer".to_string()),
//             *messages[1].content()
//         );
//     }
//
//     #[tokio::test]
//     pub async fn stream_cancelled_response_stopped_generation() {
//         // Arrange
//
//         let last_sent_message = Arc::new(AtomicU32::new(1));
//         let ai_state = Arc::new(AiState::default());
//         let ai_state_clone = ai_state.clone();
//
//         let mock_client = MockClient {
//             model: None,
//             completion_fn: Arc::new(Some(Box::new(|request| {
//                 if let Message::User { content } = request.chat_history.last()
//                     && let UserContent::Text(text) = content.last()
//                     && text.text() == "User message: User prompt"
//                 {
//                     let tool_call = AssistantContent::tool_call(
//                         "1",
//                         "submit",
//                         serde_json::to_value(GenerateTitle {
//                             title: "Chat title".to_string(),
//                         })
//                         .unwrap(),
//                     );
//                     return CompletionResponse {
//                         choice: OneOrMany::one(tool_call),
//                         raw_response: MultiResponse::Mock,
//                         usage: Usage::default(),
//                         message_id: None,
//                     };
//                 }
//
//                 panic!()
//             }))),
//             stream_fn: Arc::new(Some(Box::new(move |request| {
//                 if let Message::User { content } = request.chat_history.last()
//                     && let UserContent::Text(text) = content.last()
//                     && text.text() == "User prompt"
//                 {
//                     let current = last_sent_message.load(Ordering::Relaxed);
//                     if current > 3 {
//                         ai_state_clone.cancel_generation();
//                     }
//                     last_sent_message.store(current + 1, Ordering::Relaxed);
//                     return Ok(Some(RawStreamingChoice::Message(current.to_string())));
//                 }
//
//                 Ok(None)
//             }))),
//         };
//
//         let injector = get_test_dependencies(mock_client, ai_state).await;
//         let scope = injector.start_scope();
//         let service = scope.resolve::<AiService>().await;
//         let repository = scope.resolve::<dyn AiRepository>().await;
//
//         let received_finished = Arc::new(AtomicBool::new(false));
//
//         let request = StreamAiRequest {
//             prompt: "User prompt".to_string(),
//             ..Default::default()
//         };
//
//         // Act
//
//         service
//             .stream(request, |event| {
//                 if let StreamLlmResponseEvent::Finished = event {
//                     received_finished.clone().store(true, Ordering::Relaxed);
//                 }
//                 Ok(())
//             })
//             .await
//             .unwrap();
//
//         // Assert
//
//         assert!(received_finished.load(Ordering::Relaxed));
//
//         let chats = repository
//             .get_all_chats_sorted_by_date_desc()
//             .await
//             .unwrap();
//         let messages = repository
//             .get_chat_messages_ordered(chats[0].id())
//             .await
//             .unwrap();
//         assert_eq!(
//             MessageContent::Assistant("123".to_string()),
//             *messages[1].content()
//         );
//     }
//
//     #[tokio::test]
//     pub async fn stream_error_during_stream_called_correct_event_and_did_not_save_ai_message() {
//         // Arrange
//
//         let sent_stream_answer = AtomicBool::new(false);
//
//         let mock_client = MockClient {
//             model: None,
//             completion_fn: Arc::new(Some(Box::new(|request| {
//                 if let Message::User { content } = request.chat_history.last()
//                     && let UserContent::Text(text) = content.last()
//                     && text.text() == "User message: User prompt"
//                 {
//                     let tool_call = AssistantContent::tool_call(
//                         "1",
//                         "submit",
//                         serde_json::to_value(GenerateTitle {
//                             title: "Chat title".to_string(),
//                         })
//                         .unwrap(),
//                     );
//                     return CompletionResponse {
//                         choice: OneOrMany::one(tool_call),
//                         raw_response: MultiResponse::Mock,
//                         usage: Usage::default(),
//                         message_id: None,
//                     };
//                 }
//
//                 panic!()
//             }))),
//             stream_fn: Arc::new(Some(Box::new(move |request| {
//                 if let Message::User { content } = request.chat_history.last()
//                     && let UserContent::Text(text) = content.last()
//                     && text.text() == "User prompt"
//                 {
//                     if sent_stream_answer.load(Ordering::Relaxed) {
//                         // Fail on second time.
//                         return Err(CompletionError::ResponseError("error from AI".to_string()));
//                     } else {
//                         sent_stream_answer.store(true, Ordering::Relaxed);
//                         return Ok(Some(RawStreamingChoice::Message("Bot answer".to_string())));
//                     }
//                 }
//
//                 Ok(None)
//             }))),
//         };
//
//         let injector = get_test_dependencies(mock_client, Arc::new(AiState::default())).await;
//         let scope = injector.start_scope();
//         let service = scope.resolve::<AiService>().await;
//         let repository = scope.resolve::<dyn AiRepository>().await;
//
//         let received_error = Arc::new(AtomicBool::new(false));
//         let received_finished = Arc::new(AtomicBool::new(false));
//
//         let request = StreamAiRequest {
//             prompt: "User prompt".to_string(),
//             ..Default::default()
//         };
//
//         // Act
//
//         service
//             .stream(request, |event| {
//                 match event {
//                     StreamLlmResponseEvent::Error(error) => {
//                         received_error.clone().store(
//                             error == "CompletionError: ResponseError: error from AI",
//                             Ordering::Relaxed,
//                         );
//                     }
//                     StreamLlmResponseEvent::Finished => {
//                         received_finished.clone().store(true, Ordering::Relaxed);
//                     }
//                     _ => (),
//                 }
//                 Ok(())
//             })
//             .await
//             .unwrap();
//
//         // Assert
//
//         assert!(received_finished.load(Ordering::Relaxed));
//         assert!(received_error.load(Ordering::Relaxed));
//
//         let chats = repository
//             .get_all_chats_sorted_by_date_desc()
//             .await
//             .unwrap();
//         assert_eq!(1, chats.len());
//
//         let messages = repository
//             .get_chat_messages_ordered(chats[0].id())
//             .await
//             .unwrap();
//         assert_eq!(1, messages.len());
//     }
// }
