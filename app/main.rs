use {
    dotenvy::dotenv,
    entities::sea_orm::{ConnectionTrait, Database, DbBackend, Statement},
    eyre::Result,
    std::sync::{Arc, Mutex},
};

// TODO: here we will trigger the start of both the grpc server and the websocket gateway
//       look at: https://github.com/hyperium/tonic/discussions/740

#[tokio::main]
async fn main() -> Result<()> {
    dotenv()?;

    let db_conn_str = std::env::var("DATABASE_URL")?;
    // let conn = Arc::new();
    let conn = Arc::new(Database::connect(&db_conn_str).await?);

    conn.query_one(Statement::from_string(
        DbBackend::Sqlite,
        r#"
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;
        "#,
    ))
    .await?;

    grpc::server::start(conn.clone()).await?;

    Ok(())
}
