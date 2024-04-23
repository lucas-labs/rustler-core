use {
    dotenvy::dotenv,
    eyre::Result,
    lool::logger::{info, ConsoleLogger, Level},
    rustlers::{rustlerjar, rustlers::binance::BinanceRustler, svc::RustlersSvc},
    tokio::join,
};

// TODO: here we will trigger the start of both the grpc server and the websocket gateway
//       look at: https://github.com/hyperium/tonic/discussions/740

#[tokio::main]
async fn main() -> Result<()> {
    ConsoleLogger::default_setup(Level::Trace, "rustler")?;

    dotenv()?;
    let conn = entities::db::get_connection().await?;
    let mut rustler = RustlersSvc::new(
        conn.clone(),
        rustlerjar! {
            "BINANCE" => BinanceRustler
        },
    )
    .await;

    let (_grpc_res, _rustlers_res) = join! {
        grpc::server::start(conn.clone()),
        rustler.start(),
    };

    info!("Shutting down");
    Ok(())
}
