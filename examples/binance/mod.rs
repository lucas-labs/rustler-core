use {
    async_trait::async_trait,
    eyre::{OptionExt, Result},
    lool::{
        logger::{debug, error, info},
        s,
    },
    rustler_core::{
        rustler,
        rustlers::{svc::quote, MarketHourType, Rustler, RustlerAccessor, RustlerStatus, Ticker},
    },
    std::collections::HashMap,
    tokio::select,
};

rustler!(
    /// A fake rustler that does nothing but changing between different statuses.
    pub struct FooRustler {}
);

#[allow(dead_code)]
impl FooRustler {
    pub fn create() -> impl Rustler {
        Self::default()
    }

    pub fn create_with_external_stuff(name: String) -> impl Fn() -> FooRustler {
        move || {
            println!("Creating a new FooRustler using external name = {}", name);
            Self::default()
        }
    }

    async fn start_rustling(&mut self) -> Result<()> {
        let sender = self.msg_sender().as_ref().ok_or_eyre("Sender not found")?.clone();

        tokio::spawn(async move {
            debug!("Starting rustling");
            select! {
                _ = async move {
                    loop {
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                        let result = sender
                            .send(quote(
                                s!("BTCUSDT"),
                                s!("BINANCE"),
                                50000.0,
                                0.0,
                                198798798798,
                                MarketHourType::Regular,
                            ))
                            .await;

                        if let Err(e) = result {
                            error!("Failed to send message: {}", e);
                        }
                    }
                } => {
                    error!("Rustling stopped");
                },

            }
        });

        Ok(())
    }
}

#[async_trait]
impl Rustler for FooRustler {
    async fn connect(&mut self) -> Result<()> {
        if self.status == RustlerStatus::Connected || self.status == RustlerStatus::Connecting {
            return Ok(());
        }

        self.set_status(RustlerStatus::Connecting)?;

        info!("(mock) Connecting to data source");
        self.start_rustling().await?;
        info!("(mock) Connected to data source");

        self.set_status(RustlerStatus::Connected)?;

        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        if self.status == RustlerStatus::Disconnected || self.status == RustlerStatus::Disconnecting
        {
            return Ok(());
        }

        self.set_status(RustlerStatus::Disconnecting)?;
        info!("(mock) Disconnecting from data source");
        self.set_status(RustlerStatus::Disconnected)?;
        info!("(mock) Disconnected from data source");

        Ok(())
    }

    async fn on_add(&mut self, tickers: &[Ticker]) -> Result<()> {
        info!("(mock) Adding tickers: {:?}", tickers);
        Ok(())
    }

    async fn on_delete(&mut self, tickers: &[Ticker]) -> Result<()> {
        info!("(mock) Deleting tickers: {:?}", tickers);
        Ok(())
    }
}
