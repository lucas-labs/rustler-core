use {
    core::time,
    entities::{market, sea_orm::DatabaseConnection},
    eyre::Result,
    std::{env, sync::Arc, thread, time::Instant},
    tonic::{transport::Server, Request, Response, Status},
};

pub mod market_mod {
    tonic::include_proto!("market");
}

use market_mod::{
    market_api_server::{MarketApi, MarketApiServer},
    Market, Markets,
};

use self::market_mod::Empty;

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
        if let Some(mkts) = self.svc.get_all().await.ok() {
            println!("get_all took {:?}", start.elapsed());
            Ok(Response::new(Markets {
                markets: mkts.into_iter().map(|m| Market::from_model(m)).collect(),
            }))
        } else {
            println!("get_all took {:?}", start.elapsed());
            Err(Status::internal("Failed to get markets"))
        }
    }

    async fn create(&self, market: Request<Market>) -> Result<Response<Market>, Status> {
        let start = Instant::now();
        let mkt = market.into_inner().into_model();

        if let Some(m) = self.svc.create_from_model(mkt).await.ok() {
            println!("create took {:?}", start.elapsed());
            Ok(Response::new(Market::from_model(m)))
        } else {
            println!("create took {:?} on err", start.elapsed());
            Err(Status::internal("Failed to create market"))
        }
    }
}

/// Starts the gRPC server
pub async fn start(conn: Arc<DatabaseConnection>) -> Result<()> {
    let addr = "0.0.0.0:50051".parse()?;
    let svc = market::Service::new(conn).await;

    let server = GrpcServer { svc };
    Server::builder().add_service(MarketApiServer::new(server)).serve(addr).await?;

    Ok(())
}
