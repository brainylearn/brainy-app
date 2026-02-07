use crate::{Guid, ai_integration::value_objects::message::Message};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chat {
    id: Guid,
    name: String,
    messages: Vec<Message>,
}

impl Chat {
    pub fn new(id: Option<Guid>, name: String, messages: Vec<Message>) -> Self {
        Self {
            id: id.unwrap_or(Guid::new_v4()),
            name,
            messages,
        }
    }

    /// Used for unit testing, or repositories when reconstructing a chat.
    pub fn new_unchecked(id: Guid, name: String, messages: Vec<Message>) -> Self {
        Self { id, name, messages }
    }

    pub fn id(&self) -> Guid {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn messages(&self) -> &[Message] {
        &self.messages
    }

    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }
}
