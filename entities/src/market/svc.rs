use {
    super::{Entity as MarketEntity, Model as MarketModel},
    eyre::Result,
    sea_orm::{DatabaseConnection, EntityTrait, IntoActiveModel},
    std::{
        sync::{Arc, Mutex},
        time::Instant,
    },
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

    pub async fn create(
        &self,
        id: &str,
        short_name: &str,
        full_name: &str,
        opens_from: i32,
        opens_till: i32,
        open_time: &str,
        close_time: &str,
        pre_market_offset: i32,
        post_market_offset: i32,
        time_zone_offset: i32,
    ) -> Result<MarketModel> {
        let mut start = Instant::now();

        let market = MarketModel {
            id: id.to_string(),
            short_name: short_name.to_string(),
            full_name: full_name.to_string(),
            opens_from,
            opens_till,
            open_time: open_time.to_string(),
            close_time: close_time.to_string(),
            pre_market_offset,
            post_market_offset,
            time_zone_offset,
        };

        println!("create market model took {:?}", start.elapsed());
        start = Instant::now();

        MarketEntity::insert(market.clone().into_active_model()).exec(&*self.conn).await?;

        println!("insert market model took {:?}", start.elapsed());
        Ok(market)
    }

    pub async fn create_from_model(&self, market: MarketModel) -> Result<MarketModel> {
        let start = Instant::now();
        MarketEntity::insert(market.clone().into_active_model()).exec(&*self.conn).await?;
        println!("insert market model took {:?}", start.elapsed());
        Ok(market)
    }
}
