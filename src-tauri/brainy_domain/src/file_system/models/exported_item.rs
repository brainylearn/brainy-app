use crate::cells::entities::cell::{Cell, CellType};
use serde::{Deserialize, Serialize};

use crate::file_system::value_objects::file_system_item_name::FileSystemItemName;

// TODO: probably applicaiton layer and not domain
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportedItem {
    pub name: FileSystemItemName,
    pub item_type: ExportedItemType,
    pub cells: Option<Vec<ExportedCell>>,
    pub children: Option<Vec<ExportedItem>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportedItemType {
    File,
    Folder,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportedCell {
    pub content: String,
    pub cell_type: CellType,
}

impl From<Cell> for ExportedCell {
    fn from(value: Cell) -> Self {
        ExportedCell {
            cell_type: value.cell_type().clone(),
            content: value.content().to_string(),
        }
    }
}
