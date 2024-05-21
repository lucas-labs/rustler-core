mod binance;

use {
    binance::FooRustler,
    dotenvy::dotenv,
    eyre::{set_hook, DefaultHandler, Result},
    lool::logger::{info, ConsoleLogger, Level},
    rustler_core::{
        bus,
        entities::db::get_connection,
        grpc, rustlerjar,
        rustlers::{svc::RustlersSvc, Quote},
    },
    tokio::join,
};

#[tokio::main]
async fn main() -> Result<()> {
    set_hook(Box::new(DefaultHandler::default_with))?;
    ConsoleLogger::default_setup(Level::Trace, "rustler")?;

    dotenv()?;
    let publisher = bus::redis::publisher::<Quote, _>(&"redis://127.0.0.1/").await?;

    let conn = get_connection().await?;
    let mut rustler = RustlersSvc::new(
        conn.clone(),
        rustlerjar! {
            "BINANCE" => FooRustler::create
        },
        publisher,
    )
    .await;

    let (_grpc_res, _rustlers_res) = join! {
        grpc::server::start(conn.clone()),
        rustler.start(),
    };

    info!("Shutting down");
    Ok(())
}
