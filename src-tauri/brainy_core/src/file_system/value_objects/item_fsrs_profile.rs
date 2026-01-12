use serde::{Deserialize, Serialize};

use crate::Guid;

/// Represents which FSRS-profile is used in a given item, e.g. file or a folder.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ItemFsrsProfile {
    Inherit,
    Id(Guid),
}
