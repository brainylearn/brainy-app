use rig::{
    OneOrMany,
    agent::Text,
    message::{AssistantContent, UserContent},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    Human { content: String },
    Assistant { content: String },
}

impl From<Message> for rig::message::Message {
    fn from(value: Message) -> Self {
        match value {
            Message::Human { content } => rig::message::Message::User {
                content: OneOrMany::one(UserContent::text(content)),
            },
            Message::Assistant { content } => rig::message::Message::Assistant {
                id: None,
                content: OneOrMany::one(AssistantContent::Text(Text { text: content })),
            },
        }
    }
}
