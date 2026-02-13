use round::round;
use serde::Deserialize;
use crate::handle_option_value;

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
    LiveData((Option<Quote>, Option<BarData>)),
    SubscribeSpotsData(String),
    SubscribeLiveBarsData(String),
    Error(String),
}

#[derive(Debug)]
pub struct Symbol {
    pub symbol_name: String,
    pub symbol_id: u64,
}

#[derive(Debug)]
pub struct BarData {
    pub open: f64,
    pub close: Option<f64>,
    pub high: f64,
    pub low: f64,
    pub volume: u64,
    pub timestamp: u64,
}

impl BarData {
    pub fn change_to_actual_symbol_price(
        &self,
        low: u64,
        delta_high: u64,
        delta_close: Option<u64>,
        delta_open: u64,
        volume: u64,
        timestamp: u64,
    ) -> Self {
        let new_low = round(low as f64 / 100_000.0, 4);
        let new_high = round((low as f64 + delta_high as f64) / 100_000.0, 4);
        let new_close = if let Some(close_val) = handle_option_value(delta_close) {
            Some(round((low as f64 + close_val as f64) / 100_000.0, 4))
        } else {
            None
        };
        let new_open = round((low as f64 + delta_open as f64) / 100_000.0, 4);

        BarData {
            open: new_open,
            close: new_close,
            high: new_high,
            low: new_low,
            volume,
            timestamp,
        }
    }
}

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

impl Quote {
    pub fn change_to_actual_quote_price(&self, bid: Option<u64>, ask: Option<u64>) -> Self {
        let new_bid = if bid.is_some() {
            Some(self.bid.unwrap_or(0.0) + bid.unwrap() as f64 / 100_000.0)
        } else {
            None
        };
        let new_ask = if ask.is_some() {
            Some(self.ask.unwrap_or(0.0) + ask.unwrap() as f64 / 100_000.0)
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
pub enum TradeSide {
    Buy,
    Sell,
}

pub enum TimeInForce {
    GTC,
    GTD,
    IOC,
    FOK,
}

pub struct Order{
    pub account_id: u64,
    pub symbol_id: u64,
    pub order_type: OrderType,
    pub trade_side: TradeSide,
    pub volume: u64,
    pub limit_price: Option<f64>,
    pub stop_price: Option<f64>,
    pub time_in_force: Option<TimeInForce>,
    pub expiration_timestamp: Option<i64>,
    pub label: Option<String>,
    pub client_order_id: Option<String>,
    pub relative_stop_loss: Option<i64>,
    pub relative_take_profit: Option<i64>,
    pub guaranteed_stop_loss: Option<bool>,
    pub trailing_stop_loss: Option<bool>,
    
}
