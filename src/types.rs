use serde::Deserialize;
use round::round;

use crate::open_api::ProtoOaTrendbarPeriod;

#[derive(Debug, Deserialize)]
pub struct Tokens{
    pub access_token: String,
    pub refresh_token: String, 
    pub expires_in: i64
}

#[derive(Debug, PartialEq, Deserialize)]
pub enum Scope{
    Trading,
    Accounts
}

#[derive(Debug, Deserialize)]
pub struct Account{
    pub id: u64,
    pub broker: String,
    pub is_live: bool,
    pub scope: Scope
}   
pub enum Endpoint{
    Demo,
    Live
}

#[derive(Debug)]
pub enum StreamEvent{
    ApplicationAuthorized(String),
    AccountAuthorized(String),
    SymbolsData(Vec<Symbol>),
    Error(String),
    AccountsData(Vec<Account>),
    TrendbarsData(Vec<BarData>)
}

#[derive(Debug)]
pub struct Symbol{
    pub symbol_name: String,
    pub symbol_id: u64
}

#[derive(Debug)]
pub struct BarData{
    pub open: f64,
    pub close: f64,
    pub high: f64,
    pub low: f64,
    pub volume: u64,
    pub timestamp: u64
}

impl BarData{
    pub fn change_to_actual_symbol_price(&self, low: u64, delta_high: u64, delta_close: u64, delta_open: u64, volume: u64, timestamp: u64) -> Self{
        let new_low = round(low as f64 / 100_000.0, 2);
        let high = low as f64 + delta_high as f64;
        let new_high = round(high / 100_000.0, 2);
        let close = low as f64 + delta_close as f64;
        let new_close = round(close / 100_000.0, 2);
        let open = low as f64 + delta_open as f64;
        let new_open = round(open / 100_000.0, 2);

        BarData{
            open: new_open,
            close: new_close,
            high: new_high,
            low: new_low,
            volume,
            timestamp
        }

    }
}

pub enum TimeFrame{
    M1 = 1,
    M5 = 2,
    M15 = 3,
    M30 = 4,
    H1 = 5,
    H4 = 6,
    D1 = 7
}

impl TimeFrame{
    pub fn change_proto_trendbar_period(&self) -> ProtoOaTrendbarPeriod{
        match self{
            TimeFrame::M1 => ProtoOaTrendbarPeriod::M1,
            TimeFrame::M5 => ProtoOaTrendbarPeriod::M5,
            TimeFrame::M15 => ProtoOaTrendbarPeriod::M15,
            TimeFrame::M30 => ProtoOaTrendbarPeriod::M30,
            TimeFrame::H1 => ProtoOaTrendbarPeriod::H1,
            TimeFrame::H4 => ProtoOaTrendbarPeriod::H4,
            TimeFrame::D1 => ProtoOaTrendbarPeriod::D1 
        }
    }
}

