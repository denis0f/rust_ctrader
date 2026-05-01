// this is the moving average strategy implementation using EMAs

use std::{collections::VecDeque, sync::Arc};
use crate::{CtraderClient, Order, types::OrderType, types::{Signal, Position, TradeSide}};

//#[derive(Debug, Clone, Copy, PartialEq, Eq)]



pub struct Ema {
    period: usize,
    multiplier: f64,
    buffer: VecDeque<f64>,
    last_ema: Option<f64>,
}

impl Ema {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            multiplier: 2.0 / (period as f64 + 1.0),
            buffer: VecDeque::with_capacity(period),
            last_ema: None,
        }
    }

    pub fn update(&mut self, close: f64) -> Option<f64> {
        if let Some(prev_ema) = self.last_ema {
            let ema = (close - prev_ema) * self.multiplier + prev_ema;
            self.last_ema = Some(ema);
            return Some(ema);
        }

        self.buffer.push_back(close);

        if self.buffer.len() < self.period {
            return None;
        }

        let sma = self.buffer.iter().sum::<f64>() / self.period as f64;
        self.last_ema = Some(sma);

        Some(sma)
    }
}


pub fn get_signal(fast_ema: Option<f64>, slow_ema: Option<f64>, slow_prev_ema: &mut Option<f64>, fast_prev_ema: &mut Option<f64>) -> Signal{
    let buying = fast_ema > slow_ema && fast_prev_ema < slow_prev_ema;
    let selling = fast_ema < slow_ema && fast_prev_ema > slow_prev_ema;

    *slow_prev_ema = slow_ema;
    *fast_prev_ema = fast_ema;

    if buying{
        return Signal::Buy
        
    }else if selling{
        return Signal::Sell
    }
    
    Signal::Hold
}



pub async fn take_a_trade(client: &mut Arc<CtraderClient>, account_id: i64, signal: Signal, in_position: &mut bool, prev_signal: &mut Signal, positions: &mut Vec<Position>) -> Result<(), Box<dyn std::error::Error>> {
    let symbol_id = 41;
    let lot_size = 0.05;
    let comment = String::from("Executed by the crossover_bot.");

    let trade_side = match signal {
        Signal::Buy => TradeSide::Buy,
        Signal::Sell => TradeSide::Sell,
        Signal::Hold => {
            println!("Signal is Hold; no trade action taken.");
            return Ok(());
        }
    };

    let signal_changed = matches!(
        (&*prev_signal, &signal),
        (Signal::Buy, Signal::Sell)
            | (Signal::Sell, Signal::Buy)
            | (Signal::Hold, Signal::Buy)
            | (Signal::Hold, Signal::Sell)
    );

    if *in_position && signal_changed {
        println!("Signal changed from {:?} to {:?}; closing existing position(s) first.", prev_signal, signal);
        for position in positions.clone().iter() {
            client.close_position(position.account_id.unwrap(), position.id, position.volume).await?;
        }
        *in_position = false;
        *positions = Vec::new();
        println!("Existing position(s) closed due to signal change.");
    }

    if !*in_position {
        let order = Order {
            account_id: account_id as u64,
            symbol_id,
            order_type: OrderType::Market,
            trade_side: trade_side.clone(),
            lotsize: lot_size,
            limit_price: None,
            stop_price: None,
            time_in_force: None,
            expiration_timestamp: None,
            comment: Some(comment),
            slippage_in_points: None,
            label: None,
            client_order_id: None,
            relative_stop_loss: None,
            relative_take_profit: None,
            guaranteed_stop_loss: None,
            trailing_stop_loss: None,
        };

        println!("Placing order: {:?} {:?} lots for symbol {}", trade_side, lot_size, symbol_id);
        client.new_order(order).await?;
        *in_position = true;
        *prev_signal = signal;
    } else {
        println!("Already in position and signal is unchanged: {:?}.", signal);
    }

    Ok(())
}


