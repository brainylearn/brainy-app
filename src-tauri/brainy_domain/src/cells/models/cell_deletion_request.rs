use crate::Guid;

// TODO: not needed longer
pub struct CellDeletionRequest(pub Guid);

impl CellDeletionRequest {
    pub fn new(uuid: Guid) -> Self {
        Self(uuid)
    }

    pub fn id(&self) -> Guid {
        self.0
    }
}
