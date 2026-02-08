use serde::{Deserialize, Serialize};

use crate::Guid;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Chat {
    id: Guid,
    title: String,
}

impl Chat {
    pub fn new(id: Option<Guid>, title: String) -> Self {
        Self {
            id: id.unwrap_or(Guid::new_v4()),
            title,
        }
    }

    /// Used for unit testing, or repositories when reconstructing a chat.
    pub fn new_unchecked(id: Guid, title: String) -> Self {
        Self { id, title }
    }

    pub fn id(&self) -> Guid {
        self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }
}
