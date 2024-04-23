pub mod migration;
pub use sea_orm;

mod orm {
    #[path = "market.rs"]
    pub mod market;
    #[path = "ticker.rs"]
    pub mod ticker;
}

mod services {
    #[path = "market.rs"]
    pub mod market;
    #[path = "ticker.rs"]
    pub mod ticker;
}

/// market entities and services
pub mod market {
    pub use super::{orm::market::*, services::market::*};
}

/// ticker entities and services
pub mod ticker {
    pub use super::{orm::ticker::*, services::ticker::*};
}

/// database connection stuff
pub mod db {
    use {
        eyre::Result,
        lool::{cli::stylize::Stylize, logger::info, s},
        sea_orm::{
            ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbBackend, Statement,
        },
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

    pub async fn get_connection() -> Result<DatabaseConnection> {
        let db_conn_str =
            std::env::var(RUSTLER_DATABASE).unwrap_or_else(|_| get_default_conn_str());

        let mut conn_opts = ConnectOptions::new(db_conn_str.to_owned());
        conn_opts.sqlx_logging(false);

        let conn = Database::connect(conn_opts).await?;

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
