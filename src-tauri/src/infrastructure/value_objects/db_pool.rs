use std::ops::{Deref, DerefMut};

use sqlx::SqlitePool;
use tokio::sync::Mutex;

use crate::settings::value_objects::database_location::DatabaseLocation;

pub struct DbPool {
    pool: Mutex<SqlitePool>,
    location: DatabaseLocation,
}

impl DbPool {
    pub fn new(pool: SqlitePool, location: DatabaseLocation) -> Self {
        Self {
            pool: Mutex::new(pool),
            location,
        }
    }

    pub fn location(&self) -> &DatabaseLocation {
        &self.location
    }
}

impl Deref for DbPool {
    type Target = Mutex<SqlitePool>;

    fn deref(&self) -> &Self::Target {
        &self.pool
    }
}

impl DerefMut for DbPool {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.pool
    }
}
