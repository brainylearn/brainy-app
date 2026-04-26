use std::sync::Arc;

use injector::injector::Injector;
use tokio::sync::Mutex;

use crate::common::{db_pool::DbPool, db_transaction::DbTransaction};

pub fn register_scoped_tx(injector: &mut Injector) {
    injector.register_scope_factory::<DbTransaction>(|scope| {
        Box::pin(async move {
            let db_pool = scope.resolve::<DbPool>().await;
            let pool = db_pool.pool().await;
            let tx = pool.begin().await.expect("Cannot create a new transaction");
            let db_transaction = DbTransaction::new(Mutex::new(tx));
            Arc::new(db_transaction)
        })
    });
}
