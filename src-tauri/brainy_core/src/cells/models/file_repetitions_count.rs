use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
// TODO: maybe change name to study repetition
pub struct FileRepetitionCounts {
    pub new: u32,
    pub learning: u32,
    pub relearning: u32,
    pub review: u32,
}
