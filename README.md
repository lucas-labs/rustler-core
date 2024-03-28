<p align="center"><img src=".github/img/rustler.svg" height="256"></p>

<br>
<br>
<br>

<p align="center">
𝐫𝐮𝐬𝐭𝐥𝐞𝐫 is a web scraping service that scrapes several stock market providers for stock pricing data. It is built using the <code>Rust</code> programming language.
</p>

<br>
<br>
<br>

## Framweork

- [Tonic (grpc)](https://docs.rs/tonic/latest/tonic/index.html)
- websockets:
  - [tokio-tungstenite](https://docs.rs/tokio-tungstenite/latest/tokio_tungstenite/)
  - [fastwebsockets](https://crates.io/crates/fastwebsockets)
  - [embedded-websocket](https://crates.io/crates/embedded-websocket) - bajo nivel - small
  - [web-socket](https://crates.io/crates/web-socket) - supuestamente el mas rapido
- Rx Rust:
  - [rxrust](https://crates.io/crates/rxrust)
  - [another-rxrust](https://crates.io/crates/another-rxrust) parece un poco mejor

## Commands

```bash
# add a dependency to a project
cargo add {dependency} -p {project}

# example
cargo add tokio-tungstenite -p gateway
```