use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::Guid;

#[derive(
    Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub enum State {
    #[default]
    New,
    Learning,
    Relearning,
    Review,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Repetition {
    pub(in crate::cells) id: Guid,
    pub(in crate::cells) file_id: Guid,
    pub(in crate::cells) cell_id: Guid,
    pub(in crate::cells) due: DateTime<Utc>,
    pub(in crate::cells) stability: f64,
    pub(in crate::cells) difficulty: f64,
    pub(in crate::cells) elapsed_days: i64,
    pub(in crate::cells) scheduled_days: i64,
    pub(in crate::cells) reps: i64,
    pub(in crate::cells) lapses: i64,
    pub(in crate::cells) state: State,
    pub(in crate::cells) last_review: DateTime<Utc>,
    pub(in crate::cells) additional_content: Option<String>,
}
