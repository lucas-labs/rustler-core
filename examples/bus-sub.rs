use {
    eyre::{set_hook, DefaultHandler, Result},
    rustler_core::rustlers::{
        bus::{self, SubscriberTrait},
        Quote, Ticker,
    },
    rxrust::observable::{ObservableExt, ObservableItem},
};

#[tokio::main]
async fn main() -> Result<()> {
    set_hook(Box::new(DefaultHandler::default_with))?;

    let mut sx = bus::redis::subscriber::<Quote, _>(&"redis://127.0.0.1/").await?;

    let ticker = Ticker {
        market: "BINANCE".to_string(),
        symbol: "BTCUSDT".to_string(),
    };

    let _obs = sx.stream().await?.filter(move |quote| quote.belongs_to(&ticker)).subscribe(|v| {
        println!("Received quote: {}", v);
    });

    // wait for 10 seconds
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

    Ok(())
}
