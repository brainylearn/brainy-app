use brainy_domain::{Guid, ai_integration::entities::chat::Chat};
use chrono::{DateTime, Utc};

pub struct ChatRow {
    pub id: Guid,
    pub title: String,
    pub created_date: DateTime<Utc>,
}

impl From<ChatRow> for Chat {
    fn from(value: ChatRow) -> Self {
        Chat::new_unchecked(value.id, value.created_date, value.title)
    }
}
