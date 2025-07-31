use thiserror::Error;
use uuid::Uuid;

use crate::domain::value_objects::path::Path;

#[derive(Debug, Clone)]
pub struct Folder {
    id: Uuid,
    path: Path,
    subfolders: Vec<Folder>,
}

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
    pub fn new(id: Option<Uuid>, path: Path) -> Folder {
        Folder {
            id: id.unwrap_or(Uuid::new_v4()),
            path,
            subfolders: Vec::new(),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn add_subfolder(&mut self, folder: Folder) -> Result<(), FolderError> {
        if self.subfolders.iter().any(|f| f.path == folder.path) {
            return Err(FolderError::FolderExists { name: folder.path.name() });
        }
        self.subfolders.push(folder);

        Ok(())
    }
}
