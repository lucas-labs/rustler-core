use {
    super::rustler::Rustler,
    crate::entities::market,
    std::{collections::HashMap, sync::Arc},
    tokio::sync::Mutex,
};

/// **🤠 » rustlerjar! macro**
///
/// A macro to create a `RustlerJar` with multiple Rustler instances and their corresponding
/// mappings.
///
/// **Usage**
///
/// ```rust
/// let rustler_jar = rustlerjar! {
///    "NYSE", "NASDAQ" => FooRustler,
///    "BINANCE" => BarRustler,
/// };
/// ```
#[macro_export]
macro_rules! rustlerjar {
    ($($($name:expr),* => $rustler:ident),* $(,)?) => {{
        use $crate::rustlers::RustlerAccessor;

        let mut instances: Vec<Box<dyn $crate::rustlers::Rustler>> = Vec::new();
        let mut mappings = std::collections::HashMap::new();

        $(
            let instance = Box::new($rustler::create());
            $(
                mappings.insert($name.to_string(), instance.name());
            )*
            instances.push(instance);
        )*

        $crate::rustlers::rustlerjar::RustlerJar::new(instances, mappings)
    }};
}

/// **🤠 » RustlerJar**
///
/// A `RustlerJar` is a collection of Rustlers and their corresponding mappings to the markets.
/// Which indicates which Rustler should be used for a given market. Rustlers are stored as
/// instances of `Box<dyn Rustler>`, and the mappings are stored as a `HashMap<String, String>` (
/// where the key is the market short name and the value is the Rustler name).
///
/// **Usage**
///
/// The easiest way to create a `RustlerJar` is by using the `rustlerjar!` macro.
/// ```rust
/// let rustler_jar = rustlerjar! {
///   "NYSE", "NASDAQ" => FooRustler,
///   "BINANCE" => BarRustler,
/// };
///
/// let rustler = rustler_jar.get(&market);
/// ```
pub struct RustlerJar {
    rustlers: HashMap<String, Arc<Mutex<Box<dyn Rustler>>>>,
    mappings: HashMap<String, String>,
}

impl RustlerJar {
    /// create a new `RustlerJar` with the given Rustlers and mappings.
    ///
    /// **☢️ warn**: using the `rustlerjar!` macro is recommended
    pub fn new(rustlers_list: Vec<Box<dyn Rustler>>, mappings: HashMap<String, String>) -> Self {
        let mut rustlers = HashMap::new();
        for rustler in rustlers_list {
            rustlers.insert(rustler.name(), Arc::new(Mutex::new(rustler)));
        }

        Self { rustlers, mappings }
    }

    /// get the Rustler for the given market
    pub fn get(&self, market: &market::Model) -> Option<&Arc<Mutex<Box<dyn Rustler>>>> {
        let key = self.get_key(market);
        self.rustlers.get(key)
    }

    pub fn get_mut(&mut self, market: &market::Model) -> Option<&mut Arc<Mutex<Box<dyn Rustler>>>> {
        let key = self.get_key(market).to_owned();
        self.rustlers.get_mut(&key)
    }

    /// get the key from the mappings for the given market
    fn get_key(&self, market: &market::Model) -> &str {
        self.mappings.get(&market.short_name).unwrap()
    }
}
