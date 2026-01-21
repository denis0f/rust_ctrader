pub mod open_api{
    tonic::include_proto!("open_api");
}
pub mod auth;
pub mod types;
pub use auth::AuthClient;
pub use types::{Scope, Tokens};