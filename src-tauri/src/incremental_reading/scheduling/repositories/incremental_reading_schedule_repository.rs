use async_trait::async_trait;

use crate::{
    Guid, common::repository_error::RepositoryError,
    incremental_reading::scheduling::entities::incremental_reading_schedule::IncrementalReadingSchedule,
};

#[async_trait]
pub trait IncrementalReadingScheduleRepository: Send + Sync {
    async fn get_by_cell_id(
        &self,
        cell_id: Guid,
    ) -> Result<Option<IncrementalReadingSchedule>, RepositoryError>;
    async fn create(&self, schedule: &IncrementalReadingSchedule) -> Result<(), RepositoryError>;
    async fn update(&self, schedule: &IncrementalReadingSchedule) -> Result<(), RepositoryError>;
}
