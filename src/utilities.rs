use chrono::{Utc, Datelike, Duration, Local, TimeZone};

use crate::CtraderClient;


//this function is for handling the timestamp to string from 

pub fn handle_timestamp(timestamp: u64) -> String{
    // `timestamp` is provided in minutes; convert to seconds
    let secs = timestamp as i64 * 60;
    // Create UTC DateTime from seconds and convert to local timezone
    if let Some(dt_utc) = Utc.timestamp_opt(secs, 0).single() {
        let dt_local = dt_utc.with_timezone(&Local);
        dt_local.to_string()
    } else {
        // Fallback to current local time if conversion fails
        timestamp.to_string()
    }
}


//method for handling the option values
pub fn handle_option_value<T>(option: Option<T>) -> Option<T> {
    match option {
        Some(value) => Some(value),
        None => None
    }
}


//converting lot sizes into the protocol standard volume 

pub async fn lots_to_protocol_std_volume(client: &CtraderClient, symbol_id: i64, account_id: i64 , order_lotsize: f64)-> Result<i64, Box<dyn std::error::Error>>{
    let symbol_id_u64 = symbol_id as u64;
    {
            let map = client.symbol_data.lock().await;
            if !map.contains_key(&symbol_id_u64) {
                drop(map);
                client.get_symbol_by_id(account_id, symbol_id, Some("new order symbol lookup".to_string())).await?;
            }
        }

        // Get lot_size
        let lot_size = {
            let map = client.symbol_data.lock().await;
            map.get(&symbol_id_u64).and_then(|sd| sd.lot_size).unwrap_or(10000) as f64
        };

        // Compute volume in protocol units (0.01 of a unit)
        let volume = (order_lotsize * lot_size) as i64;
        
        Ok(volume)


}


