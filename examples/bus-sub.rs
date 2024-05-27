use {
    eyre::{set_hook, DefaultHandler, Result},
    futures::{future, StreamExt},
    rustler_core::{
        bus::{self, SubscriberTrait},
        rustlers::{Quote, Ticker},
    },
    tokio::sync::mpsc,
};

#[tokio::main]
async fn main() -> Result<()> {
    set_hook(Box::new(DefaultHandler::default_with))?;

    let (cancel_tx, mut cancel_rx) = mpsc::channel::<()>(1);

    let mut sx = bus::redis::subscriber::<Quote, _>(&"redis://127.0.0.1/").await?;

    let ticker = Ticker {
        market: "BINANCE".to_string(),
        symbol: "BTCUSDT".to_string(),
        quote_asset: None,
    };

    let mut stream =
        sx.stream().await?.filter(move |quote| future::ready(quote.belongs_to(&ticker)));

    tokio::spawn(async move {
        // cancel the streaming after 10 seconds
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        println!("Cancelling stream");
        cancel_tx.send(()).await.unwrap();
    });

    while let Some(quote) = stream.next().await {
        println!("Received quote: {}", quote);
        if cancel_rx.try_recv().is_ok() {
            break;
        }
    }

    println!("Stream cancelled");

    Ok(())
}
