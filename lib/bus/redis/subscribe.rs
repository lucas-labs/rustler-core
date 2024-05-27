use {
    super::{key, stream::SourceStream, PrefixedPubSub, RedisClient, KEY_PREFIX},
    crate::bus::{BusMessage, SubscriberTrait},
    eyre::Result,
    futures::{Stream, StreamExt},
    lool::{fail, s},
    std::pin::Pin,
    tonic::async_trait,
};

// IDEA: create another version using tokio broadcast channels
// https://github.com/exein-io/pulsar/blob/99ad35c8d13eaf1a37d7b6a9dcb812a5a1231d00/crates/pulsar-core/src/bus.rs

/// 🐎 » bus **Subscriber**
///
/// allows to subscribe to a redis key pattern and receive messages from the redis bus
pub struct RedisSubscriber<RM: BusMessage> {
    // TODO: replace with storing tokio multiplexed connection like in publish.rs when redis@0.26.0
    //       is released see https://github.com/redis-rs/redis-rs/issues/1137.
    //       this way we can just clone the connection when needing instead of storing the
    //       redis client and creating a new connection
    client: redis::Client,
    key_prefix: String,
    pattern: String,
    pub source_stream: Option<SourceStream<RM>>,
}

impl<RM: BusMessage> PrefixedPubSub for RedisSubscriber<RM> {
    fn get_prefix(&self) -> String {
        self.key_prefix.clone()
    }

    fn set_prefix(&mut self, prefix: &str) -> &mut Self {
        self.key_prefix = s!(prefix);
        self
    }
}

impl<RM: BusMessage> RedisSubscriber<RM> {
    /// 🐎 » create a new bus subscriber
    pub async fn new<RC>(redis: &RC) -> Result<Self>
    where
        RC: RedisClient,
    {
        Ok(Self {
            pattern: s!("*"),
            client: redis.get_client()?,
            key_prefix: s!(KEY_PREFIX),
            source_stream: None,
        })
    }

    /// 🐎 » set the pattern to subscribe to
    pub fn with_pattern(&mut self, pattern: &str) -> &mut Self {
        self.pattern = s!(pattern);
        self
    }

    /// 🐎 » returns the pattern used to subscribe to the redis bus, including the prefix if set
    pub fn get_pattern(&self) -> String {
        key(self.get_prefix(), self.pattern.clone())
    }

    /// subscribe to the redis feed
    async fn start_streaming(&mut self) -> Result<()> {
        if self.source_stream.is_none() {
            self.source_stream = Some(SourceStream::new());
        }

        let pattern = self.pattern.clone();
        let mut conn = self.client.get_async_pubsub().await?;
        let prefix = self.get_prefix();

        if let Some(stream) = self.source_stream.as_mut() {
            let sender = stream.sender().unwrap();

            tokio::spawn(async move {
                conn.psubscribe(key(prefix, pattern)).await?;

                let mut msg_stream = conn.on_message();
                while let Some(msg) = msg_stream.next().await {
                    if let Ok(payload) = msg.get_payload::<String>() {
                        // TODO: handle possible panic when parsing message
                        //       using catch_unwind
                        let message = RM::from_message(payload);
                        let _ = sender.send(message);
                    }
                }

                Result::<()>::Ok(())
            });
        }

        Ok(())
    }
}

#[async_trait]
impl<RM: BusMessage> SubscriberTrait<RM> for RedisSubscriber<RM> {
    async fn stream(&mut self) -> Result<Pin<Box<dyn Stream<Item = RM> + Send + 'static>>> {
        if self.source_stream.is_none() {
            self.start_streaming().await?;
        }

        match self.source_stream.as_ref() {
            Some(stream) => stream.subscribe(),
            None => fail!("Could not start streaming messages from redis bus"),
        }
    }
}
