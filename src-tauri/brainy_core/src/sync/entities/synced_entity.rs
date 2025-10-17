use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::Guid;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SyncedEntity {
    pub user_id: Guid,
    pub entity_id: Guid,
    pub created_date: DateTime<Utc>,
    pub last_sync_date: DateTime<Utc>,
    pub entity_type: EntityType,
    pub data: String,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum EntityType {
    Folder,
    File,
    Cell,
    Repetition,
    Review,
    DeletedEntity,
}
