use std::fmt::Display;

use thiserror::Error;

/// Represents the path of a file.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Path(Vec<String>);

#[derive(Error, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("Root does not have a parent!")]
    RootDoesNotHaveParent,
}

impl Path {
    pub fn new(path: &str) -> Self {
        let segments = path.split('/').map(|segment| segment.trim().to_string());
        let non_empty_segments: Vec<_> = segments.filter(|segment| !segment.is_empty()).collect();
        Path(non_empty_segments)
    }

    pub fn parent_directory(&self) -> Result<Path, Error> {
        if self.0.is_empty() {
            return Err(Error::RootDoesNotHaveParent);
        }

        let parent_segments = self
            .0
            .clone()
            .into_iter()
            .take(self.0.len() - 1)
            .collect::<Vec<_>>();
        Ok(Self::new(&parent_segments.join("/")))
    }

    /// Return the name of the folder/file represented by the path.
    pub fn name(&self) -> String {
        self.0.last().unwrap().into()
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.join("/"))
    }
}
