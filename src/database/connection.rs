use sqlx::{mysql::MySqlPoolOptions, MySqlPool};
use std::time::Duration;


pub async fn connection(db_url: &str) -> Result<MySqlPool, sqlx::Error> {
    MySqlPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(3))
        .connect(db_url)
        .await
}
