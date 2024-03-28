use {
    super::{Entity as MarketEntity, Model as MarketModel},
    eyre::Result,
    sea_orm::{DatabaseConnection, EntityTrait, IntoActiveModel},
    std::{sync::Arc, time::Instant},
};

pub struct Service {
    conn: Arc<DatabaseConnection>,
}

impl Service {
    pub async fn new(conn: Arc<DatabaseConnection>) -> Self {
        Self { conn }
    }

    pub async fn get_all(&self) -> Result<Vec<MarketModel>> {
        // let conn = self.conn.unwrap();
        let markets = MarketEntity::find().all(&*self.conn).await?;
        Ok(markets)
    }

    pub async fn create(&self, market: MarketModel) -> Result<MarketModel> {
        let start = Instant::now();
        MarketEntity::insert(market.clone().into_active_model()).exec(&*self.conn).await?;
        println!("insert market model took {:?}", start.elapsed());
        Ok(market)
    }
}
