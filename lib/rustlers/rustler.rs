pub extern crate chrono;
pub extern crate eyre;

use {
    super::bus::{RedisMessage, ToFromRedisMessage, ToRedisKey, ToRedisVal},
    crate::entities::{market, ticker},
    async_trait::async_trait,
    chrono::{DateTime, Local},
    eyre::Result,
    lool::s,
    std::{
        collections::HashMap,
        fmt::{self, Display, Formatter},
    },
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
    Pre = 0,
    Regular = 1,
    Post = 2,
    Extended = 3,
}

impl From<MarketHourType> for u8 {
    fn from(market_hour_type: MarketHourType) -> Self {
        market_hour_type as u8
    }
}

impl From<u8> for MarketHourType {
    fn from(market_hour_type: u8) -> Self {
        match market_hour_type {
            0 => MarketHourType::Pre,
            1 => MarketHourType::Regular,
            2 => MarketHourType::Post,
            3 => MarketHourType::Extended,
            _ => MarketHourType::Regular,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Quote {
    pub id: String,
    pub market: String,
    pub price: f64,
    pub change_percent: f64,
    pub time: i64,
    pub market_hours: MarketHourType,
}

impl Quote {
    pub fn belongs_to(&self, ticker: &Ticker) -> bool {
        self.id == ticker.symbol && self.market == ticker.market
    }
}

impl Display for Quote {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl ToRedisVal for Quote {
    fn to_redis_val(&self) -> Vec<(String, String)> {
        let market_hours_u8: u8 = self.market_hours.clone().into();

        vec![
            (s!("id"), self.id.to_owned()),
            (s!("market"), self.market.to_owned()),
            (s!("price"), self.price.to_string()),
            (s!("market_hours"), market_hours_u8.to_string()),
            (s!("time"), self.time.to_string()),
            (s!("change_percent"), self.change_percent.to_string()),
        ]
    }
}

impl ToRedisKey for Quote {
    fn to_redis_key(&self) -> String {
        format!("quote:{}:{}", self.market, self.id)
    }
}

impl ToFromRedisMessage for Quote {
    fn as_message(&self) -> String {
        // id¦market¦price¦change_percent¦time¦market_hours
        format!(
            "{}¦{}¦{}¦{}¦{}¦{}",
            self.id,
            self.market,
            self.price,
            self.change_percent,
            self.time,
            Into::<u8>::into(self.market_hours.clone())
        )
    }

    /// 🐎 » creates a `Quote` from a message
    ///
    /// the message should be in the format `id¦market¦price¦change_percent¦time¦market_hours`
    ///
    /// **panics** if the message is not in the correct format
    fn from_message<T: AsRef<str>>(msg: T) -> Self {
        let msg = msg.as_ref();
        let parts: Vec<&str> = msg.split('¦').collect();

        let id = parts[0].to_string();
        let market = parts[1].to_string();
        let price = parts[2].parse::<f64>().unwrap();
        let change_percent = parts[3].parse::<f64>().unwrap();
        let time = parts[4].parse::<i64>().unwrap();
        let market_hours = parts[5].parse::<u8>().unwrap().into();

        Self {
            id,
            market,
            price,
            change_percent,
            time,
            market_hours,
        }
    }
}

impl PartialEq<Ticker> for Quote {
    fn eq(&self, other: &Ticker) -> bool {
        self.id == other.symbol && self.market == other.market
    }
}

impl PartialEq<Quote> for Ticker {
    fn eq(&self, other: &Quote) -> bool {
        self.symbol == other.id && self.market == other.market
    }
}

impl PartialEq<Quote> for Quote {
    fn eq(&self, other: &Quote) -> bool {
        self.id == other.id && self.market == other.market
    }
}

impl RedisMessage for Quote {}

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

/// 🐎 » a scruct representing a ticker
///
/// in `rustler` a ticker is the union between a symbol (stock identifier) and its market
///
/// they `key` of a ticker is the concatenation of the market and the symbol separated by a colon
///
/// e.g. `AAPL` in the `NASDAQ` market would have the key `NASDAQ:AAPL`
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

    /// 🐎 » returns the key of the ticker
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
    /// 🐎 » fn called after tickers are added to the rustler
    fn on_add(&mut self, tickers: &[Ticker]) -> Result<()>;
    /// 🐎 » fn called after tickers are deleted from the rustler
    fn on_delete(&mut self, tickers: &[Ticker]) -> Result<()>;
    /// 🐎 » connects the rustler to the data source
    async fn connect(&mut self) -> Result<()>;
    /// 🐎 » disconnects the rustler from the data source
    async fn disconnect(&mut self) -> Result<()>;
    // #endregion

    /// 🐎 » starts the rustler
    async fn start(&mut self) -> Result<()> {
        let opts = self.opts();
        if opts.connect_on_start {
            self.connect().await?;
        }
        Ok(())
    }

    /// 🐎 » updates last stop and last run times and calls the appropriate callback
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

/// **🐎 » rustler builder macro**
///
/// The `rustler!` macro is used to define a new `Rustler` struct, expanding the struct definition
/// with the required fields and derives, and implementing the `RustlerAccessor` trait for the
/// struct.
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
