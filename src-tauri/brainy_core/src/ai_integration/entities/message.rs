use rig::{
    OneOrMany,
    agent::Text,
    message::{AssistantContent, UserContent},
};

use crate::Guid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    chat_id: Guid,
    message_index: i64,
    role: MessageRole,
    content: Option<String>,
}

impl Message {
    pub(in crate::ai_integration) fn new(
        chat_id: Guid,
        message_index: i64,
        role: MessageRole,
        content: Option<String>,
    ) -> Self {
        Self {
            chat_id,
            message_index,
            role,
            content,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageRole {
    Human,
    Assistant,
}

impl From<Message> for rig::message::Message {
    fn from(value: Message) -> Self {
        match value.role {
            MessageRole::Human => rig::message::Message::User {
                content: OneOrMany::one(UserContent::text(value.content.unwrap_or_default())),
            },
            MessageRole::Assistant => rig::message::Message::Assistant {
                id: None,
                content: OneOrMany::one(AssistantContent::Text(Text {
                    text: value.content.unwrap_or_default(),
                })),
            },
        }
    }
}
