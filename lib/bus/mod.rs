use std::{fmt::Debug, pin::Pin};

use futures::Stream;

use {eyre::Result, tonic::async_trait};

pub mod redis;

/// 🐎 » represents a value that can be serialized to a bus value
pub trait ToBusVal {
    fn to_bus_val(&self) -> Vec<(String, String)>;
}

/// 🐎 » represents a value that can be serialized to a bus key
pub trait ToBusKey {
    fn to_bus_key(&self) -> String;
}

/// 🐎 » represents a value that can be serialized to and from a bus message
pub trait ToFromBusMessage {
    fn as_message(&self) -> String;
    fn from_message<T: AsRef<str>>(msg: T) -> Self;
}

/// 🐎 » supertrait combining all bus object traits + debug + send + sync + 'static
pub trait BusMessage:
    ToBusVal + ToBusKey + ToFromBusMessage + Debug + Clone + Send + Sync + PartialEq + 'static
{
}

/// 🐎 » trait for bus **Publisher**s
#[async_trait]
pub trait PublisherTrait<RM: BusMessage> {
    /// 🐎 » publish a message to the bus
    async fn publish(&mut self, value: RM) -> Result<()>;
}

/// 🐎 » trait for bus **Publisher**s
#[async_trait]
pub trait SubscriberTrait<RM: BusMessage> {
    /// 🐎 » **stream**
    ///
    /// returns an `Observable` stream of messages from the redis bus
    async fn stream(&mut self) -> Result<Pin<Box<dyn Stream<Item = RM> + Send + 'static>>>;
}
