pub mod market;
pub mod ticker;

pub use sea_orm;

pub mod db {
    use {
        eyre::Result,
        sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, Statement},
        std::sync::Arc,
    };

    pub async fn get_connection() -> Result<Arc<DatabaseConnection>> {
        let db_conn_str = std::env::var("DATABASE_URL")?;
        let conn = Arc::new(Database::connect(&db_conn_str).await?);

        conn.query_one(Statement::from_string(
            DbBackend::Sqlite,
            r#"
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            "#,
        ))
        .await?;

        Ok(conn)
    }
}
