pub mod open_api {
    tonic::include_proto!("open_api");
}
pub mod auth;
pub mod types;
//pub mod ctrader_;
pub mod ctrader;

pub use auth::AuthClient;
pub use ctrader::CtraderClient;
pub use types::{
    Account, BarData, Endpoint, Quote, Scope, StreamEvent,
    Symbol, TimeFrame, Tokens, Order
};

//pub use ctrader_::CtraderClient;


pub fn handle_option_value<T>(option: Option<T>) -> Option<T> {
    match option {
        Some(value) => Some(value),
        None => None
    }
}