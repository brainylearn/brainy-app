use brainy_core::cells::entities::cell::Cell;
use serde::{Deserialize, Serialize};

use crate::entity::repetition;

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub cells: Vec<Cell>,
    pub repetitions: Vec<repetition::Model>,
}
