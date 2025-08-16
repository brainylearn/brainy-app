use thiserror::Error;
use uuid::Uuid;

use crate::file_system::path::Path;

#[derive(Debug, Clone)]
pub struct Folder {
    id: uuid::fmt::Hyphenated,
    path: Path,
}

// TODO: remove
#[derive(Error, Debug)]
pub enum FolderError {
    #[error("The file with the name '{name}' could not be found!")]
    FileNotFound { name: String },
    #[error("The file with the name '{name}' already exists!")]
    FileExists { name: String },

    #[error("The folder with the name '{name}' could not be found!")]
    FolderNotFound { name: String },
    #[error("The folder with the name '{name}' already exists!")]
    FolderExists { name: String },
}

impl Folder {
    pub fn new(id: Option<uuid::fmt::Hyphenated>, path: Path) -> Folder {
        Folder {
            id: id.unwrap_or(Uuid::new_v4().into()),
            path,
        }
    }

    pub fn id(&self) -> uuid::fmt::Hyphenated {
        self.id
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}
