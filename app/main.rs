use {
    dotenvy::dotenv,
    eyre::Result,
    lool::logger::{info, ConsoleLogger, Level}, tokio::{join, select},
};

// TODO: here we will trigger the start of both the grpc server and the websocket gateway
//       look at: https://github.com/hyperium/tonic/discussions/740

#[tokio::main]
async fn main() -> Result<()> {
    ConsoleLogger::default_setup(Level::Trace, "rustler")?;

    dotenv()?;
    let conn = entities::db::get_connection().await?;
    let mut rustler = rustlers::svc::RustlersSvc::new(conn.clone()).await;

    let (grpc_res, rustlers_res) = join! {
        grpc::server::start(conn.clone()),
        rustler.start(),        
    };

    info!("Shutting down");
    Ok(())
}
