use std::{env, path::PathBuf, sync::Arc};

use brainy_infrastructure::common::{
    db_pool::DbPool, utils::create_sqlite_pool::create_sqlite_pool,
};
use injector::injector::Injector;
use tokio::fs;

use crate::{
    common::utils::create_injector::register_scoped_tx,
    infrastructure::value_objects::app_data_directory::AppDataDirectory,
};
use brainy_domain::{Guid, settings::value_objects::database_location::DatabaseLocation};

pub async fn create_temp_directory() -> PathBuf {
    let path = env::temp_dir().join(Guid::new_v4().to_string());
    fs::create_dir_all(path.clone()).await.unwrap();
    path
}

pub async fn create_test_injector() -> Injector {
    let mut injector = Injector::default();

    let app_data_directory = AppDataDirectory::new(create_temp_directory().await);
    injector.register_singleton(Arc::new(app_data_directory.clone()));

    let sqlite_pool = create_sqlite_pool("sqlite::memory:").await.unwrap();
    let database_location = DatabaseLocation::new_unchecked(app_data_directory.get_path().clone());

    let db_pool = DbPool::new(sqlite_pool, database_location);
    injector.register_singleton(Arc::new(db_pool));
    register_scoped_tx(&mut injector);

    injector
}
