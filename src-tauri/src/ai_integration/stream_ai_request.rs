use serde::{Deserialize, Serialize};

use brainy_domain::Guid;

#[derive(Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamAiRequest {
    pub prompt: String,
    pub chat_id: Option<Guid>,
    pub file_id: Option<Guid>,
}
