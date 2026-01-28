pub mod open_api{
    tonic::include_proto!("open_api");
}
pub mod auth;
pub mod types;
//pub mod ctrader_;
pub mod ctrader;

pub use auth::AuthClient;
pub use types::{Scope, Tokens, Account, Endpoint, StreamEvent, Symbol, BarData};
pub use ctrader::CtraderClient;

//pub use ctrader_::CtraderClient;