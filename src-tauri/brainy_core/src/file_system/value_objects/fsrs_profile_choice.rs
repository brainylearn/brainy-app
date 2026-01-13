use serde::{Deserialize, Serialize};

use crate::Guid;

/// Represents which FSRS-profile is chosen in a given item, e.g. file or a folder.
#[derive(Copy, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type", content = "content")]
pub enum FsrsProfileChoice {
    Inherit,
    Id(Guid),
}
