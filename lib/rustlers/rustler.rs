pub extern crate chrono;
pub extern crate eyre;

use {
    super::svc::RustlerMsg,
    crate::{
        bus::{redis::stream::StreamMsg, BusMessage, ToBusKey, ToBusVal, ToFromBusMessage},
        entities::{market, ticker},
    },
    async_trait::async_trait,
    chrono::{DateTime, Local},
    eyre::Result,
    lool::s,
    serde::Serialize,
    std::{
        collections::HashMap,
        fmt::{self, Display, Formatter},
    },
    tokio::sync::mpsc::Sender,
};

/// 🐎 » a struct representing the status of a rustler at a given time
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum RustlerStatus {
    Connecting,
    Connected,
    Disconnecting,
    #[default]
    Disconnected,
}

/// 🐎 » an enum representing the different types of market hours
#[derive(Debug, Clone, Serialize)]
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

/// 🐎 » a struct storing a ticker's quote at a given time, and the change in price since the last
/// quote
#[derive(Debug, Clone, Serialize)]
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

impl ToBusVal for Quote {
    fn to_bus_val(&self) -> Vec<(String, String)> {
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

impl ToBusKey for Quote {
    fn to_bus_key(&self) -> String {
        format!("quote:{}:{}", self.market, self.id)
    }
}

impl ToFromBusMessage for Quote {
    /// 🐎 » converts a `Quote` to a serialized message that can be sent over a redis channel
    ///
    /// the message is in the format `id¦market¦price¦change_percent¦time¦market_hours`
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

impl StreamMsg for Quote {}
impl BusMessage for Quote {}

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
    pub quote_asset: Option<String>,
}

impl Ticker {
    pub fn from(t: &ticker::Model, m: &market::Model) -> Self {
        let market = match &m.pub_name {
            Some(market) => market.clone(),
            None => m.short_name.clone(),
        };

        Self {
            symbol: t.symbol.clone(),
            market,
            quote_asset: t.quote_symbol.clone(),
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

    /// 🐎 » returns the name of the rustler
    fn name(&self) -> String;

    /// 🐎 » returns the [`RustlerStatus`] of the rustler
    fn status(&self) -> &RustlerStatus;
    /// 🐎 » sets the [`RustlerStatus`] of the rustler
    fn set_status(&mut self, status: RustlerStatus) -> Result<()>;

    /// 🐎 »  returns `true` if the rustler's [`RustlerStatus`] is [RustlerStatus::Connecting]
    fn is_connecting(&self) -> bool {
        self.status() == &RustlerStatus::Connecting
    }
    /// 🐎 »  returns `true` if the rustler's [`RustlerStatus`] is [RustlerStatus::Connected]
    fn is_connected(&self) -> bool {
        self.status() == &RustlerStatus::Connected
    }
    /// 🐎 »  returns `true` if the rustler's [`RustlerStatus`] is [RustlerStatus::Disconnecting]
    fn is_disconnecting(&self) -> bool {
        self.status() == &RustlerStatus::Disconnecting
    }
    /// 🐎 »  returns `true` if the rustler's [`RustlerStatus`] is [RustlerStatus::Disconnected]
    fn is_disconnected(&self) -> bool {
        self.status() == &RustlerStatus::Disconnected
    }
    /// 🐎 »  returns `true` if the rustler's [`RustlerStatus`] is [RustlerStatus::Connected] or
    /// [RustlerStatus::Connecting]
    fn is_connected_or_connecting(&self) -> bool {
        self.is_connected() || self.is_connecting()
    }
    /// 🐎 »  returns `true` if the rustler's [`RustlerStatus`] is [RustlerStatus::Disconnected] or
    /// [RustlerStatus::Disconnecting]
    fn is_disconnected_or_disconnecting(&self) -> bool {
        self.is_disconnected() || self.is_disconnecting()
    }

    /// 🐎 » returns the next run time of the rustler
    fn next_run(&self) -> &DateTime<Local>;
    /// 🐎 » sets the next run time of the rustler
    fn set_next_run(&mut self, next_run: DateTime<Local>);

    /// 🐎 » returns the next stop time of the rustler
    fn next_stop(&self) -> &Option<DateTime<Local>>;
    //// 🐎 » sets the next stop time of the rustler
    fn set_next_stop(&mut self, next_stop: Option<DateTime<Local>>);

    /// 🐎 » returns the last run time of the rustler
    fn last_run(&self) -> &Option<DateTime<Local>>;
    /// 🐎 » sets the last run time of the rustler
    fn set_last_run(&mut self, last_run: Option<DateTime<Local>>);

    /// 🐎 » returns the last stop time of the rustler
    fn last_stop(&self) -> &Option<DateTime<Local>>;
    /// 🐎 » sets the last stop time of the rustler
    fn set_last_stop(&mut self, last_stop: Option<DateTime<Local>>);

    /// 🐎 » returns the last update time of the rustler
    fn last_update(&self) -> &Option<DateTime<Local>>;
    /// 🐎 » sets the last update time of the rustler
    fn set_last_update(&mut self, last_update: Option<DateTime<Local>>);

    /// 🐎 » returns the options of the rustler
    fn opts(&self) -> &RustlerOpts;
    /// 🐎 » sets the options (see [`RustlerOpts`]) of the rustler
    fn set_opts(&mut self, opts: RustlerOpts);

    /// 🐎 » returns the [`Ticker`]s of the rustler
    fn tickers(&self) -> &HashMap<String, Ticker>;
    /// 🐎 » returns the [`Ticker`]s of the rustler as mutable
    fn tickers_mut(&mut self) -> &mut HashMap<String, Ticker>;
    /// 🐎 » sets the [`Ticker`]s of the rustler
    fn set_tickers(&mut self, tickers: HashMap<String, Ticker>);

    /// 🐎 » returns the message sender of the rustler
    ///
    /// the message sender is used to send messages back to the rustler service; if the message is
    /// a [`RustlerMsg::QuoteMsg`] then the rustler will publish the quote to the bus (redis
    /// probably)
    fn msg_sender(&self) -> &Option<Sender<RustlerMsg>>;
    /// 🐎 » returns the message sender of the rustler as mutable
    fn msg_sender_mut(&mut self) -> &mut Option<Sender<RustlerMsg>>;
    /// 🐎 » sets the message sender of the rustler
    fn set_msg_sender(&mut self, sender: Option<Sender<RustlerMsg>>);
    // #endregion
}

#[async_trait]
pub trait Rustler: RustlerAccessor + Send + Sync {
    // #region Unimplemented trait functions
    /// 🐎 » fn called after tickers are added to the rustler
    ///
    /// After calling this function the rustler should start broadcasting quotes for the added
    /// tickers.
    async fn on_add(&mut self, tickers: &[Ticker]) -> Result<()>;
    /// 🐎 » fn called after tickers are deleted from the rustler
    ///
    /// After calling this function the tickers should be removed from the rustler and it should
    /// stop broadcasting quotes for the deleted tickers.
    async fn on_delete(&mut self, tickers: &[Ticker]) -> Result<()>;
    /// 🐎 » connects the rustler to the data source
    ///
    /// The implementation should take care of setting up any resources, open connections, etc.
    /// after calling this function the rustler should be in a connected state, and the `status`
    /// should be `RustlerStatus::Connected`.
    ///
    /// Being in a connected state not necessarily means that the rustler has started rustling, it
    /// just means that it is connected to the data source and ready to start rustling. Although
    /// the implementation can start rustling after connecting if needed. In most cases, the
    /// rustler should only start rustling after `on_add` is called and the rustler has something to
    /// rustle.
    async fn connect(&mut self) -> Result<()>;
    /// 🐎 » disconnects the rustler from the data source
    ///
    /// The implementation should take care of cleaning up any resources, close
    /// connections, etc.
    ///
    /// After calling this function the rustler should be in a
    /// disconnected state, and the `status` should be `RustlerStatus::Disconnected`.
    ///
    /// Being in a disconnected state means that the rustler is not connected to the data source and
    /// is not rustling or broadcasting any quotes.
    ///
    /// After calling this function it is assumed that the rustler:
    ///   - is not rustling
    ///   - is not connected to the data source
    ///   - has freed up any resources and is ready to be dropped if necessary
    ///   - can connect to the data source again if needed, by calling the `connect` function
    ///
    /// This function will be called atomatically when the rustler does not have any tickers
    /// anymore (after calling `on_delete` and the tickers map is empty)
    async fn disconnect(&mut self) -> Result<()>;
    // #endregion

    /// 🐎 » starts the rustler
    async fn start(&mut self) -> Result<()> {
        let opts = self.opts();
        if opts.connect_on_start && !self.is_connected_or_connecting() {
            self.connect().await?;
        }
        Ok(())
    }

    /// 🐎 » updates last stop and last run times and calls the appropriate callback
    ///
    /// should be called after the status of the rustler changes
    fn handle_status_change(&mut self) -> Result<()> {
        match self.status() {
            RustlerStatus::Disconnected => self.set_last_stop(Some(Local::now())),
            RustlerStatus::Connected => self.set_last_run(Some(Local::now())),
            _ => {}
        };

        Ok(())
    }

    /// 🐎 » adds tickers to the rustler
    ///
    /// Will call the [`Rustler::on_add`] function, so that the implementation can decide what to do after
    /// adding the tickers (e.g. sending a message to a websocket to start listening for quotes,
    /// send an http request, etc.).
    ///
    /// Depending on the `connect_on_add` option in the rustler's options, the rustler will
    /// call [`Rustler::connect`] if it is disconnected before calling [`Rustler::on_add`].
    async fn add(&mut self, new_tickers: &Vec<Ticker>) -> Result<()> {
        let tickers = self.tickers_mut();
        let mut added_tickers = vec![];

        for new_ticker in new_tickers {
            // if the ticker already exists in the tickers map, skip it
            if tickers.contains_key(&new_ticker.key()) {
                continue;
            }

            tickers.insert(new_ticker.key(), new_ticker.clone());
            added_tickers.push(new_ticker.clone());
        }

        if self.opts().connect_on_add {
            // if disconnected, then connect the rustler
            if !self.is_connected_or_connecting() {
                self.connect().await?;
            }
        }

        if !added_tickers.is_empty() {
            self.on_add(&added_tickers).await?;
        }

        Ok(())
    }

    /// 🐎 » deletes tickers from the rustler
    ///
    /// Will call the [`Rustler::on_delete`] function, so that the implementation can decide what to
    /// do after deleting the tickers (e.g. sending a message to a websocket to stop listening for
    /// quotes, etc.).
    ///
    /// If after deleting the tickers the tickers map is empty, the rustler will call
    /// [`Rustler::disconnect`] to disconnect the rustler from the data source.
    async fn delete(&mut self, new_tickers: &Vec<Ticker>) -> Result<()> {
        let tickers = self.tickers_mut();
        let mut removed_tickers = vec![];

        for new_ticker in new_tickers {
            let removed_ticker = tickers.remove(&new_ticker.key());
            if let Some(removed_ticker) = removed_ticker {
                removed_tickers.push(removed_ticker);
            }
        }

        // if after deleting the tickers the tickers map is
        // empty, disconnect the rustler
        if tickers.is_empty() && !self.is_disconnected_or_disconnecting() {
            self.disconnect().await?;
        }

        if !removed_tickers.is_empty() {
            self.on_delete(&removed_tickers).await?;
        }

        Ok(())
    }
}

/// macro that expands to the accessor functions for a `Rustler` struct
///
/// __intended for internal use only__
#[macro_export]
macro_rules! rustler_accessors {
    (
        $name:ident
    ) => {
        fn name(&self) -> String {
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
        // TODO: Instead of next_run and next_stop, store the scheduling rules
        //       we can calculate the next run and next stop times from the rules, and will also be
        //       useful to decide if we should recover from a disconnection or not (we should only
        //       recover if the rules say we should be connected at the current time, otherwise we
        //       should stay disconnected, even if it was an abnormal disconnection)
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
        fn msg_sender(
            &self,
        ) -> &Option<tokio::sync::mpsc::Sender<$crate::rustlers::svc::RustlerMsg>> {
            &self.msg_sender
        }
        fn msg_sender_mut(
            &mut self,
        ) -> &mut Option<tokio::sync::mpsc::Sender<$crate::rustlers::svc::RustlerMsg>> {
            &mut self.msg_sender
        }
        fn set_msg_sender(
            &mut self,
            sender: Option<tokio::sync::mpsc::Sender<$crate::rustlers::svc::RustlerMsg>>,
        ) {
            self.msg_sender = sender;
        }
    };
}

/// #### 🐎 » rustler builder macro
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
        #[derive(Default)]
        $vis struct $name {
            status: $crate::rustlers::RustlerStatus,
            next_run: $crate::rustlers::chrono::DateTime<$crate::rustlers::chrono::Local>,
            next_stop: Option<$crate::rustlers::chrono::DateTime<$crate::rustlers::chrono::Local>>,
            last_run: Option<$crate::rustlers::chrono::DateTime<$crate::rustlers::chrono::Local>>,
            last_stop: Option<$crate::rustlers::chrono::DateTime<$crate::rustlers::chrono::Local>>,
            last_update: Option<$crate::rustlers::chrono::DateTime<$crate::rustlers::chrono::Local>>,
            opts: $crate::rustlers::RustlerOpts,
            tickers: HashMap<String, $crate::rustlers::Ticker>,
            msg_sender: Option<tokio::sync::mpsc::Sender<$crate::rustlers::svc::RustlerMsg>>,
            $($fields)*
        }

        // Implement the RustlerAccessor trait for the struct
        impl $crate::rustlers::RustlerAccessor for $name {
            $crate::rustler_accessors!($name);
        }
    };
}
