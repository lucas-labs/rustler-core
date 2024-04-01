use {
    self::market_mod::Empty,
    entities::{market, sea_orm::DatabaseConnection},
    eyre::Result,
    lool::{cli::stylize::Stylize, logger::info},
    market_mod::{
        market_api_server::{MarketApi, MarketApiServer},
        Market, Markets,
    },
    std::{net::SocketAddr, sync::Arc, time::Instant},
    tonic::{transport::Server, Request, Response, Status},
};

pub mod market_mod {
    tonic::include_proto!("market");
}

const RUSTLER_GRPC_API_ADDR: &str = "RUSTLER_GRPC_API_ADDR";

impl Market {
    fn into_model(self) -> market::Model {
        market::Model {
            id: self.id,
            short_name: self.short_name,
            full_name: self.full_name,
            opens_from: self.opens_from,
            opens_till: self.opens_till,
            open_time: self.open_time,
            close_time: self.close_time,
            pre_market_offset: self.pre_market_offset,
            post_market_offset: self.post_market_offset,
            time_zone_offset: self.time_zone_offset,
        }
    }

    fn from_model(model: market::Model) -> Self {
        Self {
            id: model.id,
            short_name: model.short_name,
            full_name: model.full_name,
            opens_from: model.opens_from,
            opens_till: model.opens_till,
            open_time: model.open_time,
            close_time: model.close_time,
            pre_market_offset: model.pre_market_offset,
            post_market_offset: model.post_market_offset,
            time_zone_offset: model.time_zone_offset,
        }
    }
}

pub struct GrpcServer {
    svc: market::Service,
}

#[tonic::async_trait]
impl MarketApi for GrpcServer {
    async fn get_all(&self, _: Request<Empty>) -> Result<Response<Markets>, Status> {
        let start = Instant::now();
        let response = if let Ok(mkts) = self.svc.get_all().await {
            Ok(Response::new(Markets {
                markets: mkts.into_iter().map(Market::from_model).collect(),
            }))
        } else {
            Err(Status::internal("Failed to get markets"))
        };

        info!("`MarketApi.get_all` took {:?}", start.elapsed());

        response
    }

    async fn create(&self, market: Request<Market>) -> Result<Response<Market>, Status> {
        let start = Instant::now();
        let mkt = market.into_inner().into_model();

        let response = if let Ok(m) = self.svc.create(mkt).await {
            Ok(Response::new(Market::from_model(m)))
        } else {
            Err(Status::internal("Failed to create market"))
        };

        info!("`MarketApi.create` took {:?}", start.elapsed());

        response
    }
}

/// Starts the gRPC server
pub async fn start(conn: Arc<DatabaseConnection>) -> Result<()> {
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

    let svc = market::Service::new(conn).await;
    let server = GrpcServer { svc };

    info!(
        "🎉 gRPC server listening on {}",
        addr.clone().to_string().green()
    );
    Server::builder().add_service(MarketApiServer::new(server)).serve(addr).await?;

    Ok(())
}
