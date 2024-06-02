use {
    super::{event, stats::ServerStats},
    async_trait::async_trait,
    eyre::Result,
    futures::{stream::SplitSink, StreamExt},
    lool::logger::{error, info},
    std::sync::Arc,
    tokio::sync::Mutex,
    tokio_tungstenite::{
        accept_hdr_async,
        tungstenite::{handshake::server::Callback, Message},
    },
};

pub use {
    tokio::net::{TcpListener, TcpStream},
    tokio_tungstenite::{
        tungstenite::{
            handshake::server::{ErrorResponse, Request, Response},
            Error,
        },
        WebSocketStream,
    },
};

pub type Outgoing = SplitSink<WebSocketStream<TcpStream>, Message>;

#[async_trait]
pub trait EventDispatcher: Send {
    async fn dispatch(
        &self,
        event: String,
        data: event::Data,
        outgoing: Arc<Mutex<Outgoing>>,
        conn_id: String,
    ) -> Result<()>;
}

/// 🐎 » **`socket::Server`**
/// --
///
/// A websocket gateway server that listens for incoming connections and dispatches events to the
/// appropriate event handlers by using the provided `EventDispatcher`.
///
/// ### Example
/// See `examples/socket.rs` for a complete example.
pub struct Server<ED>
where
    ED: EventDispatcher + Clone + Send + Sync + 'static,
{
    stats: Arc<ServerStats>,
    listener: TcpListener,
    host: String,
    port: String,
    event_dispatcher: ED,
}

impl<ED> Server<ED>
where
    ED: EventDispatcher + Clone + Send + Sync + 'static,
{
    pub async fn new(host: &str, port: &str, event_dispatcher: ED) -> std::io::Result<Self> {
        let listener = TcpListener::bind(format!("{}:{}", host, port)).await?;

        Ok(Self {
            listener,
            event_dispatcher,
            host: host.to_string(),
            port: port.to_string(),
            stats: Arc::new(ServerStats::new()),
        })
    }

    /// **🐎 » `start_no_cb`**: start the server
    pub async fn start_no_cb(&mut self) {
        let noop_cb = |_: &Request, response: Response| Ok(response);
        self.start(noop_cb).await;
    }

    /// **🐎 » `start`**
    ///
    /// Starts the server with a handshake callback. Usefull for customizing the
    /// handshake process, e.g. checking headers, etc.
    ///
    /// **Tip:** if you don't need to customize the handshake process, use
    /// `start_no_cb` instead.
    pub async fn start<HCb>(&mut self, cb: HCb)
    where
        HCb: Callback + Unpin + Clone,
    {
        info!("Started Rustler WS Server on {}:{}", self.host, self.port);

        let stats = &self.stats.clone();

        while let Ok((stream, peer)) = self.listener.accept().await {
            let dispatcher = self.event_dispatcher.clone();
            let cb = cb.clone();
            info!("Incoming connection from: {}", peer);

            // call the handshake callback
            let ws_stream = accept_hdr_async(stream, cb).await;

            if let Ok(ws_stream) = ws_stream {
                stats.inc_current_clients();
                let stats = stats.clone();
                let conn_id = uuid::Uuid::new_v4();

                tokio::spawn(async move {
                    match Server::handle_connection(ws_stream, dispatcher, conn_id).await {
                        Ok(_) => info!("Connection {} closed", conn_id),
                        Err(e) => error!("Error handling connection: {:?}", e),
                    };

                    // decrement client count
                    stats.clone().dec_current_clients();
                    info!("{:?}", stats);
                });
            }

            info!("{:?}", stats);
        }
    }

    /// subscribe to incoming messages
    async fn handle_connection(
        stream: WebSocketStream<TcpStream>,
        event_dispatcher: ED,
        conn_id: uuid::Uuid,
    ) -> Result<()> {
        let (outgoing, mut incoming) = stream.split();
        let synced_outgoing = Arc::new(Mutex::new(outgoing));

        while let Some(msg) = incoming.next().await {
            Server::handle_message(msg?, &event_dispatcher, synced_outgoing.clone(), conn_id)
                .await?;
        }

        Ok(())
    }

    /// handle an incoming message
    async fn handle_message(
        msg: Message,
        event_dispatcher: &ED,
        outgoing: Arc<Mutex<Outgoing>>,
        conn_id: uuid::Uuid,
    ) -> Result<HandlingResult> {
        if msg.is_text() || msg.is_binary() {
            if let Ok(event) = serde_json::from_str::<event::WsEvent>(&msg.to_string()) {
                let outgoing = Arc::clone(&outgoing);
                let result = event_dispatcher
                    .dispatch(event.event, event.data, outgoing, conn_id.into())
                    .await;

                match result {
                    Ok(_) => {}
                    Err(e) => {
                        error!("Error dispatching event: {:?}", e);
                    }
                };
            }
        }

        if msg.is_close() {
            return Ok(HandlingResult::Closed);
        }

        // TODO: should we handle ping/pong messages?

        Ok(HandlingResult::Handled)
    }
}

#[derive(PartialEq)]
enum HandlingResult {
    Handled,
    Closed,
}
