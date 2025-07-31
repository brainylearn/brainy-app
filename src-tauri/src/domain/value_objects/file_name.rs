use thiserror::Error;

#[derive(PartialEq, Eq)]
pub struct FileName(String);

#[derive(Error, Debug)]
enum Error {
    #[error("Name cannot be empty!")]
    EmptyName,
}

impl FileName {
    pub fn new(name: String) -> Result<FileName, Error> {
        let name = name.trim_matches('/').trim().to_string();
        if name.trim().is_empty() {
            return Err(Error::EmptyName);
        }
        Ok(FileName(name))
    }

    pub fn val(&self) -> &String {
        &self.0
    }
}
