use chrono::{Utc, Datelike, Duration, Local, TimeZone};


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