use {dotenvy::dotenv, eyre::Result, log::info};

// TODO: here we will trigger the start of both the grpc server and the websocket gateway
//       look at: https://github.com/hyperium/tonic/discussions/740

#[tokio::main]
async fn main() -> Result<()> {
    dotenv()?;
    let conn = entities::db::get_connection().await?;

    info!("Starting gRPC server...");

    grpc::server::start(conn.clone()).await?;

    Ok(())
}
