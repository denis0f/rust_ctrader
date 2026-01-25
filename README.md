# rust_ctrader
# rust-ctrader Auth Client

> **Note:** This document covers **only the authentication (OAuth2) part** of the `rust-ctrader` library. It does **not** describe the full cTrader client, streaming, or trading APIs.

The Auth Client provides a clean Rust abstraction over the **cTrader Open API OAuth2 flow**, allowing you to:

* Generate an authorization URL
* Exchange an authorization code for access & refresh tokens
* Refresh expired access tokens

It is async, minimal, and designed to be composed into higher-level cTrader clients.

---

## Requirements

### Rust Toolchain

Install Rust using `rustup`:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Verify installation:

```bash
rustc --version
cargo --version
```

---

### Protobuf Compiler (`protoc`)

Although this document focuses on authentication, the library relies on **cTrader Open API protobuf definitions**, which require `protoc` to be installed on your system.

#### Linux (Ubuntu / Debian)

```bash
sudo apt update
sudo apt install -y protobuf-compiler
```

Verify:

```bash
protoc --version
```

#### macOS (Homebrew)

```bash
brew install protobuf
```

Verify:

```bash
protoc --version
```

---

## Environment Setup

It is recommended to use environment variables (or a `.env` file during development).

```env
client_id=YOUR_CLIENT_ID
client_secret=YOUR_CLIENT_SECRET
redirect_uri=YOUR_REDIRECT_URI
refresh_token=OPTIONAL_REFRESH_TOKEN
```

These values are obtained when you register your application in the **cTrader Open API developer portal**.

---

## AuthClient Overview

The `AuthClient` encapsulates the **OAuth2 Authorization Code Grant** used by cTrader.

```rust
#[derive(Debug)]
pub struct AuthClient {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}
```

### Creating an AuthClient

```rust
use rust_ctrader::AuthClient;

let client = AuthClient::new(
    "your_client_id",
    "your_client_secret",
    "your_redirect_uri",
);
```

---

## OAuth2 Flow (How It Works)

### 1. Select an Authorization Scope

Scopes define what level of access your application is requesting.

```rust
use rust_ctrader::Scope;

let scope = Scope::Trading;   // or Scope::Accounts
```

* `Trading` → Trading-related permissions
* `Accounts` → Account-level access

---

### 2. Generate the Authorization URL

```rust
let authorization_url = client.get_authorization_url(scope);

println!("Visit this URL to authorize the application:\n{}", authorization_url);
```

The user must:

1. Open the URL in a browser
2. Log in to cTrader
3. Grant access to the application

After approval, cTrader redirects the user to:

```text
{redirect_uri}?code=AUTHORIZATION_CODE
```

---

### 3. Exchange Authorization Code for Tokens

Use the returned `code` to request access and refresh tokens.

```rust
let tokens = client.get_access_tokens("AUTHORIZATION_CODE").await?;

println!("Access Token: {}", tokens.access_token);
println!("Refresh Token: {}", tokens.refresh_token);
```

The `Tokens` response typically contains:

* `access_token`
* `refresh_token`
* `expires_in`
* `token_type`

---

### 4. Refresh an Expired Access Token

Access tokens expire. Use the refresh token to obtain a new one without user interaction.

```rust
let refreshed_tokens = client
    .refresh_token(&tokens.refresh_token)
    .await?;

println!("New access token: {}", refreshed_tokens.access_token);
```

---

## Complete Example

```rust
use rust_ctrader::{AuthClient, Scope};
use std::{env, io::stdin};
use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let client_id = env::var("client_id")?;
    let client_secret = env::var("client_secret")?;
    let redirect_uri = env::var("redirect_uri")?;

    let client = AuthClient::new(&client_id, &client_secret, &redirect_uri);

    let scope = Scope::Trading;
    let authorization_url = client.get_authorization_url(scope);

    println!("Visit the URL below to authorize:\n{}", authorization_url);
    println!("Enter the authorization code:");

    let mut code = String::new();
    stdin().read_line(&mut code)?;

    let tokens = client.get_access_tokens(code.trim()).await?;
    println!("Received tokens: {:#?}", tokens);

    let refreshed = client.refresh_token(&tokens.refresh_token).await?;
    println!("Refreshed tokens: {:#?}", refreshed);

    Ok(())
}
```

---

## Notes & Best Practices

* Never commit access or refresh tokens to version control
* Store secrets securely (env vars, vaults, or secret managers)
* Refresh tokens before access token expiration
* Always use HTTPS redirect URIs in production

---

## Summary

The Auth Client is a **small, focused OAuth2 layer** for cTrader Open API:

* No UI assumptions
* No trading logic
* Easy to embed into larger systems

Once authenticated, the returned tokens can be used with other parts of the cTrader Open API client.

---

Happy coding 🚀
