pub mod market;
pub mod ticker;

pub use sea_orm;

pub mod db {
    use {
        eyre::Result,
        lool::{cli::stylize::Stylize, logger::info, s},
        sea_orm::{
            ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbBackend, Statement,
        },
        std::sync::Arc,
    };

    const RUSTLER_DATABASE: &str = "RUSTLER_DATABASE";

    fn get_default_conn_str() -> String {
        let conn_str = s!("sqlite://rustler.db?mode=rwc");
        info!(
            "No `{}` env var found, using default: {}",
            RUSTLER_DATABASE.italic(),
            conn_str.green()
        );
        conn_str
    }

    pub async fn get_connection() -> Result<Arc<DatabaseConnection>> {
        let db_conn_str =
            std::env::var(RUSTLER_DATABASE).unwrap_or_else(|_| get_default_conn_str());

        let mut conn_opts = ConnectOptions::new(db_conn_str.to_owned());
        conn_opts.sqlx_logging(false);

        let conn = Arc::new(Database::connect(conn_opts).await?);

        conn.query_one(Statement::from_string(
            DbBackend::Sqlite,
            r#"
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            "#,
        ))
        .await?;

        info!("Database to {} connection established", db_conn_str.green());
        Ok(conn)
    }
}
