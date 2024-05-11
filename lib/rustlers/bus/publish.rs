use {
    super::{key, PrefixedPubSub, RedisClient, RedisMessage, KEY_PREFIX},
    eyre::Result,
    redis::{aio::MultiplexedConnection, AsyncCommands},
};

/// 🐎 » bus **Publisher**
///
/// allows to push a message or resource to the bus
#[derive(Clone)]
pub struct Publisher<RM: RedisMessage> {
    conn: MultiplexedConnection,
    key_prefix: String,
    resource_type: std::marker::PhantomData<RM>,
}

impl<RM: RedisMessage> PrefixedPubSub for Publisher<RM> {
    fn get_prefix(&self) -> String {
        self.key_prefix.clone()
    }

    fn set_prefix(&mut self, prefix: &str) -> &mut Self {
        self.key_prefix = prefix.to_string();
        self
    }
}

impl<RM: RedisMessage> Publisher<RM> {
    /// 🐎 » create a new bus publisher
    pub async fn new<RC>(redis: &RC) -> Result<Self>
    where
        RC: RedisClient,
    {
        let redis = redis.get_client()?;
        let conn = redis.get_multiplexed_tokio_connection().await?;

        Ok(Self {
            conn,
            key_prefix: KEY_PREFIX.to_string(),
            resource_type: std::marker::PhantomData,
        })
    }

    /// 🐎 » publish a message to the bus
    pub async fn publish(&mut self, value: RM) -> Result<()> {
        let obj_key = key(self.get_prefix(), value.to_redis_key());
        // set hash key
        self.conn.hset_multiple(&obj_key, value.to_redis_val().as_slice()).await?;

        // publish to the appropriate channel
        self.conn.publish(&obj_key, value.as_message()).await?;
        Ok(())
    }
}
