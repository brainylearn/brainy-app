use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::Guid;

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rating {
    #[default]
    Again,
    Hard,
    Good,
    Easy,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Review {
    pub(in crate::cells) id: Guid,
    /// Review can should exist even when the cell is deleted.
    pub(in crate::cells) cell_id: Option<Guid>,
    pub(in crate::cells) study_time: u32,
    pub(in crate::cells) date: DateTime<Utc>,
    pub(in crate::cells) rating: Rating,
}

impl Review {
    pub fn new(
        id: Option<Guid>,
        cell_id: Option<Guid>,
        study_time: u32,
        date: DateTime<Utc>,
        rating: Rating,
    ) -> Self {
        Self {
            id: id.unwrap_or(Guid::new_v4()),
            cell_id,
            study_time,
            date,
            rating,
        }
    }

    pub fn id(&self) -> Guid {
        self.id
    }

    pub fn cell_id(&self) -> Option<Guid> {
        self.cell_id
    }

    pub fn study_time(&self) -> u32 {
        self.study_time
    }

    pub fn date(&self) -> DateTime<Utc> {
        self.date
    }

    pub fn rating(&self) -> &Rating {
        &self.rating
    }
}

impl Default for Review {
    fn default() -> Self {
        Self {
            id: Guid::new_v4(),
            cell_id: Default::default(),
            study_time: Default::default(),
            date: Default::default(),
            rating: Default::default(),
        }
    }
}
