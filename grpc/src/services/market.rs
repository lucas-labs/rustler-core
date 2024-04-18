use {
    self::market_mod::Empty,
    crate::services::handle_sql_err,
    entities::market,
    eyre::Result,
    lool::logger::{error, info},
    market_mod::{
        market_api_server::{MarketApi, MarketApiServer},
        Market, Markets,
    },
    std::{any::Any, fmt::Debug, time::Instant},
    tonic::{Request, Response, Status},
};

pub mod market_mod {
    tonic::include_proto!("market");
}

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
    pub(crate) svc: market::Service,
}

impl GrpcServer {
    pub fn log_if_err<T: Any, K: Debug>(&self, res: &Result<T, K>) {
        if let Err(err) = &res {
            error!("{:?}", err);
        }
    }

    pub fn svc(self) -> MarketApiServer<GrpcServer> {
        MarketApiServer::new(self)
    }
}

#[tonic::async_trait]
impl MarketApi for GrpcServer {
    async fn get_all(&self, _: Request<Empty>) -> Result<Response<Markets>, Status> {
        let start = Instant::now();
        let result = self.svc.get_all().await;
        self.log_if_err(&result);

        let response = match result {
            Ok(mkts) => Ok(Response::new(Markets {
                markets: mkts.to_owned().into_iter().map(Market::from_model).collect(),
            })),
            Err(err) => Err(handle_sql_err(err, "Getting", "markets")),
        };

        info!("`MarketApi.get_all` took {:?}", start.elapsed());
        response
    }

    async fn create(&self, market: Request<Market>) -> Result<Response<Market>, Status> {
        let start = Instant::now();
        let mkt = market.into_inner().into_model();
        let result = self.svc.create(mkt).await;
        self.log_if_err(&result);

        let response = match result {
            Ok(m) => Ok(Response::new(Market::from_model(m))),
            Err(err) => Err(handle_sql_err(err, "creating", "market")),
        };

        info!("`MarketApi.create` took {:?}", start.elapsed());
        response
    }
}
