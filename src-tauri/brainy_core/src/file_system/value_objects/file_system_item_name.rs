use std::fmt::Display;

use thiserror::Error;

/// A common value ojbect used to represent the name of a folder or a file.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileSystemItemName(String);

#[derive(Error, Debug, PartialEq, Eq)]
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

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn new_empty_name_returned_error() {
        // Act

        let actual = FileSystemItemName::new("  ".to_string());

        // Assert

        assert_eq!(Err(Error::EmptyName), actual);
    }

    #[test]
    fn new_containing_slash_in_name_returned_error() {
        // Act

        let actual = FileSystemItemName::new("file 1/file2".to_string());

        // Assert

        assert_eq!(
            Err(Error::InvalidName("The name cannot contain forward slash!")),
            actual
        );
    }

    #[test]
    fn new_valid_name_returned_result() {
        // Act

        let actual = FileSystemItemName::new("file 1".to_string());

        // Assert

        assert_eq!(
            Ok(FileSystemItemName::new_unchecked("file 1".to_string())),
            actual
        );
    }
}
