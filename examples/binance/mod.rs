use {
    async_trait::async_trait,
    eyre::Result,
    lool::logger::info,
    rustler_core::{
        rustler,
        rustlers::{Rustler, RustlerAccessor, RustlerStatus, Ticker},
    },
    std::collections::HashMap,
};

rustler!(
    /// A fake rustler that does nothing but changing between different statuses.
    pub struct FooRustler {}
);

impl FooRustler {
    pub fn create() -> impl Rustler {
        Self::default()
    }
}

#[async_trait]
impl Rustler for FooRustler {
    async fn connect(&mut self) -> Result<()> {
        if self.status == RustlerStatus::Connected || self.status == RustlerStatus::Connecting {
            return Ok(());
        }

        self.set_status(RustlerStatus::Connecting)?;

        info!("Connecting to data source");

        self.set_status(RustlerStatus::Connected)?;

        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        if self.status == RustlerStatus::Disconnected || self.status == RustlerStatus::Disconnecting
        {
            return Ok(());
        }

        self.set_status(RustlerStatus::Disconnecting)?;

        info!("Disconnecting from data source");

        self.set_status(RustlerStatus::Disconnected)?;

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
