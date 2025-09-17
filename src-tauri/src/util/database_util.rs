// TODO: delete
use sea_orm::{Database, DatabaseConnection};

use crate::migration;

pub async fn load_database(path: &String) -> DatabaseConnection {
    let db_conn = Database::connect(format!("sqlite:///{path}?mode=rwc"))
        .await
        .expect("Cannot open the database");
    migration::setup_schema(&db_conn)
        .await
        .expect("Could not setup the database schema!");
    db_conn
}
