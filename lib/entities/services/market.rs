use {
    crate::entities::{
        market::{Entity as Market, Model as MarketModel},
        ticker::{Entity as Ticker, Model as TickerModel},
    },
    eyre::Result,
    sea_orm::{DatabaseConnection, DbErr, EntityTrait, IntoActiveModel},
};

/// 🤠 » service for the `Market` entity
pub struct Service {
    conn: DatabaseConnection,
}

impl Service {
    pub async fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }

    /// 🤠 » gets all markets from the database
    pub async fn get_all(&self) -> Result<Vec<MarketModel>, DbErr> {
        let markets = Market::find().all(&self.conn).await?;
        Ok(markets)
    }

    /// 🤠 » gets a market by its id
    pub async fn create(&self, market: MarketModel) -> Result<MarketModel, DbErr> {
        Market::insert(market.clone().into_active_model()).exec(&self.conn).await?;
        Ok(market)
    }

    /// 🤠 » gets all markets with their tickers
    pub async fn get_all_with_tickers(
        &self,
    ) -> Result<Vec<(MarketModel, Vec<TickerModel>)>, DbErr> {
        let markets_with_tickers: Vec<(MarketModel, Vec<TickerModel>)> =
            Market::find().find_with_related(Ticker).all(&self.conn).await?;

        Ok(markets_with_tickers)
    }
}
