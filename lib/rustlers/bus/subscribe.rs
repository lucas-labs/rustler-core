use {
    super::{key, PrefixedPubSub, RedisClient, RedisMessage, KEY_PREFIX},
    eyre::Result,
    futures::StreamExt,
    lool::{fail, s},
    rxrust::{
        observable::BoxIt, observer::Observer, ops::box_it::CloneableBoxOpThreads,
        subject::SubjectThreads, subscription::Subscription,
    },
    std::convert::Infallible,
};

// IDEA: create another version using tokio broadcast channels
// https://github.com/exein-io/pulsar/blob/99ad35c8d13eaf1a37d7b6a9dcb812a5a1231d00/crates/pulsar-core/src/bus.rs

/// 🐎 » bus **Subscriber**
pub struct Subscriber<RM: RedisMessage> {
    // TODO: replace with storing tokio multiplexed connection like in publish.rs when redis@0.26.0
    //       is released see https://github.com/redis-rs/redis-rs/issues/1137.
    //       this way we can just clone the connection when needing instead of storing the
    //       redis client and creating a new connection
    client: redis::Client,
    subject: Option<SubjectThreads<RM, Infallible>>,
    key_prefix: String,
    pattern: String,
}

impl<RM: RedisMessage> PrefixedPubSub for Subscriber<RM> {
    fn get_prefix(&self) -> String {
        self.key_prefix.clone()
    }

    fn set_prefix(&mut self, prefix: &str) -> &mut Self {
        self.key_prefix = s!(prefix);
        self
    }
}

impl<RM: RedisMessage> Subscriber<RM> {
    /// 🐎 » create a new bus subscriber
    pub async fn new<RC>(redis: &RC) -> Result<Self>
    where
        RC: RedisClient,
    {
        Ok(Self {
            pattern: s!("*"),
            client: redis.get_client()?,
            key_prefix: s!(KEY_PREFIX),
            subject: None,
        })
    }

    pub fn with_pattern(&mut self, pattern: &str) -> &mut Self {
        self.pattern = s!(pattern);
        self
    }

    pub fn get_pattern(&self) -> String {
        key(self.get_prefix(), self.pattern.clone())
    }

    /// 🐎 » subscribe to a channel
    pub async fn start_streaming(&mut self) -> Result<()> {
        if self.subject.is_none() {
            self.subject = Some(SubjectThreads::default());
        }

        if self.subject.is_some() && self.subject.as_ref().unwrap().is_closed() {
            drop(self.subject.take());
            self.subject = Some(SubjectThreads::default());
        }

        let pattern = self.pattern.clone();
        let mut conn = self.client.get_async_pubsub().await?;
        let prefix = self.get_prefix();

        if let Some(subject) = self.subject.as_mut() {
            let mut stream = subject.clone();

            tokio::spawn(async move {
                conn.psubscribe(key(prefix, pattern)).await?;

                let mut msg_stream = conn.on_message();
                while let Some(msg) = msg_stream.next().await {
                    if let Ok(payload) = msg.get_payload::<String>() {
                        // TODO: handle possible panic when parsing message
                        //       using catch_unwind
                        let message = RM::from_message(payload);
                        stream.next(message);
                    }
                }

                // if the connection with redis is closed, complete the stream
                stream.complete();

                Result::<()>::Ok(())
            });
        }

        Ok(())
    }

    pub async fn stream(&mut self) -> Result<CloneableBoxOpThreads<RM, Infallible>> {
        if self.subject.is_none() {
            self.start_streaming().await?;
        }

        match self.subject.as_ref() {
            Some(subject) => Ok(subject.clone().box_it()),
            None => fail!("Could not start streaming messages from redis bus"),
        }
    }
}
