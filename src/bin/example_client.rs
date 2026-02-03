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
use rust_ctrader::{CtraderClient, Endpoint, StreamEvent, TimeFrame};
use std::env;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    dotenv().ok();

    let client_id = env::var("client_id")?;
    let client_secret = env::var("client_secret")?;
    let access_token = env::var("access_token")?;

    let (client, mut event_rx) = CtraderClient::connect(&client_id, &client_secret, &access_token, Endpoint::Demo).await?;

    client.start().await;

    client.authorize_application().await?;

    let mut account_id = 0_i64; // Replace with a valid account ID after retrieving accounts

    //listen for events
    while let Some(event) = event_rx.recv().await{
        match event{
            StreamEvent::ApplicationAuthorized(msg) => {
                println!("Application authorized event received: {}", msg);
                client.get_accounts().await?;
            }
            StreamEvent::AccountsData(accounts) => {
                println!("Accounts data received: {:#?}", accounts);
                account_id = accounts[1].id as i64; //select the first account for authorization
                client.authorize_account(accounts[1].id as i64).await?;
            }
            StreamEvent::AccountAuthorized(msg) => {
                println!("Account authorized event received: {}", msg);
                // Further actions can be taken here after account authorization
                if account_id != 0_i64{
                    client.get_symbols(account_id, false).await?;
                }                
            }
            StreamEvent::SymbolsData(symbols) => {
                println!("Symbols data received: {:#?}", symbols[4]);

                //prepare the timestamp from last monday to today in milliseconds using the chrono crate
                use chrono::{Utc, Datelike, Duration};
                let now = Utc::now();
                let last_monday = now - Duration::days((now.weekday().num_days_from_monday() + 7) as i64);
                let from_timestamp = last_monday.timestamp_millis();
                let to_timestamp = now.timestamp_millis();


                for symbol in symbols{
                    if symbol.symbol_name == "XAUUSD"{
                        println!("the symbol id for XAUUSD is {}", symbol.symbol_id);
                        client.get_trend_bar_data(symbol.symbol_id as i64, TimeFrame::D1, account_id, from_timestamp, to_timestamp).await?;
                        break;
                    }
                }

            }
            StreamEvent::TrendbarsData(trendbars) => {
                println!("Trend bars data received: {:#?}", trendbars);
                // Further actions can be taken here after receiving trend bars data
            }
            StreamEvent::Error(err_msg) => {
                eprintln!("Error event received: {}", err_msg);
            }
            _ => {}
        }
    }

    Ok(())

}