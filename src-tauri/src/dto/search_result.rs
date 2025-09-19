use brainy_core::cells::entities::{cell::Cell, repetition::Repetition};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub cells: Vec<Cell>,
    pub repetitions: Vec<Repetition>,
}
