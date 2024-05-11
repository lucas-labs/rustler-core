use {
    crate::entities::{
        orm::ticker,
        ticker::{Entity as Ticker, Model as TickerModel},
    },
    eyre::Result,
    sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, IntoActiveModel, QueryFilter},
};

/// 🐎 » service for the `Ticker` entity
pub struct Service {
    conn: DatabaseConnection,
}

impl Service {
    /// 🐎 » creates a new `Ticker` service
    pub async fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }

    /// 🐎 » retrieves all tickers from the database
    pub async fn get_all(&self) -> Result<Vec<TickerModel>, DbErr> {
        let tickers = Ticker::find().all(&self.conn).await?;
        Ok(tickers)
    }

    /// 🐎 » retrieves a ticker from the database, given its id
    pub async fn get(&self, id: String) -> Result<Option<TickerModel>, DbErr> {
        let ticker = Ticker::find_by_id(id).one(&self.conn).await?;
        Ok(ticker)
    }

    /// 🐎 » retrieves a ticker from the database, given its symbol
    pub async fn get_by_symbol(&self, symbol: String) -> Result<Option<TickerModel>, DbErr> {
        let ticker =
            Ticker::find().filter(ticker::Column::Symbol.eq(symbol)).one(&self.conn).await?;

        Ok(ticker)
    }

    /// 🐎 » creates a new ticker in the database
    pub async fn create(&self, market: TickerModel) -> Result<TickerModel, DbErr> {
        Ticker::insert(market.clone().into_active_model()).exec(&self.conn).await?;
        Ok(market)
    }
}
