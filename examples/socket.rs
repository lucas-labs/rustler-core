use {
    async_trait::async_trait,
    eyre::Result,
    futures::{SinkExt, StreamExt},
    lool::logger::{error, info, ConsoleLogger, Level},
    rustler_core::{
        bus::{self, SubscriberTrait},
        rustlers::Quote,
        socket::{self, event, Error, EventDispatcher, Outgoing, Request, Response},
    },
    std::sync::Arc,
    tokio::{join, sync::Mutex},
};

#[derive(Clone)]
struct Dispatcher {}

#[async_trait]
impl EventDispatcher for Dispatcher {
    async fn dispatch(
        &self,
        event: String,
        data: event::Data,
        outgoing: Arc<Mutex<Outgoing>>,
        conn_id: String,
    ) -> Result<()> {
        info!("Event: {}", event);
        info!("Data: {:?}", data);
        info!("Connection ID: {}", conn_id);

        let mut sx = bus::redis::subscriber::<Quote, _>(&"redis://127.0.0.1/").await?;
        let mut quote_feed = sx.stream().await?;

        tokio::spawn(async move {
            while let Some(quote) = quote_feed.next().await {
                let response = serde_json::to_string(&quote).unwrap();
                let mut o = outgoing.lock().await;

                if let Err(e) = o.send(response.into()).await {
                    // if error is AlreadyClosed or ConnectionClosed, then break the loop
                    match e {
                        Error::AlreadyClosed | Error::ConnectionClosed => {
                            break;
                        }
                        _ => {
                            error!("Error sending message: {:?}", e);
                        }
                    }
                }
            }

            info!("Hasta la vista, baby!");
        });

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    ConsoleLogger::builder()
        .with_level(Level::Trace)
        .with_name("rustler")
        .ignore("tungstenite::protocol")
        .ignore("tungstenite::protocol::frame*")
        .ignore("tokio_tungstenite::compat*")
        .ignore("tokio_tungstenite")
        .install()?;

    let dispatcher = Dispatcher {};
    let mut ws_server = socket::Server::new("127.0.0.1", "9002", dispatcher).await?;

    let handshaker = |_res: &Request, response: Response| {
        Ok(response)

        // or fail the handshake, e.g. because of authentication failure
        //
        // let (mut parts, _) = response.into_parts();
        // parts.status = StatusCode::UNAUTHORIZED;
        // let res = ErrorResponse::from_parts(parts, None);
        // Err(res)
        //
        // or
        // let res = Response::builder().status(401).body(None).unwrap();
        // Err(res)
    };

    join!(ws_server.start(handshaker));

    Ok(())
}
