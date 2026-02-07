use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{Sqlite, SqliteConnection, SqlitePool, Transaction};
use tokio::sync::Mutex;

use crate::{
    Guid,
    ai_integration::{
        entities::chat::Chat,
        repositories::{
            sqlite_ai_repository::file_row::{
                ASSISTANT_ROLE, ChatRow, HUMAN_ROLE, MessageRow, to_chat,
            },
            traits::ai_repository::AiRepository,
        },
        value_objects::message::Message,
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
    async fn upsert_chat(&self, chat: Chat) -> Result<(), RepositoryError> {
        let mut tx = self.tx.lock().await;
        let tx = tx.as_mut();

        let id = chat.id();
        let name = chat.name();

        let result = sqlx::query!(
            r#"INSERT INTO ai_chats(
                id,
                name)
            VALUES ($1, $2)
            ON CONFLICT(id) DO UPDATE
            SET id = $1,
                name = $2
            "#,
            id,
            name
        )
        .execute(&mut *tx)
        .await;

        upsert_messages(tx, &chat).await?;

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
                name
            FROM ai_chats
            WHERE id = $1"#,
            id
        )
        .fetch_one(&*self.pool)
        .await;

        if let Err(err) = chat_row {
            return Err(RepositoryError::UnknownError(err.to_string()));
        }

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

        if let Err(err) = message_rows {
            return Err(RepositoryError::UnknownError(err.to_string()));
        }

        let chat = to_chat(chat_row.unwrap(), message_rows.unwrap());
        Ok(chat)
    }
}

async fn upsert_messages(tx: &mut SqliteConnection, chat: &Chat) -> Result<(), RepositoryError> {
    for (index, message) in chat.messages().iter().enumerate() {
        let content;
        let role;
        let index = index as i64;

        match message {
            Message::Human {
                content: current_content,
            } => {
                content = current_content;
                role = HUMAN_ROLE;
            }
            Message::Assistant {
                content: current_content,
            } => {
                content = current_content;
                role = ASSISTANT_ROLE;
            }
        };

        let chat_id = chat.id();

        let result = sqlx::query!(
            r#"INSERT INTO ai_messages(
                ai_chat_id,
                message_index,
                role,
                content)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT(ai_chat_id, message_index) DO UPDATE SET
                ai_chat_id = $1,
                message_index = $2,
                role = $3,
                content = $4
            "#,
            chat_id,
            index,
            role,
            content
        )
        .execute(&mut *tx)
        .await;

        if let Err(err) = result {
            return Err(RepositoryError::UnknownError(err.to_string()));
        }
    }

    Ok(())
}

mod file_row {
    use crate::ai_integration::value_objects::message::Message;

    use super::*;

    pub(super) const HUMAN_ROLE: &str = "human";
    pub(super) const ASSISTANT_ROLE: &str = "human";

    pub(super) struct ChatRow {
        pub id: Guid,
        pub name: String,
    }

    pub(super) struct MessageRow {
        pub chat_id: String,
        pub message_index: i64,
        pub role: String,
        pub content: Option<String>,
    }

    pub(super) fn to_chat(chat_row: ChatRow, mut message_rows: Vec<MessageRow>) -> Chat {
        message_rows.sort_by_key(|message| message.message_index);

        let mapped_messages = message_rows
            .into_iter()
            .map(|message| {
                if message.role == HUMAN_ROLE {
                    Message::Human {
                        content: message.content.unwrap_or_default(),
                    }
                } else {
                    Message::Assistant {
                        content: message.content.unwrap_or_default(),
                    }
                }
            })
            .collect::<Vec<_>>();

        Chat::new_unchecked(chat_row.id, chat_row.name, mapped_messages)
    }
}
