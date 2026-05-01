pub mod open_api {
    tonic::include_proto!("open_api");
}
pub mod auth;
pub mod types;
//pub mod ctrader_;
pub mod ctrader;
pub mod utilities;

pub use auth::AuthClient;
pub use ctrader::CtraderClient;
pub use types::{
    Account, BarData, Endpoint, Quote, Scope, StreamEvent,
    Symbol, SymbolData, TimeFrame, Tokens, Order, RelativeBarData, Signal
};

//pub use ctrader_::CtraderClient;

pub mod strategies;


