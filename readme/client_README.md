# CtraderClient

This document describes the **`CtraderClient`**, the core networking client responsible for connecting to cTrader Open API servers, sending protobuf messages, and handling incoming responses and events.

This client sits **after authentication** and uses an already-issued access token to communicate with cTrader over **secure TCP/TLS**.

---

## What the CtraderClient Does

The `CtraderClient` is responsible for:

* Establishing **TLS-encrypted TCP connections** to cTrader servers
* Supporting **Demo** and **Live** environments
* Sending protobuf-encoded Open API requests
* Reading and decoding protobuf responses
* Dispatching parsed data via async event channels

It acts as the **transport + protocol layer** of the library.

---

## Key Responsibilities

### Secure Connection

* Connects to `demo.ctraderapi.com` or `live.ctraderapi.com`
* Uses `tokio`, `tokio-rustls`, and `TcpStream`
* Manages a shared TLS stream safely using `Arc<Mutex<>>`

---

### Message Sending

All requests are sent as:

1. Message length (`u32`)
2. Protobuf-encoded `ProtoMessage`

```text
[length][protobuf payload]
```

This matches the cTrader Open API binary protocol.

---

### Message Reading & Dispatching

* Continuously reads messages from the socket
* Decodes them into `ProtoMessage`
* Routes them to the appropriate handler
* Emits structured events via an async channel

---

## Supported Operations

The current client implementation supports:

* Application authorization
* Account listing by access token
* Account authorization
* Symbol list retrieval
* Historical trend bar (candlestick) data
* Error handling and propagation

More trading and streaming features can be layered on top of this foundation.

---

## Creating a Client

```rust
use rust_ctrader::{CtraderClient, Endpoint};

let (client, mut events) = CtraderClient::connect(
    Endpoint::Demo,
    &access_token,
    &client_id,
    &client_secret,
).await?;
```

* Returns a shared `Arc<CtraderClient>`
* Returns an `mpsc::Receiver<StreamEvent>` for async events

---

## Starting the Read Loop

The read loop must be started to receive responses from the server.

```rust
client.start().await;
```

This spawns a background task that continuously listens for incoming messages.

---

## Example: Authorize Application & Account

```rust
// authorize the application
client.authorize_application().await?;

// request accounts
client.get_accounts().await?;

// later, authorize a specific account
client.authorize_account(account_id).await?;
```

Responses are received asynchronously through `StreamEvent`.

---

## Example: Fetch Symbols

```rust
client.get_symbols(account_id, false).await?;
```

This sends a symbols list request and emits `StreamEvent::SymbolsData` once received.

---

## Example: Fetch Historical Trend Bars

```rust
use rust_ctrader::open_api::ProtoOaTrendbarPeriod;

client.get_trend_bar_data(
    symbol_id,
    ProtoOaTrendbarPeriod::D1,
    account_id,
    from_timestamp,
    to_timestamp,
).await?;
```

Trend bar data is returned asynchronously via:

```rust
StreamEvent::TrendbarsData(Vec<BarData>)
```

---

## Handling Events

All decoded responses are pushed into an async channel:

```rust
while let Some(event) = events.recv().await {
    match event {
        StreamEvent::AccountsData(accounts) => { /* handle accounts */ }
        StreamEvent::SymbolsData(symbols) => { /* handle symbols */ }
        StreamEvent::TrendbarsData(bars) => { /* handle bars */ }
        StreamEvent::Error(err) => eprintln!("Error: {}", err),
        _ => {}
    }
}
```

---

## Error Handling

* Server-side errors are decoded from `ProtoOaErrorRes`
* Errors are forwarded as `StreamEvent::Error`
* Connection or decode failures are surfaced immediately

---

## Design Notes

* Async-first (`tokio`)
* Thread-safe shared stream access
* Minimal abstraction over the Open API protocol
* Easy to extend with new protobuf request/response types

---

## Summary

`CtraderClient` is the **core engine** of the library:

* Handles secure communication
* Encodes and decodes Open API messages
* Emits strongly-typed events

It provides a solid foundation for building trading systems, bots, and analytics on top of cTrader Open API.
