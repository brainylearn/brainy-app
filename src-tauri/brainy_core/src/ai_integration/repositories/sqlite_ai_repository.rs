use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{Sqlite, SqlitePool, Transaction};
use tokio::sync::Mutex;

use crate::{
    Guid,
    ai_integration::{
        entities::{chat::Chat, message::Message},
        repositories::{
            sqlite_ai_repository::file_row::{ChatRow, MessageRow},
            traits::ai_repository::AiRepository,
        },
    },
    common::repository_error::RepositoryError,
};

pub struct SqliteAiRepository {
    pool: Arc<SqlitePool>,
    tx: Arc<Mutex<Transaction<'static, Sqlite>>>,
}

impl SqliteAiRepository {
    pub fn new(pool: Arc<SqlitePool>, tx: Arc<Mutex<Transaction<'static, Sqlite>>>) -> Self {
        Self { pool, tx }
    }
}

// TODO: unit test
#[async_trait]
impl AiRepository for SqliteAiRepository {
    async fn get_all_chats(&self) -> Result<Vec<Chat>, RepositoryError> {
        let chat_rows = sqlx::query_as!(
            ChatRow,
            r#"SELECT
                id as "id: _",
                title
            FROM ai_chats"#
        )
        .fetch_all(&*self.pool)
        .await;

        match chat_rows {
            Ok(chat_rows) => Ok(chat_rows.into_iter().map(|chat| chat.into()).collect()),
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
        }
    }

    async fn upsert_chat(&self, chat: Chat) -> Result<(), RepositoryError> {
        let mut tx = self.tx.lock().await;
        let tx = tx.as_mut();

        let id = chat.id();
        let name = chat.title();

        let result = sqlx::query!(
            r#"INSERT INTO ai_chats(
                id,
                title)
            VALUES ($1, $2)
            ON CONFLICT(id) DO UPDATE
            SET id = $1,
                title = $2
            "#,
            id,
            name
        )
        .execute(&mut *tx)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
        }
    }

    async fn get_by_id(&self, id: Guid) -> Result<Chat, RepositoryError> {
        let chat_row = sqlx::query_as!(
            ChatRow,
            r#"SELECT
                id as "id: _",
                title
            FROM ai_chats
            WHERE id = $1"#,
            id
        )
        .fetch_one(&*self.pool)
        .await;

        match chat_row {
            Ok(chat_row) => Ok(chat_row.into()),
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
        }
    }

    async fn get_chat_messages(&self, id: Guid) -> Result<Vec<Message>, RepositoryError> {
        let message_rows = sqlx::query_as!(
            MessageRow,
            r#"SELECT
                ai_chat_id as "chat_id: _",
                message_index,
                role,
                content
            FROM ai_messages
            WHERE ai_chat_id = $1"#,
            id
        )
        .fetch_all(&*self.pool)
        .await;

        match message_rows {
            Ok(message_rows) => Ok(message_rows
                .into_iter()
                .map(|message| message.into())
                .collect()),
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
        }
    }
}

// TODO:
// async fn upsert_messages(tx: &mut SqliteConnection, chat: &Chat) -> Result<(), RepositoryError> {
//     for (index, message) in chat.messages().iter().enumerate() {
//         let content;
//         let role;
//         let index = index as i64;
//
//         match message {
//             Message::Human {
//                 content: current_content,
//             } => {
//                 content = current_content;
//                 role = HUMAN_ROLE;
//             }
//             Message::Assistant {
//                 content: current_content,
//             } => {
//                 content = current_content;
//                 role = ASSISTANT_ROLE;
//             }
//         };
//
//         let chat_id = chat.id();
//
//         let result = sqlx::query!(
//             r#"INSERT INTO ai_messages(
//                 ai_chat_id,
//                 message_index,
//                 role,
//                 content)
//             VALUES ($1, $2, $3, $4)
//             ON CONFLICT(ai_chat_id, message_index) DO UPDATE SET
//                 ai_chat_id = $1,
//                 message_index = $2,
//                 role = $3,
//                 content = $4
//             "#,
//             chat_id,
//             index,
//             role,
//             content
//         )
//         .execute(&mut *tx)
//         .await;
//
//         if let Err(err) = result {
//             return Err(RepositoryError::UnknownError(err.to_string()));
//         }
//     }
//
//     Ok(())
// }

mod file_row {
    use crate::ai_integration::entities::message::MessageRole;

    use super::*;

    pub(super) const HUMAN_ROLE: &str = "human";
    pub(super) const ASSISTANT_ROLE: &str = "human";

    pub(super) struct ChatRow {
        pub id: Guid,
        pub title: String,
    }

    pub(super) struct MessageRow {
        pub chat_id: Guid,
        pub message_index: i64,
        pub role: String,
        pub content: Option<String>,
    }

    impl From<ChatRow> for Chat {
        fn from(value: ChatRow) -> Self {
            Chat::new_unchecked(value.id, value.title)
        }
    }

    impl From<MessageRow> for Message {
        fn from(value: MessageRow) -> Self {
            let role = if value.role == HUMAN_ROLE {
                MessageRole::Human
            } else {
                MessageRole::Assistant
            };

            Message::new(value.chat_id, value.message_index, role, value.content)
        }
    }
}
