use {
    crate::services,
    entities::{market, sea_orm::DatabaseConnection, ticker},
    eyre::Result,
    lool::{cli::stylize::Stylize, logger::info},
    std::net::SocketAddr,
    tonic::transport::Server,
};

const RUSTLER_GRPC_API_ADDR: &str = "RUSTLER_GRPC_API_ADDR";

/// Starts the gRPC server
pub async fn start(conn: DatabaseConnection) -> Result<()> {
    fn get_default_addr() -> String {
        let addr = "0.0.0.0:50051";
        info!(
            "`{}` not set, using default {}",
            RUSTLER_GRPC_API_ADDR.italic(),
            addr.green()
        );
        addr.to_owned()
    }

    let addr: SocketAddr =
        std::env::var(RUSTLER_GRPC_API_ADDR).unwrap_or_else(|_| get_default_addr()).parse()?;

    let market_db = market::Service::new(conn.clone()).await;
    let ticker_db = ticker::Service::new(conn.clone()).await;

    let market_grpc = services::market::GrpcServer { svc: market_db };
    let ticker_grpc = services::ticker::GrpcServer { svc: ticker_db };

    info!(
        "🎉 gRPC server listening on {}",
        addr.clone().to_string().green()
    );

    Server::builder()
        .add_service(market_grpc.svc())
        .add_service(ticker_grpc.svc())
        .serve(addr)
        .await?;

    Ok(())
}
