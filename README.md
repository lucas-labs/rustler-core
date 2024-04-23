<p align="center"><img src=".github/img/rustler-core-logo.svg" height="264"></p>

<br>
<br>
<br>

<p align="center">
𝐫𝐮𝐬𝐭𝐥𝐞𝐫 ⫮ 𝐜𝐨𝐫𝐞 is a library that contains the core functionality for `rustler`, a web scraping service that scrapes several stock market providers for stock pricing data. It is built using the <code>Rust</code> programming language.
</p>

<br>
<br>
<br>

## Why "rustler"

A `rustler` is a person who steals live***stock***. Well, this library is a service that collects _stock_ market data from the internet. So, it's a "_rustler_" for stock market data.

Also, this library is built using the `Rust` programming language... so, ***rust***ler 😊

## What this library includes

This library defines the core functionality for a `rustler`. It includes the following:

- A [`Rustler`](./lib/rustlers/rustlers.rs) trait that defines the core functionality for a `rustler`.
- A [`RustlersSvc`](./lib/rustlers/svc.rs) which orchestrates the `rustlers` at runtime, scheduling them to scrape stock pricing data between market hours.

Apart from the above, this library also defines:

- a [database schema](./lib/entities/orm/) for storing market hours, which is used by the `RustlersSvc` to schedule the `rustlers`.
- initial [database migrations](./lib/entities/migration) to create the schema.
- a [grpc server](./lib/grpc/) to interact with the rustlers database.
- <img alt="unimplemented" src="./.github/img/todo.svg" height="12"> a [websocket gateway server](./lib/socket/) to stream stock pricing data to subscribed clients 

> [!NOTE]
> 
> This library defines a _rustler_ as a service that scrapes stock pricing data for a
> particular market.
> 
> Although this library contains the core and abstract functionality for the rustlers, it doesn't include any concrete implementation for them.
> 
> Actual concrete implementations for each market cannot be published for many reasons.

## Example

Check the [examples](./examples) directory for an example of how to use this library.
