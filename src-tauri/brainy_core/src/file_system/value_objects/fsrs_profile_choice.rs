use serde::{Deserialize, Serialize};

use crate::Guid;

/// Represents which FSRS-profile is chosen in a given item, e.g. file or a folder.
#[derive(Copy, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type", content = "content")]
pub enum FsrsProfileChoice {
    Inherit,
    Id(Guid),
}

pub trait ToOptionWithId {
    fn to_option(&self) -> Option<Guid>;
}

impl ToOptionWithId for &FsrsProfileChoice {
    fn to_option(&self) -> Option<Guid> {
        if let &FsrsProfileChoice::Id(id) = self {
            Some(*id)
        } else {
            None
        }
    }
}
