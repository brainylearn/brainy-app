use uuid::Uuid;

use crate::domain::value_objects::file_name::FileName;

pub struct File {
    id: Uuid,
    pub name: FileName,
}

impl File {
    pub fn new(id: Option<Uuid>, name: FileName) -> File {
        File {
            id: id.unwrap_or(Uuid::new_v4()),
            name,
        }
    }
}

pub struct Cell {
    id: Uuid,
    file_id: i32,
    content: String,
    searchable_content: String,
    // cell_type: CellType,
}

pub struct Repetition;
