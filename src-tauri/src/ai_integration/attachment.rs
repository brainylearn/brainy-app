use rig::Embed;
use serde::{Deserialize, Serialize};

#[derive(Embed, Clone, Debug, Serialize, Deserialize)]
pub struct Attachment {
    // TODO:
    // chat_id: String,
    // TODO:
    // name: String
    #[embed]
    pub content: String,
}
