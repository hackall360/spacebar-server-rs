use sqlx::{AnyPool, migrate::Migrator};

pub type DbPool = AnyPool;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

pub async fn init_database(database_url: &str) -> Result<DbPool, sqlx::Error> {
    let pool = AnyPool::connect(database_url).await?;
    MIGRATOR.run(&pool).await?;
    Ok(pool)
}

pub async fn close_database(pool: DbPool) {
    pool.close().await;
}

pub mod entities {
    use serde::{Deserialize, Serialize};
    use serde_json::Value;
    use sqlx::FromRow;

    #[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
    pub struct Migration {
        pub id: i64,
        pub timestamp: i64,
        pub name: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
    pub struct Config {
        pub key: String,
        pub value: Option<Value>,
    }
}
