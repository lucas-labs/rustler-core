use {
    dotenvy::dotenv,
    eyre::Result,
    lool::logger::{info, ConsoleLogger, Level},
};

// TODO: here we will trigger the start of both the grpc server and the websocket gateway
//       look at: https://github.com/hyperium/tonic/discussions/740

#[tokio::main]
async fn main() -> Result<()> {
    ConsoleLogger::default_setup(Level::Trace, "rustler")?;

    dotenv()?;
    let conn = entities::db::get_connection().await?;

    grpc::server::start(conn.clone()).await?;

    info!("Shutting down");
    Ok(())
}
