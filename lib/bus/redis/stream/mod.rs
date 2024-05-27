use {
    eyre::Result,
    futures::Stream,
    lool::fail,
    std::{
        pin::Pin,
        task::{Context, Poll},
    },
    tokio::sync::broadcast::{self, Receiver, Sender},
};

use crate::bus::BusMessage;

pub struct SourceStream<RM: BusMessage> {
    sender: Option<Sender<RM>>,
}

impl<RM: BusMessage> Default for SourceStream<RM> {
    fn default() -> Self {
        Self::new()
    }
}

impl<RM: BusMessage> SourceStream<RM> {
    // Create a new SourceStream with a broadcast channel
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(100); // Adjust the buffer size as needed
        SourceStream {
            sender: Some(sender),
        }
    }

    pub fn sender(&self) -> Option<Sender<RM>> {
        self.sender.clone()
    }

    // Subscribe to the stream
    pub fn subscribe(&self) -> Result<Pin<Box<dyn Stream<Item = RM> + Send + 'static>>> {
        if let Some(sender) = &self.sender {
            let receiver = sender.subscribe();
            Ok(Box::pin(BroadcastStream { receiver }))
        } else {
            fail!("SourceStream has been consumed")
        }
    }
}

// Wrapper around Receiver to implement Stream
struct BroadcastStream<RM: BusMessage> {
    receiver: Receiver<RM>,
}

impl<RM: BusMessage> Stream for BroadcastStream<RM> {
    type Item = RM;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        // Spawn an async task to receive the message
        tokio::task::block_in_place(|| match futures::executor::block_on(this.receiver.recv()) {
            Ok(msg) => Poll::Ready(Some(msg)),
            Err(broadcast::error::RecvError::Closed) => Poll::Ready(None),
            Err(broadcast::error::RecvError::Lagged(_)) => Poll::Pending,
        })
    }
}
