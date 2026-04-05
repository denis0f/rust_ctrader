use round::round;
use serde::Deserialize;
use crate::utilities::{handle_option_value, handle_timestamp};

use crate::open_api::ProtoOaTrendbarPeriod;

#[derive(Debug, Deserialize)]
pub struct Tokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

#[derive(Debug, PartialEq, Deserialize)]
pub enum Scope {
    Trading,
    Accounts,
}

#[derive(Debug, Deserialize)]
pub struct Account {
    pub id: u64,
    pub broker: String,
    pub is_live: bool,
    pub scope: Scope,
}
pub enum Endpoint {
    Demo,
    Live,
}

#[derive(Debug)]
pub enum StreamEvent {
    ApplicationAuthorized(String),
    AccountAuthorized(String),
    SymbolsData(Vec<Symbol>),
    AccountsData(Vec<Account>),
    TrendbarsData(Vec<BarData>),
    QuotesData(Vec<Quote>),
    LiveData((Option<Quote>, Option<BarData>, Option<BarData>)),
    /// Returned whenever the client has retrieved detailed symbol metadata.
    /// The contained map is a snapshot of the updated entries.  Callers may
    /// also query `CtraderClient::symbol_data` directly instead of using this
    /// event.
    SymbolData(Vec<Symbol>),
    SubscribeSpotsData(String),
    SubscribeLiveBarsData(String),
    ExecutionEvent(String),
    Error(String),
}

#[derive(Debug)]
pub struct Symbol {
    pub symbol_name: String,
    pub symbol_id: u64,
}

/// Information about a symbol returned by the symbol‑by‑id request.  We cache
/// only the fields required for volume conversion so that `Order` handling
/// can look up `max_volume`/`min_volume` etc.  Storing it in a small struct
/// also makes it convenient to keep behind a `HashMap` in the client.
#[derive(Debug, Clone)]
pub struct SymbolData {
    pub symbol_id: u64,
    pub max_volume: Option<i64>,
    pub min_volume: Option<i64>,
    pub step_volume: Option<i64>,
    pub digits: Option<i32>,
    pub pip_position: Option<i32>,
    pub lot_size: Option<i64>,
}


#[derive(Debug)]
pub struct BarData {
    pub open: f64,
    pub close: Option<f64>,
    pub high: f64,
    pub low: f64,
    pub volume: u64,
    pub timestamp: Option<String>,
}

impl Clone for BarData {
    fn clone(&self) -> Self {
        BarData {
            open: self.open,
            close: self.close,
            high: self.high,
            low: self.low,
            volume: self.volume,
            timestamp: self.timestamp.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RelativeBarData{
    pub delta_open: f64,
    pub delta_close: Option<f64>,
    pub delta_high: f64,
    pub low: f64,
    pub volume: u64,
    pub timestamp: u64,

}


impl RelativeBarData {
    pub fn change_to_actual_symbol_price(
        &self,
    ) -> BarData {
        let new_low = round(self.low as f64 / 100_000.0, 4);
        let new_high = round((self.low as f64 + self.delta_high as f64) / 100_000.0, 4);
        let new_close = if let Some(close_val) = handle_option_value(self.delta_close) {
            Some(round((self.low as f64 + close_val as f64) / 100_000.0, 4))
        } else {
            None
        };
        let new_open = round((self.low as f64 + self.delta_open as f64) / 100_000.0, 4);

        BarData {
            open: new_open,
            close: new_close,
            high: new_high,
            low: new_low,
            volume: self.volume,
            timestamp: Some(handle_timestamp(self.timestamp)),
        }
    }
}



#[derive(Debug)]
pub enum TimeFrame {
    M1 = 1,
    M5 = 2,
    M15 = 3,
    M30 = 4,
    H1 = 5,
    H4 = 6,
    D1 = 7,
}

impl TimeFrame {
    pub fn change_proto_trendbar_period(&self) -> ProtoOaTrendbarPeriod {
        match self {
            TimeFrame::M1 => ProtoOaTrendbarPeriod::M1,
            TimeFrame::M5 => ProtoOaTrendbarPeriod::M5,
            TimeFrame::M15 => ProtoOaTrendbarPeriod::M15,
            TimeFrame::M30 => ProtoOaTrendbarPeriod::M30,
            TimeFrame::H1 => ProtoOaTrendbarPeriod::H1,
            TimeFrame::H4 => ProtoOaTrendbarPeriod::H4,
            TimeFrame::D1 => ProtoOaTrendbarPeriod::D1,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Quote {
    pub symbol_id: i64,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub timestamp: u64,
}

impl Clone for Quote {
    fn clone(&self) -> Self {
        Quote {
            symbol_id: self.symbol_id,
            bid: self.bid,
            ask: self.ask,
            timestamp: self.timestamp,
        }
    }
}

impl Quote {
    pub fn change_to_actual_quote_price(&self) -> Self {
        let new_bid = if self.bid.is_some() {
            Some(self.bid.unwrap_or(0.0) as f64 / 100_000.0)
        } else {
            None
        };
        let new_ask = if self.ask.is_some() {
            Some(self.ask.unwrap_or(0.0) as f64 / 100_000.0)
        } else {
            None
        };
        Quote {
            symbol_id: self.symbol_id,
            bid: new_bid,
            ask: new_ask,
            timestamp: self.timestamp,
        }
    }
}

pub enum OrderType {
    Market,
    Limit,
    Stop,
    StopLimit,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TradeSide {
    Buy,
    Sell,
}

pub enum TimeInForce {
    GTC,
    GTD,
    IOC,
    FOK,
    MOO
}

pub struct Order{
    pub account_id: u64,
    pub symbol_id: u64,
    pub order_type: OrderType,
    pub trade_side: TradeSide,
    pub lotsize: f64,
    pub limit_price: Option<f64>,/// this is madatory when the order type is Limit order
    pub stop_price: Option<f64>, /// this is madatory when the order type is Stop_Limit order
    pub time_in_force: Option<TimeInForce>, /// expiration instruction of an oder
    pub expiration_timestamp: Option<i64>,/// this is the epiration time for the GTD orders 
    pub comment: Option<String>,
    pub slippage_in_points: Option<i32>, /// this is the number of the points you want to allow for slippage for the limit/market orders 
    pub label: Option<String>,
    pub client_order_id: Option<String>,
    pub relative_stop_loss: Option<i64>,
    pub relative_take_profit: Option<i64>,
    pub guaranteed_stop_loss: Option<bool>,/// supposed to be set to true for the limited risk accounts
    pub trailing_stop_loss: Option<bool>,
    
}

impl Order {
}
