pub mod binance;
pub extern crate chrono;
pub extern crate eyre;

use {
    async_trait::async_trait,
    chrono::{DateTime, Local},
    entities::{market, ticker},
    eyre::Result,
    std::collections::HashMap,
};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum RustlerStatus {
    Connecting,
    Connected,
    Disconnecting,
    #[default]
    Disconnected,
}

#[derive(Debug, Clone)]
pub enum MarketHourType {
    Pre,
    Regular,
    Post,
    Extended,
}

#[derive(Debug, Clone)]
pub struct Quote {
    pub id: String,
    pub market: String,
    pub price: f64,
    pub change_percent: Option<f64>,
    pub time: Option<i64>,
    pub market_hours: Option<MarketHourType>,
}

#[derive(Debug, Clone)]
pub struct RustlerOpts {
    pub connect_on_start: bool,
    pub connect_on_add: bool,
}

impl Default for RustlerOpts {
    fn default() -> Self {
        Self {
            connect_on_start: true,
            connect_on_add: true,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ScrapperCallbacks {
    pub on_connected: Option<fn() -> Result<()>>,
    pub on_disconnected: Option<fn() -> Result<()>>,
    pub on_message: Option<fn(message: Quote) -> Result<()>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Ticker {
    pub symbol: String,
    pub market: String,
}

impl Ticker {
    pub fn from(t: &ticker::Model, m: &market::Model) -> Self {
        Self {
            symbol: t.symbol.clone(),
            market: m.short_name.clone(),
        }
    }

    pub fn many_from(tickers: &[ticker::Model], market: &market::Model) -> Vec<Self> {
        tickers.iter().map(|t| Self::from(t, market)).collect()
    }

    pub fn key(&self) -> String {
        format!("{}:{}", self.market, self.symbol)
    }
}

pub trait RustlerAccessor {
    // #region fields g&s
    fn name(&self) -> String;

    fn static_name() -> String
    where
        Self: Sized;

    fn status(&self) -> &RustlerStatus;
    fn set_status(&mut self, status: RustlerStatus) -> Result<()>;

    fn next_run(&self) -> &DateTime<Local>;
    fn set_next_run(&mut self, next_run: DateTime<Local>);

    fn next_stop(&self) -> &Option<DateTime<Local>>;
    fn set_next_stop(&mut self, next_stop: Option<DateTime<Local>>);

    fn last_run(&self) -> &Option<DateTime<Local>>;
    fn set_last_run(&mut self, last_run: Option<DateTime<Local>>);

    fn last_stop(&self) -> &Option<DateTime<Local>>;
    fn set_last_stop(&mut self, last_stop: Option<DateTime<Local>>);

    fn last_update(&self) -> &Option<DateTime<Local>>;
    fn set_last_update(&mut self, last_update: Option<DateTime<Local>>);

    fn opts(&self) -> &RustlerOpts;
    fn set_opts(&mut self, opts: RustlerOpts);

    fn tickers(&self) -> &HashMap<String, Ticker>;
    fn tickers_mut(&mut self) -> &mut HashMap<String, Ticker>;
    fn set_tickers(&mut self, tickers: HashMap<String, Ticker>);

    fn callbacks(&self) -> &Option<ScrapperCallbacks>;
    fn set_callbacks(&mut self, callbacks: Option<ScrapperCallbacks>);
    // #endregion
}

#[async_trait]
pub trait Rustler: RustlerAccessor + Send + Sync {
    // #region Unimplemented trait functions
    /// fn called after tickers are added to the rustler
    fn on_add(&mut self, tickers: &[Ticker]) -> Result<()>;
    /// fn called after tickers are deleted from the rustler
    fn on_delete(&mut self, tickers: &[Ticker]) -> Result<()>;
    /// connects the rustler to the data source
    async fn connect(&mut self) -> Result<()>;
    /// disconnects the rustler from the data source
    async fn disconnect(&mut self) -> Result<()>;
    // #endregion

    /// should be called at construction time
    async fn start(&mut self) -> Result<()> {
        let opts = self.opts();
        if opts.connect_on_start {
            self.connect().await?;
        }
        Ok(())
    }

    /// updates last stop and last run times and calls the appropriate callback
    ///
    /// should be called after the status of the rustler changes
    fn handle_status_change(&mut self) -> Result<()> {
        match self.status() {
            RustlerStatus::Disconnected => {
                self.set_last_stop(Some(Local::now()));

                if let Some(callbacks) = self.callbacks() {
                    if let Some(on_disconnected) = callbacks.on_disconnected {
                        on_disconnected()?;
                    }
                }
            }
            RustlerStatus::Connected => {
                self.set_last_run(Some(Local::now()));

                if let Some(callbacks) = self.callbacks() {
                    if let Some(on_connected) = callbacks.on_connected {
                        on_connected()?;
                    }
                }
            }
            _ => {}
        };

        Ok(())
    }

    /// adds new tickers to the rustler
    async fn add(&mut self, new_tickers: &Vec<Ticker>) -> Result<()> {
        let tickers = self.tickers_mut();

        for new_ticker in new_tickers {
            // if the ticker already exists in the tickers map, skip it
            if tickers.contains_key(&new_ticker.key()) {
                continue;
            }

            tickers.insert(new_ticker.key(), new_ticker.clone());
        }

        if self.opts().connect_on_add {
            // if disconnected, then connect the rustler
            if self.status() == &RustlerStatus::Disconnected {
                self.connect().await?;
            }
        }

        self.on_add(new_tickers)?;
        Ok(())
    }

    /// deletes tickers from the rustler
    async fn delete(&mut self, new_tickers: &Vec<Ticker>) -> Result<()> {
        let tickers = self.tickers_mut();

        for new_ticker in new_tickers {
            tickers.remove(&new_ticker.key());
        }

        // if after deleting the tickers the tickers map is
        // empty, disconnect the rustler
        if tickers.is_empty() {
            self.disconnect().await?;
        }

        self.on_delete(new_tickers)?;
        Ok(())
    }

    /// registers a new quote by passing it to the on_message callback
    fn register_quote(&self, quote: Quote) -> Result<()> {
        if let Some(callbacks) = self.callbacks() {
            if let Some(on_message) = callbacks.on_message {
                on_message(quote)?;
            }
        }

        Ok(())
    }
}

#[macro_export]
macro_rules! rustler_accessors {
    (
        $name:ident
    ) => {
        fn name(&self) -> String {
            stringify!($name).to_string()
        }
        fn static_name() -> String {
            stringify!($name).to_string()
        }
        fn status(&self) -> &$crate::rustlers::RustlerStatus {
            &self.status
        }
        fn set_status(
            &mut self,
            status: $crate::rustlers::RustlerStatus,
        ) -> $crate::rustlers::eyre::Result<()> {
            self.status = status;
            self.handle_status_change()?;

            lool::logger::info!(
                "Rustler {} status changed to {:?}",
                self.name(),
                self.status()
            );

            Ok(())
        }
        fn next_run(&self) -> &$crate::rustlers::chrono::DateTime<$crate::rustlers::chrono::Local> {
            &self.next_run
        }
        fn set_next_run(
            &mut self,
            next_run: $crate::rustlers::chrono::DateTime<$crate::rustlers::chrono::Local>,
        ) {
            self.next_run = next_run;
        }
        fn next_stop(
            &self,
        ) -> &Option<$crate::rustlers::chrono::DateTime<$crate::rustlers::chrono::Local>> {
            &self.next_stop
        }
        fn set_next_stop(
            &mut self,
            next_stop: Option<$crate::rustlers::chrono::DateTime<$crate::rustlers::chrono::Local>>,
        ) {
            self.next_stop = next_stop;
        }
        fn last_run(
            &self,
        ) -> &Option<$crate::rustlers::chrono::DateTime<$crate::rustlers::chrono::Local>> {
            &self.last_run
        }
        fn set_last_run(
            &mut self,
            last_run: Option<$crate::rustlers::chrono::DateTime<$crate::rustlers::chrono::Local>>,
        ) {
            self.last_run = last_run;
        }
        fn last_stop(
            &self,
        ) -> &Option<$crate::rustlers::chrono::DateTime<$crate::rustlers::chrono::Local>> {
            &self.last_stop
        }
        fn set_last_stop(
            &mut self,
            last_stop: Option<$crate::rustlers::chrono::DateTime<$crate::rustlers::chrono::Local>>,
        ) {
            self.last_stop = last_stop;
        }
        fn last_update(
            &self,
        ) -> &Option<$crate::rustlers::chrono::DateTime<$crate::rustlers::chrono::Local>> {
            &self.last_update
        }
        fn set_last_update(
            &mut self,
            last_update: Option<
                $crate::rustlers::chrono::DateTime<$crate::rustlers::chrono::Local>,
            >,
        ) {
            self.last_update = last_update;
        }
        fn opts(&self) -> &$crate::rustlers::RustlerOpts {
            &self.opts
        }
        fn set_opts(&mut self, opts: $crate::rustlers::RustlerOpts) {
            self.opts = opts;
        }
        fn tickers(&self) -> &HashMap<String, $crate::rustlers::Ticker> {
            &self.tickers
        }
        fn tickers_mut(&mut self) -> &mut HashMap<String, $crate::rustlers::Ticker> {
            &mut self.tickers
        }
        fn set_tickers(&mut self, tickers: HashMap<String, $crate::rustlers::Ticker>) {
            self.tickers = tickers;
        }
        fn callbacks(&self) -> &Option<$crate::rustlers::ScrapperCallbacks> {
            &self.callbacks
        }
        fn set_callbacks(&mut self, callbacks: Option<$crate::rustlers::ScrapperCallbacks>) {
            self.callbacks = callbacks;
        }
    };
}

// Define the macro
#[macro_export]
macro_rules! rustler {
    // Entry point for the macro, takes the struct definition
    (
        $(#[$outer:meta])*
        $vis:vis struct $name:ident { $($fields:tt)* }
    ) => {
        // Expand to the struct with derives and the fields
        $(#[$outer])*
        #[derive(Debug, Clone, Default)]
        $vis struct $name {
            status: $crate::rustlers::RustlerStatus,
            next_run: $crate::rustlers::chrono::DateTime<$crate::rustlers::chrono::Local>,
            next_stop: Option<$crate::rustlers::chrono::DateTime<$crate::rustlers::chrono::Local>>,
            last_run: Option<$crate::rustlers::chrono::DateTime<$crate::rustlers::chrono::Local>>,
            last_stop: Option<$crate::rustlers::chrono::DateTime<$crate::rustlers::chrono::Local>>,
            last_update: Option<$crate::rustlers::chrono::DateTime<$crate::rustlers::chrono::Local>>,
            opts: $crate::rustlers::RustlerOpts,
            tickers: HashMap<String, $crate::rustlers::Ticker>,
            callbacks: Option<$crate::rustlers::ScrapperCallbacks>,
            $($fields)*
        }

        // Implement the RustlerAccessor trait for the struct
        impl $crate::rustlers::RustlerAccessor for $name {
            $crate::rustler_accessors!($name);
        }
    };
}
