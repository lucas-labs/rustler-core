mod binance;

use {
    eyre::{set_hook, DefaultHandler, Result},
    lool::s,
    rustler_core::rustlers::{
        bus::{self, PublisherTrait},
        MarketHourType, Quote,
    },
};

#[tokio::main]
async fn main() -> Result<()> {
    set_hook(Box::new(DefaultHandler::default_with))?;
    let mut px = bus::redis::publisher(&"redis://127.0.0.1/").await?;
    let variations = vec![-4.3, -1.1, 2.0, -0.5, 1.5, -1.3, 0.7, 0.3, -0.1, 3.4];

    let vars = variations.clone();
    let vars2 = variations.clone();

    let mut publisher = px.clone();
    let publish1 = async move {
        let mut idx = 0;
        let mut price = 50000.0;

        loop {
            // sleep for 1 second
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            // random percentage change
            let change_percent = vars[idx];
            price = price + (price * change_percent / 100.0);

            let quote = Quote {
                market: s!("BINANCE"),
                id: s!("BTCUSDT"),
                change_percent,
                market_hours: MarketHourType::Regular,
                price,
                time: 198798798798,
            };

            println!("Publishing quote, {}", quote);

            publisher.publish(quote).await?;

            // keep the index within 0 and length of variations
            idx = (idx + 1) % vars.len();
        }

        #[allow(unreachable_code)]
        Result::<()>::Ok(())
    };

    let publish2 = async move {
        let mut idx = 0;
        let mut price = 300.0;

        loop {
            // sleep for 1 second
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            // random percentage change
            let change_percent = vars2[idx];
            price = price + (price * change_percent / 100.0);

            let quote = Quote {
                market: s!("NASDAQ"),
                id: s!("GOOGL"),
                change_percent,
                market_hours: MarketHourType::Regular,
                price,
                time: 198798798798,
            };

            println!("Publishing quote, {}", quote);

            px.publish(quote).await?;

            // keep the index within 0 and length of vars2
            idx = (idx + 1) % vars2.len();
        }

        #[allow(unreachable_code)]
        Result::<()>::Ok(())
    };

    let _ = tokio::join!(publish1, publish2);

    Ok(())
}
