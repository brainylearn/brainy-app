use std::{env, path::PathBuf, sync::Arc};

use injector::injector::Injector;
use tokio::{fs, sync::Mutex};

use crate::{
    Guid,
    common::{DbPool, DbTransaction, utils::create_sqlite_pool::create_sqlite_pool},
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

    // TODO: duplicated in lib.rs
    injector.register_scope_factory::<Mutex<DbTransaction>>(|scope| {
        Box::pin(async move {
            let pool = scope.resolve::<DbPool>().await;
            let tx = pool.begin().await.expect("Cannot create a new transaction");
            Arc::new(Mutex::new(tx))
        })
    });

    injector
}
