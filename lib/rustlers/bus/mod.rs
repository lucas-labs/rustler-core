use {eyre::Result, redis::Client, std::fmt::Debug};

pub mod publish;
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

/// 🐎 » represents a value that can be serialized to a redis value
pub trait ToRedisVal {
    fn to_redis_val(&self) -> Vec<(String, String)>;
}

/// 🐎 » represents a value that can be serialized to a redis key
pub trait ToRedisKey {
    fn to_redis_key(&self) -> String;
}

/// 🐎 » represents a value that can be serialized to and from a redis message
pub trait ToFromRedisMessage {
    fn as_message(&self) -> String;
    fn from_message<T: AsRef<str>>(msg: T) -> Self;
}

/// 🐎 » supertrait combining all redis object traits + debug + send + sync + 'static
pub trait RedisMessage:
    ToRedisVal + ToRedisKey + ToFromRedisMessage + Debug + Clone + Send + Sync + PartialEq + 'static
{
}

/// 🐎 » **publisher**: create bus publisher
///
/// **Arguments**
/// - `redis` - a redis client or a redis connection string
///
/// **Returns**
/// - a new `Publisher` instance
pub async fn publisher<RM: RedisMessage, RC: RedisClient>(
    redis: &RC,
) -> Result<publish::Publisher<RM>> {
    publish::Publisher::new(redis).await
}

/// 🐎 » **subscriber**: create bus subscriber
///
/// **Arguments**
/// - `redis` - a redis client or a redis connection string
///
/// **Returns**
/// - a new `Subscriber` instance
pub async fn subscriber<RM: RedisMessage, RC: RedisClient>(
    redis: &RC,
) -> Result<subscribe::Subscriber<RM>> {
    subscribe::Subscriber::new(redis).await
}

/// 🐎 » **pubsub*: create bus publisher and subscriber
///
/// **Arguments**
/// - `redis` - a redis client or a redis connection string
///
/// **Returns**
/// - a tuple containing a `Publisher` and a `Subscriber` instance
pub async fn pubsub<RO: RedisMessage, T: RedisClient>(
    redis: &T,
) -> Result<(publish::Publisher<RO>, subscribe::Subscriber<RO>)> {
    let publisher = publish::Publisher::new(redis).await?;
    let subscriber = subscribe::Subscriber::new(redis).await?;

    Ok((publisher, subscriber))
}
