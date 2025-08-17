use std::fmt::Display;

use thiserror::Error;

/// A common value ojbect used to represent the name of a folder or a file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileSystemItemName(String);

#[derive(Error, Debug)]
pub enum Error {
    #[error("Name cannot be empty!")]
    EmptyName,

    #[error("{0}")]
    InvalidName(&'static str),
}

impl FileSystemItemName {
    pub fn new(name: String) -> Result<FileSystemItemName, Error> {
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err(Error::EmptyName);
        } else if name.contains('/') {
            return Err(Error::InvalidName("The name cannot contain forward slash!"));
        }
        Ok(FileSystemItemName(name))
    }

    pub fn new_unchecked(name: String) -> FileSystemItemName {
        FileSystemItemName(name)
    }
}

impl Display for FileSystemItemName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
