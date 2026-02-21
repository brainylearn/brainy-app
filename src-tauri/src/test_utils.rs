use std::{env, path::PathBuf, sync::Arc};

use injector::{injector::Injector, register_scope};
use sqlx::SqlitePool;
use tokio::{fs, sync::Mutex};

use crate::{
    DbTransaction, Guid,
    common::{unit_of_work_ext::UnitOfWork, utils::create_sqlite_pool::create_sqlite_pool},
};

pub async fn create_temp_directory() -> PathBuf {
    let path = env::temp_dir().join(Guid::new_v4().to_string());
    fs::create_dir_all(path.clone()).await.unwrap();
    path
}

pub async fn create_test_injector() -> Injector {
    let mut injector = Injector::default();

    let pool = create_sqlite_pool("sqlite::memory:").await.unwrap();
    injector.register_singleton(Arc::new(pool));
    register_scope!(injector, UnitOfWork);

    // TODO: duplicated in lib.rs
    injector.register_scope_factory::<DbTransaction>(|scope| {
        Box::pin(async move {
            let pool = scope.resolve::<SqlitePool>().await;
            let tx = pool.begin().await.expect("Cannot create a new transaction");
            Arc::new(Mutex::new(tx))
        })
    });

    injector
}
