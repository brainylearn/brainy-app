use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::Guid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CellType {
    FlashCard,
    Note,
    Cloze,
    TrueFalse,
}

impl Display for CellType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).expect("Cannot serialize CellType")
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cell {
    id: Guid,
    file_id: Guid,
    content: String,
    cell_type: CellType,
    index: u32,
}

impl Cell {
    // TODO: repetitions, maybe an aggregate
    pub(in crate::cells) fn new(
        id: Option<Guid>,
        file_id: Guid,
        content: String,
        cell_type: CellType,
        index: u32,
    ) -> Self {
        Self {
            id: id.unwrap_or(Guid::new_v4()),
            file_id,
            content,
            cell_type,
            index,
        }
    }

    pub fn id(&self) -> Guid {
        self.id
    }

    pub fn file_id(&self) -> Guid {
        self.file_id
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn cell_type(&self) -> &CellType {
        &self.cell_type
    }

    pub fn index(&self) -> u32 {
        self.index
    }
}
