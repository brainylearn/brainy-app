use std::{env, path::PathBuf, sync::Arc};

use injector::{injector::Injector, register_scope};
use sqlx::SqlitePool;
use tokio::{fs, sync::Mutex};

use crate::{
    DbTransaction, Guid,
    common::{sqlite_repositories_context::SqliteRepositoriesContext, unit_of_work::UnitOfWork},
};

pub async fn create_temp_directory() -> PathBuf {
    let path = env::temp_dir().join(Guid::new_v4().to_string());
    fs::create_dir_all(path.clone()).await.unwrap();
    path
}

pub async fn create_test_injector() -> Injector {
    let mut injector = Injector::default();

    let context = SqliteRepositoriesContext::create_testing_context().await;
    injector.register_singleton(context.pool.clone());
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
