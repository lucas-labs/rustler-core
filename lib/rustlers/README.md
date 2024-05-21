<h1 align="center"><img src="../../.github/img/doc-title-rustler.svg" height="264"></h1>

<br>
<br>

## `rustler.rs`

Contains the `rustler!` macro, which is used to define a `Rustler`.

`Rustler` is a trait that extends the `RustlerAccessor` trait.

Together, they define the interface and the common functionalities for all Rustlers.

### The `RustlerAccessor` trait

The `RustlerAccessor` trait defines the interface for accessing the Rustler's data (`getters` and `setters` that all Rustlers must implement).

Some of the expected accessors are:
- `status` and `set_status`
- `tickers` and `set_tickers`
- `msg_sender` and `set_msg_sender`

### The `Rustler` trait

The `Rustler` trait extends the `RustlerAccessor` trait and defines the interface for common Rustler's functionalities:

- `start` the Rustler, calling abstract `connect` if the Rustler is set to connect on start
- status change handling
- `add` new tickers to the Rustler (calling `on_add` at the end if tickers were added). Also calls `connect` if the Rustler is set to connect on add and the Rustler is not already connected
- `delete` tickers from the Rustler (calling `on_delete` at the end if tickers were deleted). Also calls `disconnect` if there are no more tickers in the Rustler.

The `Rustler` trait also defines the following abstract methods that must be implemented by each `Rustler` implementation.

- `connect` method that connects the Rustler to the data source
- `disconnect` method that disconnects the Rustler from the data source
- `on_add` method that is called when new tickers are added to the Rustler. This method is called when new tickers are added to the Rustler and must implement the logic to start tracking and rustling the new tickers.
- `on_delete` method that is called when tickers are deleted from the Rustler. This method is called when tickers are deleted from the Rustler and must implement the logic to stop tracking and rustling the deleted tickers.

### The `rustler!` macro

The `rustler!` macro is used to define a `Rustler` and to automatically implement the `RustlerAccessor` trait. This adds the necessary fields and accessors to the struct.

**Example:**

```rust
rustler! {
    pub struct MyRustler { }
}
```

Now we have a `MyRustler` struct that implements the `RustlerAccessor`
trait and has all the necessary fields and accessors :)

## `rustlerjar.rs`

This files defines the `RustlerJar` struct. 

A `RustlerJar` is a collection of `Rustler`s and their corresponding mappings to markets. Such mapping indicates which Rustler should be used for a given market.

It provides methods to retrieve Rustlers by `Market`.

### The `rustlerjar!` macro

The `rustlerjar!` macro is used to create an instance of a `RustlerJar` on an easy way.

**Example:**

```rust
let rustler_jar = rustlerjar! {
  "NYSE", "NASDAQ" => MarketRustler::create,
  "BINANCE" => BinanceRustler::create(url),
};

let rustler = rustler_jar.get(&market);
```

the `rustlerjar!` expects a mapping of **market names** pointing to **`Rustler` creation functions (constructors)** and will return a `RustlerJar` instance.

Note: the `rustlerjar!` macro executes the `create` function for each `Rustler`, so in the example above, we assume that `BinaceRustler::create(url)` returns a function that creates a new instance of `BinanceRustler` and not an instance of `BinanceRustler`.

## `svc.rs`

Contains the `RustlersSvc` struct.

The `RustlersSvc` struct is a service that manages the execution of several `Rustler`s from a `RustlerJar`.

It is responsible for starting and stopping the Rustlers on the right schedule.

It contains a `MarketService`, which connects to the database and is used to retrieve the markets (including their schedules) and their tickets. Then, for each market, it retrieves the corresponding Rustler from the `RustlerJar`, adds the tickers to the it, and starts it.

> [!NOTE]
>
>  <img alt="unimplemented" src="./../../.github/img/todo.svg" height="12">
> 
> Although it's not yet implemented, the `RustlersSvc` will also be responsible for adding and deleting tickers and rustlers at runtime.