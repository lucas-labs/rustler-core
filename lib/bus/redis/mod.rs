use {super::BusMessage, eyre::Result, redis::Client};

pub mod publish;
pub mod stream;
pub mod subscribe;

pub(crate) const KEY_PREFIX: &str = "rustler";

/// creates a message redis key from a prefix and a key
pub(crate) fn key<T: AsRef<str>, K: AsRef<str>>(prefix: T, key: K) -> String {
    let prefix = prefix.as_ref();
    let key = key.as_ref();

    match prefix.is_empty() {
        true => key.to_string(),
        false => format!("{}:{}", prefix, key),
    }
}

/// represents a pub or sub handler that can be prefixed
pub trait PrefixedPubSub {
    fn get_prefix(&self) -> String;
    fn set_prefix(&mut self, prefix: &str) -> &mut Self;

    fn with_prefix(&mut self, prefix: &str) -> &mut Self {
        self.set_prefix(prefix);
        self
    }

    fn without_prefix(&mut self) -> &mut Self {
        self.set_prefix("");
        self
    }
}

/// 🐎 » represents a an entity that can provide a redis client
pub trait RedisClient {
    fn get_client(&self) -> Result<Client>;
}

impl RedisClient for Client {
    fn get_client(&self) -> Result<Client> {
        let redis = self.clone();
        Ok(redis)
    }
}

impl RedisClient for &str {
    fn get_client(&self) -> Result<Client> {
        let redis = Client::open(*self)?;
        Ok(redis)
    }
}

impl RedisClient for String {
    fn get_client(&self) -> Result<Client> {
        let redis = Client::open(self.as_str())?;
        Ok(redis)
    }
}

/// 🐎 » **publisher**: create bus publisher
///
/// **Arguments**
/// - `redis` - a redis client or a redis connection string
///
/// **Returns**
/// - a new `Publisher` instance
pub async fn publisher<RM: BusMessage, RC: RedisClient>(
    redis: &RC,
) -> Result<publish::RedisPublisher<RM>> {
    publish::RedisPublisher::new(redis).await
}

/// 🐎 » **subscriber**: create bus subscriber
///
/// **Arguments**
/// - `redis` - a redis client or a redis connection string
///
/// **Returns**
/// - a new `Subscriber` instance
pub async fn subscriber<RM: BusMessage, RC: RedisClient>(
    redis: &RC,
) -> Result<subscribe::RedisSubscriber<RM>> {
    subscribe::RedisSubscriber::new(redis).await
}

/// 🐎 » **pubsub*: create bus publisher and subscriber
///
/// **Arguments**
/// - `redis` - a redis client or a redis connection string
///
/// **Returns**
/// - a tuple containing a `Publisher` and a `Subscriber` instance
pub async fn pubsub<RO: BusMessage, T: RedisClient>(
    redis: &T,
) -> Result<(publish::RedisPublisher<RO>, subscribe::RedisSubscriber<RO>)> {
    let publisher = publish::RedisPublisher::new(redis).await?;
    let subscriber = subscribe::RedisSubscriber::new(redis).await?;

    Ok((publisher, subscriber))
}
