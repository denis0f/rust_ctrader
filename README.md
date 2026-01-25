# rust-ctrader

A Rust library for interacting with the **cTrader Open API**, providing an async, type-safe foundation for building trading systems, analytics tools, and account management applications.

The goal of this project is to expose cTrader functionality in a **clean and idiomatic Rust API**, while handling the low-level details such as OAuth2, TLS, and protobuf messaging.

---

## What It Covers

`rust-ctrader` is designed as a **full client toolkit** for cTrader Open API, including:

* **Authentication (OAuth2)**

  * Authorization URL generation
  * Access & refresh token handling

* **Secure Connectivity**

  * Encrypted TCP/TLS connections
  * Support for **Demo** and **Live** environments

* **Protobuf Messaging**

  * Strongly-typed messages generated from cTrader `.proto` files

* **Account & Market Data**

  * Trading accounts and metadata
  * Symbols and market-related information

* **Trading Operations**

  * Order placement and management
  * Positions and execution updates

* **Real-Time Streaming**

  * Live updates for orders, positions, and balances

---

## Typical Workflow

1. Authenticate using OAuth2
2. Obtain access and refresh tokens
3. Connect securely to cTrader servers over TLS
4. Send and receive protobuf messages
5. Perform data queries or trading actions

---

## Requirements

* Rust (via `rustup`)
* Protobuf compiler (`protoc`)

### Install `protoc`

**Linux (Ubuntu / Debian)**

```bash
sudo apt install -y protobuf-compiler
```

**macOS (Homebrew)**

```bash
brew install protobuf
```

---

## Status

* Authentication: ✅ implemented
* TCP/TLS connectivity: 🚧 in progress
* Trading & data features: 🚧 evolving

---

## Summary

`rust-ctrader` aims to be a **reliable Rust foundation for the cTrader Open API**, suitable for bots, dashboards, and trading infrastructure.

More features are actively being added.
