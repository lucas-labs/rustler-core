use {
    crate::{
        rustler,
        rustlers::{Rustler, RustlerAccessor, RustlerStatus, Ticker},
    },
    async_trait::async_trait,
    eyre::Result,
    lool::{cli::stylize::Stylize, logger::info},
    std::collections::HashMap,
};

const BINANCE_WSS_URL: &str = "wss://stream.binance.com:9443/stream";
// const MANUAL_CLOSE_CODE: u16 = 4663;
// const MANUAL_CLOSE_REASON: &str = "Manually Disconnected";

rustler!(
    /// 🤠 » **binance rustler**
    ///
    /// A rustler that steals quotes from Binance
    pub struct BinanceRustler {}
);

impl BinanceRustler {
    pub fn create() -> impl Rustler {
        Self::default()
    }
}

#[async_trait]
impl Rustler for BinanceRustler {
    async fn connect(&mut self) -> Result<()> {
        if self.status == RustlerStatus::Connected || self.status == RustlerStatus::Connecting {
            return Ok(());
        }

        self.set_status(RustlerStatus::Connecting)?;

        info!(
            "Connecting to Binance WSS: {}",
            BINANCE_WSS_URL.bright_green()
        );

        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        info!("Disconnecting from Binance WSS");

        Ok(())
    }

    fn on_add(&mut self, tickers: &[Ticker]) -> Result<()> {
        info!("Adding tickers: {:?}", tickers);

        Ok(())
    }

    fn on_delete(&mut self, tickers: &[Ticker]) -> Result<()> {
        info!("Deleting tickers: {:?}", tickers);

        Ok(())
    }
}
