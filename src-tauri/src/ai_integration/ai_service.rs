use std::sync::Arc;

use injector_derive::ScopeInjectable;
#[cfg(not(test))]
use rig::client::{Nothing, ProviderClient};
#[cfg(not(test))]
use rig::providers::ollama;
use rig::tool::Tool;
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
use crate::ai_integration::tools::AcceptToolCallFromJson;
use crate::ai_integration::tools::create_flash_card::AcceptCreateFlashCard;
use crate::cells::cell_service::CellService;
use crate::cells::repositories::traits::cell_repository::CellRepository;
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
const DEFAULT_MAX_TURN: usize = 10;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum StreamLlmResponseEvent {
    CreatedChat(Chat),
    InProgress(String),
    ToolCalled(Message),
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
    #[error("Unknown tool name was given")]
    UnknownToolName,
    #[error("An unknown error has happened!")]
    UnknownError(String),
    #[error("Can only accept tool calls")]
    CanOnlyAcceptToolCalls,
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
    cell_repository: Arc<dyn CellRepository>,
    cell_service: Arc<CellService>,
    #[cfg(test)]
    mock_client: Arc<MockClient>,
}

pub type OnEventCallback = Arc<dyn Send + Sync + Fn(StreamLlmResponseEvent) -> Result<(), String>>;

impl AiService {
    pub async fn stream(
        &self,
        request: StreamAiRequest,
        on_event: OnEventCallback,
    ) -> Result<(), AiServiceError> {
        let _ = self.state.start_generation().await;

        let messages;
        let chat_id;
        let mut chat_to_upsert = None;
        if let Some(request_chat_id) = request.chat_id {
            chat_id = request_chat_id;
            messages = self
                .ai_repository
                .get_chat_messages_ordered(chat_id)
                .await?;
        } else {
            let chat = self.create_chat(&request.prompt).await?;
            chat_id = chat.id();
            messages = Vec::new();
            on_event(StreamLlmResponseEvent::CreatedChat(chat.clone()))?;
            chat_to_upsert = Some(chat);
        }

        let messages_to_upsert = Arc::new(Mutex::new(vec![Message::new(
            None,
            chat_id,
            MessageContent::Human(request.prompt.clone()),
        )]));

        let messages = messages.into_iter().map(|message| message.into()).collect();

        let agent = self
            .get_agent(
                &request,
                chat_id,
                messages_to_upsert.clone(),
                on_event.clone(),
            )
            .await?;
        let mut stream = agent
            .stream_chat(request.prompt, messages)
            .with_hook(StateCancellationHook::new(self.state.clone()))
            .await;

        let mut complete_ai_response = String::new();

        while let Some(content) = stream.next().await {
            match content {
                Ok(content) => {
                    if let MultiTurnStreamItem::StreamAssistantItem(
                        StreamedAssistantContent::Text(Text { text }),
                    ) = content
                    {
                        complete_ai_response = format!("{complete_ai_response}{text}");
                        on_event(StreamLlmResponseEvent::InProgress(text))?;
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
                    }
                    break;
                }
            };
        }

        if !complete_ai_response.trim().is_empty() {
            let mut messages_to_upsert = messages_to_upsert.lock().await;
            messages_to_upsert.push(Message::new(
                None,
                chat_id,
                MessageContent::Assistant(complete_ai_response),
            ));
        }

        // Delaying database operations to the end to avoid the writes from locking
        // the database.
        if let Some(chat) = chat_to_upsert {
            self.ai_repository.upsert_chat(&chat).await?;
        }

        for message in messages_to_upsert.lock().await.iter() {
            self.ai_repository.upsert_message(message).await?;
        }

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
        request: &StreamAiRequest,
        chat_id: Guid,
        messages_to_upsert: Arc<Mutex<Vec<Message>>>,
        on_event: OnEventCallback,
    ) -> Result<Agent<MultiCompletionModel>, AiServiceError> {
        let client = self.get_multi_completion_client().await?;
        let model_name = self.get_model_name().await;

        let builder = client
            .agent(&model_name)
            .temperature(DEFAULT_TEMPERATURE)
            .name("Brainy Tutor")
            .description(
                "An active learning tutor that explains concepts and converts \
                them into study materials using the best available format.",
            )
            .default_max_turns(DEFAULT_MAX_TURN)
            .preamble(
                "You are **Brainy's** tutor. Your job is to help users understand \
                and memorize information through active learning.\n\n\
                **Responsibilities:**\n\
                1. **Explain clearly:** Answer questions and break down concepts. \
                Prioritize understanding over memorization — don't let a user \
                try to memorize something they don't yet grasp.\n\
                2. **Detect study intent:** When a user wants to study, drill, \
                or memorize specific content, create study materials using your tools.\n\n\
                **When creating study materials:**\n\
                - Choose the most effective format for each fact based on the tools available.\n\
                - **One fact per item.** Each item tests a single, atomic piece of information.\n\
                - **Be concise.** Strip every redundant word without losing clarity.\n\
                - **Add context tags.** Prefix with a short domain tag to prevent ambiguity: \
                `[Biology]`, `[WW2]`, `[Calculus]`.\n\
                - **No enumeration.** Never ask users to list multiple items. \
                Break lists into individual items.\n\
                - **Disambiguate.** When concepts are easily confused, word the item \
                to highlight the distinguishing detail.\n\
                - After creating materials, briefly summarize to the user what was added to their deck.\n\n\
                **Rules:**\n\
                - Never create study materials without using your tools.\n\
                - Do not describe, list, or repeat card content in conversation — \
                only create them via tools. Once a tool call is made, the card exists \
                in the user's deck; there is no need to echo it back.",
        );

        if let Some(file_id) = request.file_id {
            // TODO: unit test
            Ok(builder
                .tool(CreateFlashCard::new(
                    file_id,
                    chat_id,
                    messages_to_upsert,
                    Some(on_event),
                ))
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

    pub async fn accept_tool_call(&self, message_id: Guid) -> Result<(), AiServiceError> {
        let mut message = self.ai_repository.get_message_by_id(message_id).await?;
        let tool_call = match message.content_mut() {
            MessageContent::ToolCall(tool_call) => tool_call,
            _ => return Err(AiServiceError::CanOnlyAcceptToolCalls),
        };

        #[allow(clippy::needless_late_init)]
        let tool: Box<dyn AcceptToolCallFromJson>;

        if tool_call.name == CreateFlashCard::NAME {
            tool = Box::new(AcceptCreateFlashCard::new(
                self.cell_repository.clone(),
                self.cell_service.clone(),
            ));
        } else {
            return Err(AiServiceError::UnknownToolName);
        }

        tool.accept_call(tool_call, tool_call.arguments.clone())
            .await?;

        // TODO: uncomment
        //tool_call.status = ToolCallStatus::Accepted;
        self.ai_repository.upsert_message(&message).await?;

        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

    use injector::{injector::Injector, register_scope};
    use rig::{
        OneOrMany,
        completion::{CompletionError, CompletionResponse, Usage},
        message::{AssistantContent, Message, UserContent},
        streaming::RawStreamingChoice,
    };

    use crate::{
        ai_integration::{
            clients::multi_completion_client::multi_response::MultiResponse,
            repositories::sqlite_ai_repository::SqliteAiRepository,
        },
        cells::repositories::{
            sqlite_cell_repository::SqliteCellRepository,
            sqlite_review_repository::SqliteReviewRepository,
            traits::review_repository::ReviewRepository,
        },
        test_utils::create_test_injector,
    };

    use super::*;

    async fn get_test_dependencies(mock_client: MockClient, state: Arc<AiState>) -> Injector {
        let mut injector = create_test_injector().await;

        let settings = Settings {
            enable_ai: true,
            ..Default::default()
        };

        injector.register_singleton(Arc::new(Mutex::new(settings)));
        injector.register_singleton(Arc::new(mock_client));
        injector.register_singleton(state);

        register_scope!(injector, dyn AiRepository, SqliteAiRepository);
        register_scope!(injector, dyn CellRepository, SqliteCellRepository);
        register_scope!(injector, dyn ReviewRepository, SqliteReviewRepository);
        register_scope!(injector, CellService);
        register_scope!(injector, AiService);

        injector
    }

    #[tokio::test]
    pub async fn stream_new_chat_created_new_chat_and_added_messages() {
        // Arrange

        let sent_stream_answer = AtomicBool::new(false);

        let mock_client = MockClient {
            model: None,
            completion_fn: Arc::new(Some(Box::new(|request| {
                if let Message::User { content } = request.chat_history.last()
                    && let UserContent::Text(text) = content.last()
                    && text.text() == "User message: User prompt"
                {
                    let tool_call = AssistantContent::tool_call(
                        "1",
                        "submit",
                        serde_json::to_value(GenerateTitle {
                            title: "Chat title".to_string(),
                        })
                        .unwrap(),
                    );
                    return CompletionResponse {
                        choice: OneOrMany::one(tool_call),
                        raw_response: MultiResponse::Mock,
                        usage: Usage::default(),
                        message_id: None,
                    };
                }

                panic!()
            }))),
            stream_fn: Arc::new(Some(Box::new(move |request| {
                if let Message::User { content } = request.chat_history.last()
                    && let UserContent::Text(text) = content.last()
                    && text.text() == "User prompt"
                    && !sent_stream_answer.load(Ordering::Relaxed)
                {
                    sent_stream_answer.store(true, Ordering::Relaxed);
                    return Ok(Some(RawStreamingChoice::Message("Bot answer".to_string())));
                }

                Ok(None)
            }))),
        };

        let injector = get_test_dependencies(mock_client, Arc::new(AiState::default())).await;
        let scope = injector.start_scope();
        let service = scope.resolve::<AiService>().await;
        let repository = scope.resolve::<dyn AiRepository>().await;

        let received_create_chat = Arc::new(AtomicBool::new(false));
        let received_in_progress = Arc::new(AtomicBool::new(false));

        // Clone before moving into closure
        let received_create_chat_clone = Arc::clone(&received_create_chat);
        let received_in_progress_clone = Arc::clone(&received_in_progress);

        let request = StreamAiRequest {
            prompt: "User prompt".to_string(),
            ..Default::default()
        };

        // Act

        service
            .stream(
                request,
                Arc::new(move |event| {
                    match event {
                        StreamLlmResponseEvent::CreatedChat(chat) => {
                            received_create_chat_clone
                                .store(chat.title() == "Chat title", Ordering::Relaxed);
                        }
                        StreamLlmResponseEvent::InProgress(message) => {
                            received_in_progress_clone
                                .store(message == "Bot answer", Ordering::Relaxed);
                        }
                        _ => (),
                    }
                    Ok(())
                }),
            )
            .await
            .unwrap();

        // Assert

        assert!(received_create_chat.load(Ordering::Relaxed));
        assert!(received_in_progress.load(Ordering::Relaxed));

        let chats = repository
            .get_all_chats_sorted_by_date_desc()
            .await
            .unwrap();
        assert_eq!(1, chats.len());
        assert_eq!("Chat title", chats[0].title());

        let messages = repository
            .get_chat_messages_ordered(chats[0].id())
            .await
            .unwrap();
        assert_eq!(2, messages.len());

        assert_eq!(
            MessageContent::Human("User prompt".to_string()),
            *messages[0].content()
        );

        assert_eq!(
            MessageContent::Assistant("Bot answer".to_string()),
            *messages[1].content()
        );
    }

    #[tokio::test]
    pub async fn stream_cancelled_response_stopped_generation() {
        // Arrange

        let last_sent_message = Arc::new(AtomicU32::new(1));
        let ai_state = Arc::new(AiState::default());
        let ai_state_clone = ai_state.clone();

        let mock_client = MockClient {
            model: None,
            completion_fn: Arc::new(Some(Box::new(|request| {
                if let Message::User { content } = request.chat_history.last()
                    && let UserContent::Text(text) = content.last()
                    && text.text() == "User message: User prompt"
                {
                    let tool_call = AssistantContent::tool_call(
                        "1",
                        "submit",
                        serde_json::to_value(GenerateTitle {
                            title: "Chat title".to_string(),
                        })
                        .unwrap(),
                    );
                    return CompletionResponse {
                        choice: OneOrMany::one(tool_call),
                        raw_response: MultiResponse::Mock,
                        usage: Usage::default(),
                        message_id: None,
                    };
                }

                panic!()
            }))),
            stream_fn: Arc::new(Some(Box::new(move |request| {
                if let Message::User { content } = request.chat_history.last()
                    && let UserContent::Text(text) = content.last()
                    && text.text() == "User prompt"
                {
                    let current = last_sent_message.load(Ordering::Relaxed);
                    if current > 3 {
                        ai_state_clone.cancel_generation();
                    }
                    last_sent_message.store(current + 1, Ordering::Relaxed);
                    return Ok(Some(RawStreamingChoice::Message(current.to_string())));
                }

                Ok(None)
            }))),
        };

        let injector = get_test_dependencies(mock_client, ai_state).await;
        let scope = injector.start_scope();
        let service = scope.resolve::<AiService>().await;
        let repository = scope.resolve::<dyn AiRepository>().await;

        let request = StreamAiRequest {
            prompt: "User prompt".to_string(),
            ..Default::default()
        };

        // Act

        service
            .stream(request, Arc::new(move |_| Ok(())))
            .await
            .unwrap();

        // Assert

        let chats = repository
            .get_all_chats_sorted_by_date_desc()
            .await
            .unwrap();
        let messages = repository
            .get_chat_messages_ordered(chats[0].id())
            .await
            .unwrap();
        assert_eq!(
            MessageContent::Assistant("123".to_string()),
            *messages[1].content()
        );
    }

    #[tokio::test]
    pub async fn stream_error_during_stream_called_correct_event_and_did_not_save_ai_message() {
        // Arrange

        let sent_stream_answer = AtomicBool::new(false);

        let mock_client = MockClient {
            model: None,
            completion_fn: Arc::new(Some(Box::new(|request| {
                if let Message::User { content } = request.chat_history.last()
                    && let UserContent::Text(text) = content.last()
                    && text.text() == "User message: User prompt"
                {
                    let tool_call = AssistantContent::tool_call(
                        "1",
                        "submit",
                        serde_json::to_value(GenerateTitle {
                            title: "Chat title".to_string(),
                        })
                        .unwrap(),
                    );
                    return CompletionResponse {
                        choice: OneOrMany::one(tool_call),
                        raw_response: MultiResponse::Mock,
                        usage: Usage::default(),
                        message_id: None,
                    };
                }

                panic!()
            }))),
            stream_fn: Arc::new(Some(Box::new(move |request| {
                if let Message::User { content } = request.chat_history.last()
                    && let UserContent::Text(text) = content.last()
                    && text.text() == "User prompt"
                {
                    if sent_stream_answer.load(Ordering::Relaxed) {
                        // Fail on second time.
                        return Err(CompletionError::ResponseError("error from AI".to_string()));
                    } else {
                        sent_stream_answer.store(true, Ordering::Relaxed);
                        return Ok(Some(RawStreamingChoice::Message("Bot answer".to_string())));
                    }
                }

                Ok(None)
            }))),
        };

        let injector = get_test_dependencies(mock_client, Arc::new(AiState::default())).await;
        let scope = injector.start_scope();
        let service = scope.resolve::<AiService>().await;
        let repository = scope.resolve::<dyn AiRepository>().await;

        let received_error = Arc::new(AtomicBool::new(false));
        let received_error_clone = received_error.clone();

        let request = StreamAiRequest {
            prompt: "User prompt".to_string(),
            ..Default::default()
        };

        // Act

        service
            .stream(
                request,
                Arc::new(move |event| {
                    if let StreamLlmResponseEvent::Error(error) = event {
                        received_error_clone.store(
                            error == "CompletionError: ResponseError: error from AI",
                            Ordering::Relaxed,
                        );
                    }
                    Ok(())
                }),
            )
            .await
            .unwrap();

        // Assert

        assert!(received_error.load(Ordering::Relaxed));

        let chats = repository
            .get_all_chats_sorted_by_date_desc()
            .await
            .unwrap();
        assert_eq!(1, chats.len());

        let messages = repository
            .get_chat_messages_ordered(chats[0].id())
            .await
            .unwrap();
        assert_eq!(2, messages.len());
    }
}
