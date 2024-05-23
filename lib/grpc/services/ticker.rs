use {
    crate::{entities::ticker, grpc::services::handle_sql_err},
    eyre::Result,
    lool::logger::{error, info},
    std::{any::Any, fmt::Debug, time::Instant},
    ticker_mod::{
        ticker_api_server::{TickerApi, TickerApiServer},
        Empty, Ticker, TickerId, Tickers,
    },
    tonic::{Request, Response, Status},
};

pub mod ticker_mod {
    tonic::include_proto!("ticker");
}

impl Ticker {
    /// 🐎 » converts a `Ticker` entity from gRPC to a database sea-orm `ticker::Model`
    fn into_model(self) -> ticker::Model {
        ticker::Model {
            id: self.id,
            symbol: self.symbol,
            quote_symbol: self.quote_symbol,
            market_id: self.market_id,
        }
    }

    /// 🐎 » converts a `ticker::Model` database entity to a gRPC `Ticker` entity
    fn from_model(model: ticker::Model) -> Self {
        Self {
            id: model.id,
            symbol: model.symbol,
            quote_symbol: model.quote_symbol,
            market_id: model.market_id,
        }
    }
}

/// 🐎 » grpc Server to manage ticker entities
pub struct GrpcServer {
    pub(crate) svc: ticker::Service,
}

impl GrpcServer {
    pub fn log_if_err<T: Any, K: Debug>(&self, res: &Result<T, K>) {
        if let Err(err) = &res {
            error!("{:?}", err);
        }
    }

    /// 🐎 » creates the ticker api server
    pub fn svc(self) -> TickerApiServer<GrpcServer> {
        TickerApiServer::new(self)
    }
}

#[tonic::async_trait]
impl TickerApi for GrpcServer {
    /// retrieves and returns a ticker entity from the database, given its id
    async fn get(&self, req: Request<TickerId>) -> Result<Response<Ticker>, Status> {
        let start = Instant::now();
        let mkt = req.into_inner();
        let result = self.svc.get(mkt.id).await;
        self.log_if_err(&result);

        let response = match result {
            Ok(m) => {
                if let Some(m) = m {
                    Ok(Response::new(Ticker::from_model(m)))
                } else {
                    Err(Status::not_found("Ticker not found"))
                }
            }
            Err(err) => Err(handle_sql_err(err, "Getting", "ticker")),
        };

        info!("`TickerApi.get` took {:?}", start.elapsed());
        response
    }

    /// retrieves and returns all ticker entities from the database
    async fn get_all(&self, _: Request<Empty>) -> Result<Response<Tickers>, Status> {
        let start = Instant::now();
        let result = self.svc.get_all().await;
        self.log_if_err(&result);

        let response = match result {
            Ok(mkts) => Ok(Response::new(Tickers {
                tickers: mkts.iter().cloned().map(Ticker::from_model).collect(),
            })),
            Err(err) => Err(handle_sql_err(err, "Getting", "tickers")),
        };

        info!("`TickerApi.get_all` took {:?}", start.elapsed());
        response
    }

    /// creates a new ticker entity in the database
    async fn create(&self, market: Request<Ticker>) -> Result<Response<Ticker>, Status> {
        let start = Instant::now();
        let mkt = market.into_inner().into_model();
        let result = self.svc.create(mkt).await;
        self.log_if_err(&result);

        let response = match result {
            Ok(m) => Ok(Response::new(Ticker::from_model(m))),
            Err(err) => Err(handle_sql_err(err, "creating", "ticker")),
        };

        info!("`TickerApi.create` took {:?}", start.elapsed());
        response
    }
}
