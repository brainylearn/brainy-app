use std::sync::Arc;

use crate::{
    Guid,
    ai_integration::{
        ai_service::{AiService, StreamLlmResponseEvent},
        ai_state::AiState,
        entities::{chat::Chat, message::Message},
        repositories::traits::ai_repository::AiRepository,
    },
    common::{
        api_error::ApiError, injector::injector::Injector,
        traits::repositories_context::RepositoriesContext, unit_of_work::UnitOfWork,
    },
};
use tauri::{State, ipc::Channel};
use tokio::sync::Mutex;

#[tauri::command]
pub async fn stream_ai_response(
    context: State<'_, Arc<Mutex<dyn RepositoriesContext>>>,
    ai_service: State<'_, Arc<AiService>>,
    on_event: Channel<StreamLlmResponseEvent>,
    prompt: String,
    chat_id: Option<Guid>,
) -> Result<(), ApiError> {
    let result = ai_service
        .stream(prompt, chat_id, |event| match on_event.send(event) {
            Ok(_) => Ok(()),
            Err(err) => Err(err.to_string()),
        })
        .await;
    let context = context.lock().await;
    context.save_changes().await?;

    match result {
        Ok(()) => Ok(()),
        Err(err) => Err(ApiError::new(err.to_string())),
    }
}

#[tauri::command]
pub async fn stop_ai_generation(ai_state: State<'_, Arc<AiState>>) -> Result<(), ApiError> {
    ai_state.cancel_generation();
    Ok(())
}

#[tauri::command]
pub async fn get_all_ai_chats_sorted_by_date_desc(
    injector: State<'_, Injector>,
) -> Result<Vec<Chat>, ApiError> {
    let scope = injector.start_scope();
    let chats = scope
        .resolve::<dyn AiRepository>()
        .await
        .get_all_chats_sorted_by_date_desc()
        .await?;
    Ok(chats)
}

#[tauri::command]
pub async fn delete_ai_chat(injector: State<'_, Injector>, id: Guid) -> Result<(), ApiError> {
    let scope = injector.start_scope();
    scope
        .resolve::<dyn AiRepository>()
        .await
        .delete_chat(id)
        .await?;
    scope.resolve::<UnitOfWork>().await.save_changes().await?;
    Ok(())
}

#[tauri::command]
pub async fn get_chat_messages_ordered(
    injector: State<'_, Injector>,
    id: Guid,
) -> Result<Vec<Message>, ApiError> {
    let scope = injector.start_scope();
    let messages = scope
        .resolve::<dyn AiRepository>()
        .await
        .get_chat_messages_ordered(id)
        .await?;
    Ok(messages)
}

#[tauri::command]
pub async fn rename_ai_chat(
    injector: State<'_, Injector>,
    id: Guid,
    new_title: String,
) -> Result<(), ApiError> {
    let scope = injector.start_scope();
    let ai_repository = scope.resolve::<dyn AiRepository>().await;
    let mut chat = ai_repository.get_chat_by_id(id).await?;
    chat.set_title(new_title);
    ai_repository.upsert_chat(&chat).await?;
    scope.resolve::<UnitOfWork>().await.save_changes().await?;
    Ok(())
}
