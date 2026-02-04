# CtraderClient

This README explains how to use the **`CtraderClient`**, which is responsible for connecting to the cTrader Open API over secure TCP/TLS, sending protobuf requests, and handling all streamed responses via events.

This client is used **after you already obtain an access token** through the AuthClient.

---

## 🚀 What the CtraderClient Does

The client acts as the **network + protocol layer** of the library:

* Connects to **Demo** or **Live** cTrader servers via TLS
* Sends protobuf‑encoded OpenAPI messages
* Reads and decodes responses
* Emits structured events via an async channel
* Supports application & account authorization
* Fetching account lists, symbols, and historical trend bars

---

## 📦 Example Usage Overview (Matches the Example Client)

Your typical workflow looks like:

1. Load environment variables (`client_id`, `client_secret`, `access_token`)
2. Connect using `CtraderClient::connect`
3. Start background read loop with `client.start()`
4. Authorize the **application**
5. After receiving `ApplicationAuthorized`, request **accounts**
6. After receiving `AccountsData`, authorize a specific **account**
7. After receiving `AccountAuthorized`, request **symbols**
8. After receiving `SymbolsData`, request **trend bars**
9. Handle returned data from `TrendbarsData`

This mirrors the example client you're shipping with the library.

---

## 🔌 Creating and Connecting a Client

```rust
use rust_ctrader::{CtraderClient, Endpoint};

let (client, mut events) = CtraderClient::connect(
    &client_id,
    &client_secret,
    &access_token,
    Endpoint::Demo,
).await?;
```

* Returns `Arc<CtraderClient>`
* Returns `mpsc::Receiver<StreamEvent>` for receiving events

---

## ▶️ Starting the Background Read Loop

```rust
client.start().await;
```

This must be called or you **will not receive responses**.

---

## 🔐 Authorizing Application & Account

```rust
client.authorize_application().await?;
```

Events will guide the flow:

### On `ApplicationAuthorized`

```rust
client.get_accounts().await?;
```

### On `AccountsData`

```rust
client.authorize_account(account_id).await?;
```

---

## 📄 Requesting Symbols

```rust
client.get_symbols(account_id, false).await?;
```

This emits:

```rust
StreamEvent::SymbolsData(Vec<Symbol>)
```

---

## 📊 Fetching Historical Trend Bars (Candles)

```rust
use rust_ctrader::TimeFrame;

client.get_trend_bar_data(
    symbol_id,
    TimeFrame::D1,
    account_id,
    from_timestamp,
    to_timestamp,
).await?;
```

Trend bars are emitted as:

```rust
StreamEvent::TrendbarsData(Vec<BarData>)
```

---

## 📥 Handling Stream Events (Core of Usage)

```rust
while let Some(event) = events.recv().await {
    match event {
        StreamEvent::ApplicationAuthorized(msg) => { /* request accounts */ }
        StreamEvent::AccountsData(accounts) => { /* authorize account */ }
        StreamEvent::AccountAuthorized(msg) => { /* request symbols */ }
        StreamEvent::SymbolsData(symbols) => { /* request trend bars */ }
        StreamEvent::TrendbarsData(bars) => { /* handle data */ }
        StreamEvent::Error(err) => eprintln!("Error: {}", err),
        _ => {}
    }
}
```

---

## ⚠️ Notes from Example

* `account_id` must be selected from `AccountsData`
* Timestamps are in **milliseconds** (use `chrono` for convenience)
* You can choose any symbol (example uses **XAUUSD**)
* The stream sends multiple event types — always match on `StreamEvent`

---

## 📝 Summary

The `CtraderClient` is:

* The **TCP/TLS engine** for the cTrader Open API
* A **protobuf message sender/receiver**
* An **event‑driven stream client** that powers account, symbol, and data queries

Use the example client provided in the repo to see a full end‑to‑end flow.
