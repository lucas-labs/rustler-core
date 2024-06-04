use {
    eyre::Result,
    futures::Stream,
    lool::fail,
    std::pin::Pin,
    tokio::sync::broadcast::{self, Sender},
};

pub trait StreamMsg: Clone + Send + Sync + 'static {}

// TODO: move this to a separate module, or maybe to the lool library
pub struct SourceStream<RM: StreamMsg> {
    sender: Option<Sender<RM>>,
}

impl<RM: StreamMsg> Default for SourceStream<RM> {
    fn default() -> Self {
        Self::new()
    }
}

impl<RM: StreamMsg> SourceStream<RM> {
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
            let mut receiver = sender.subscribe();

            let stream = async_stream::stream! {
                while let Ok(item) = receiver.recv().await {
                    yield item;
                }
            };

            Ok(Box::pin(stream))
        } else {
            fail!("SourceStream has been consumed")
        }
    }
}
