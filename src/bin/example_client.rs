// This is an example client that demonstrates how to use the CtraderClient 
//to connect to the cTrader Open API, authorize an application and account, 
// Example usage:
// 1. Add a .env file with the following variables: client_id, client_secret, access_token.
// 2. Choose the API endpoint when connecting (Endpoint::Demo or Endpoint::Live).
// 3. Call CtraderClient::connect(client_id, client_secret, access_token, endpoint).await
//    which returns (client, event_rx).
// 4. Start the client's background tasks with client.start().await.
// 5. Authorize the application with client.authorize_application().await.
// 6. The library emits StreamEvent events on event_rx. Typical flow:
//    - StreamEvent::ApplicationAuthorized: call client.get_accounts().await to request accounts.
//    - StreamEvent::AccountsData: select an account ID and call client.authorize_account(account_id).await.
//    - StreamEvent::AccountAuthorized: you can request symbols (client.get_symbols(account_id, false).await) or other data.
//    - StreamEvent::SymbolsData: find the desired symbol and call client.get_trend_bar_data(symbol_id, TimeFrame::..., account_id, from_ms, to_ms).await.
//    - StreamEvent::TrendbarsData: handle received trend bars.
//    - StreamEvent::Error: handle error messages.
// 7. Notes:
//    - account_id must be replaced with a valid account ID returned in AccountsData (the example stores it from accounts[1].id).
//    - Timestamps for get_trend_bar_data are in milliseconds since UNIX epoch.
//    - Use the chrono crate (Utc, Duration) to compute time ranges (as shown below).
            /// for the date range I am yet to work on the easiest way 
            /// to get the timestamp from date string a user will enter not 
            /// just entering the timestamp directly. I would like to make it 
            /// user friendly by allowing date strings like (2025, 12, 15).
//    - The client emits other events; match on StreamEvent to handle them as needed.
// 8. Run the binary with `cargo run --bin example_client` after populating .env.

use dotenv::dotenv;
use rust_ctrader::{CtraderClient, Endpoint, StreamEvent, TimeFrame, types::Order, types::TradeSide, types::OrderType};
use std::env;
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Signal {
    Buy,
    Sell,
    Hold,
}

struct Ema {
    period: usize,
    multiplier: f64,
    buffer: VecDeque<f64>,
    last_ema: Option<f64>,
}

impl Ema {
    fn new(period: usize) -> Self {
        Self {
            period,
            multiplier: 2.0 / (period as f64 + 1.0),
            buffer: VecDeque::with_capacity(period),
            last_ema: None,
        }
    }

    fn update(&mut self, close: f64) -> Option<f64> {
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

fn get_crossover_signal(ema_short: Option<f64>, ema_long: Option<f64>) -> Signal {
    match (ema_short, ema_long) {
        (Some(short), Some(long)) => {
            if short > long {
                Signal::Buy
            } else if short < long {
                Signal::Sell
            } else {
                Signal::Hold
            }
        }
        _ => Signal::Hold,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    dotenv().ok();

    let client_id = env::var("client_id")?;
    let client_secret = env::var("client_secret")?;
    let access_token = env::var("access_token")?;

    let (client, mut event_rx) = CtraderClient::connect(&client_id, &client_secret, &access_token, Endpoint::Demo).await?;

    client.start().await;

    client.authorize_application().await?;

    let mut account_id = 0_i64;
    let mut ema5 = Ema::new(5);
    let mut ema8 = Ema::new(8);
    let mut has_position = false;

    //listen for events
    while let Some(event) = event_rx.recv().await{
        match event{
            StreamEvent::ApplicationAuthorized(msg) => {
                println!("Application authorized event received: {}", msg);
                client.get_accounts().await?;
            }
            StreamEvent::AccountsData(accounts) => {
                println!("Accounts data received: {:#?}", accounts);
                account_id = accounts[1].id as i64; //select the second account for authorization
                client.authorize_account(accounts[1].id as i64).await?; 
            }
            StreamEvent::AccountAuthorized(msg) => {
                println!("Account authorized event received: {}", msg);
                //Further actions can be taken here after account authorization
                // if account_id != 0_i64{
                //     client.get_symbols(account_id, false).await?;
                // }           
                //requesting trend bar data for the last week for the selected symbol client.get_trend_bar_data(symbols[4].symbol_id as i64, TimeFrame::M1, account_id, from_timestamp, to_timestamp).await?;
                
                client.subscribe_live_bars(account_id, 41 as i64, TimeFrame::M1).await?;

            }
            StreamEvent::SymbolsData(symbols) => {
                println!("Symbols data received: {:#?}", symbols);
            


                //prepare the timestamp from last monday to today in milliseconds using the chrono crate
                use chrono::{Utc, Datelike, Duration};
                let now = Utc::now();
                let last_monday = now - Duration::days((now.weekday().num_days_from_monday() + 7) as i64);
                let from_timestamp = last_monday.timestamp_millis();
                let to_timestamp = now.timestamp_millis();

                //client.get_symbol_by_id(account_id, symbols[0].symbol_id as i64).await?;


                //calling the subscribe function
                //client.subscribe_live_bars(account_id, symbols[4].symbol_id as i64, TimeFrame::M1).await?;
                //client.new_order().await?; //requesting trend bar data for the last week for the selected symbol client.get_trend_bar_data(symbols[4].symbol_id as i64, TimeFrame::M1, account_id, from_timestamp, to_timestamp).await?;
            }
            StreamEvent::TrendbarsData(trendbars) => {
                println!("Trend bars data received: {:#?}", trendbars);
                // Further actions can be taken here after receiving trend bars data

            }
            
            StreamEvent::LiveData((quote, bar_data, last_closed_candle)) => {
                //println!("Live data received: Quote: {:#?}, \n BarData: {:#?}, last candle to close is {:?}", quote, bar_data, last_closed_candle);


                // Extract close price from last closed candle and update EMAs
                if let Some(candle) = last_closed_candle {
                    println!("Processing last closed candle: {:#?}", candle);
                    if let Some(close_price) = candle.close {
                        let ema5_val = ema5.update(close_price);
                        let ema8_val = ema8.update(close_price);

                        println!("EMA5: {:?}, EMA8: {:?}", ema5_val, ema8_val);

                        // Check for crossover signal when both EMAs are calculated
                        let signal = get_crossover_signal(ema5_val, ema8_val);
                        println!("Trading Signal: {:?}", signal);

                        // Place order on Buy or Sell signal
                        if !has_position && (signal == Signal::Buy || signal == Signal::Sell) {
                            let trade_side = match signal {
                                Signal::Buy => TradeSide::Buy,
                                Signal::Sell => TradeSide::Sell,
                                Signal::Hold => unreachable!(),
                            };

                            let order = Order {
                                account_id: account_id as u64,
                                symbol_id: 41,
                                order_type: OrderType::Market,
                                trade_side: trade_side.clone(),
                                lotsize: 0.01,
                                limit_price: None,
                                stop_price: None,
                                time_in_force: None,
                                expiration_timestamp: None,
                                comment: Some("Auto trade from EMA crossover".to_string()),
                                slippage_in_points: None,
                                label: None,
                                client_order_id: None,
                                relative_stop_loss: None,
                                relative_take_profit: None,
                                guaranteed_stop_loss: None,
                                trailing_stop_loss: None,
                            };

                            println!("Placing {:?} order with signal: {:?}", trade_side, signal);
                            match client.new_order(order).await {
                                Ok(_) => {
                                    println!("Order placed successfully");
                                    has_position = true;
                                }
                                Err(e) => eprintln!("Failed to place order: {}", e),
                            }
                        }
                    }
                }
            }
            
            StreamEvent::Error(err_msg) => {
                eprintln!("Error event received: {}", err_msg);
                
            }

            _ => {}
        }
    }

    Ok(())

}


