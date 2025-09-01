use serde::{Deserialize, Serialize};

use crate::Guid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CellType {
    FlashCard,
    Note,
    Cloze,
    TrueFalse,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cell {
    id: Guid,
    file_id: Guid,
    content: String,
    cell_type: CellType,
    index: u32,
}

impl Cell {
    pub(in crate::cells) fn new(
        id: Guid,
        file_id: Guid,
        content: String,
        cell_type: CellType,
        index: u32,
    ) -> Self {
        Self {
            id,
            file_id,
            content,
            cell_type,
            index,
        }
    }
}
