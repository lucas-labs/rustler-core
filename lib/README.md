<p align="center"><img src="https://raw.githubusercontent.com/lucas-labs/rustler-core/master/.github/img/rustler-core-logo.svg" height="264"></p>

<br>
<br>

<p align="center">
𝐫𝐮𝐬𝐭𝐥𝐞𝐫 ⫮ 𝐜𝐨𝐫𝐞 is a library that contains the core functionality for `rustler`, a web scraping service 
that scrapes several stock market providers for stock pricing data. It is built using the
<code>Rust</code> programming language.
</p>

<br>
<br>

## Why "rustler"

A `rustler` is a person who steals live**_stock_**. Well, this library is a service that collects
_stock_ market data from the internet. So, it's a "_rustler_" for stock market data.

Also, this library is built using the `Rust` programming language... so, __rust__-ler 😊

## What this library includes

This library defines the core functionality for a `rustler`. It includes the following:

-   A [`rustlers::Rustler`] trait that defines the core functionality for a `rustler`.
-   A [`rustlers::svc::RustlersSvc`] which orchestrates the `rustlers` at runtime, scheduling them to scrape stock pricing data between market hours.

More info [here](rustlers).

Apart from the above, this library also defines:

-   a [database schema](entities) for storing market hours, which is used by the `RustlersSvc` to schedule the `rustlers`.
-   initial [database migrations](entities/migration) to create the schema.
-   a [grpc server](grpc) to interact with the rustlers database.
-   a [websocket gateway server](socket) to stream stock pricing data to subscribed clients


