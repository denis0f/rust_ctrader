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
use rust_ctrader::{CtraderClient, Endpoint, StreamEvent, TimeFrame, types::{Signal, Position}, strategies::{self,  moving_average_strategy::{Ema}}};
use std::env;



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    dotenv().ok();

    println!("Getting the environment variables...");

    let client_id = env::var("client_id")?;
    let client_secret = env::var("client_secret")?;
    let access_token = env::var("access_token")?;


    println!("Connecting to cTrader Open API...");

    let (mut client, mut event_rx) = CtraderClient::connect(&client_id, &client_secret, &access_token, Endpoint::Demo).await?;

    client.start().await;

    client.authorize_application().await?;

    let short_period: usize = 2_usize;
    let long_period = 3_usize;

    let mut account_id = 0_i64;
    let mut fast_ema = Ema::new(short_period);
    let mut slow_ema = Ema::new(long_period);

    let mut fast_prev_ema: Option<f64> = None;
    let mut slow_prev_ema: Option<f64> = None;
    let mut current_slow_ema = None;
    let mut current_fast_ema = None;
    let mut prev_signal = Signal::Hold;

    let mut positions: Vec<Position> = Vec::new();

    let mut in_position = false;


    //listen for events
    while let Some(event) = event_rx.recv().await{
        match event{
            StreamEvent::ApplicationAuthorized(msg) => {
                println!("Application authorized event received: {}", msg);
                client.get_accounts().await?;
            }
            StreamEvent::AccountsData(accounts) => {
                println!("Accounts data received: {:#?}", accounts);
                account_id = accounts[0].id as i64; //select the first account for authorization
                client.authorize_account(accounts[0].id as i64).await?; 
            }
            StreamEvent::AccountAuthorized(msg) => {
                println!("Account authorized event received: {}", msg);
                //Further actions can be taken here after account authorization
                // if account_id != 0_i64{
                //     client.get_symbols(account_id, false).await?;
                // }           
                //requesting trend bar data for the last week for the selected symbol client.get_trend_bar_data(symbols[4].symbol_id as i64, TimeFrame::M1, account_id, from_timestamp, to_timestamp).await?;
                
                client.subscribe_live_bars(account_id, 41 as i64, TimeFrame::M1).await?;

                println!("Subscribed to live bars for symbol ID 41 on account ID {}.", account_id);

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
                //on last closed bar receiving to calculate the ema 
                let close_price = match &last_closed_candle{
                    
                    Some(bar_data) => {
                        println!("the last_closed_candle data received: {:#?}", last_closed_candle);
                        bar_data.close
                    },
                    None => None
                };
            

                if let Some(close) = close_price{
                    current_slow_ema = slow_ema.update(close);
                    current_fast_ema = fast_ema.update(close);
                    println!("Previous EMAs - Fast EMA: {:?}, Slow EMA: {:?}", fast_prev_ema, slow_prev_ema);
                    println!("Updated EMAs - Fast EMA: {:?}, Slow EMA: {:?}", current_fast_ema, current_slow_ema);

                }
                
                //getting the signal
                let signal = strategies::get_signal(current_fast_ema, current_slow_ema, &mut slow_prev_ema, &mut fast_prev_ema);

                //placing a trade accorging to the signal genrated 
                if signal != Signal::Hold{
                    println!("Signal generated: {:?}. Attempting to take a trade...", signal);
                    strategies::take_a_trade(&mut client.clone(), account_id, signal, &mut in_position, &mut prev_signal, &mut positions).await?;
                }

            }

            //handle order execution events
            StreamEvent::ExecutionEvent(position) => {

                positions.push(position.clone());
                println!("Order execution event received: {:#?}", position);
                // Further actions can be taken here after receiving order execution events
            }
            
            StreamEvent::Error(err_msg) => {
                eprintln!("Error event received: {}", err_msg);
                
            }

            _ => {}
        }
    }

    Ok(())

}


