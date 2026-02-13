use crate::types::{Account, BarData, Quote, Scope, StreamEvent, Symbol};
use prost::Message;

use crate::handle_option_value;

impl super::CtraderClient {
    pub async fn handle_proto_message(
        &self,
        msg: super::ProtoMessage,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match msg.payload_type as i32 {
            //this handles the reponse the ProtoOaApplicationAuthReq
            x if x == super::ProtoOaPayloadType::ProtoOaApplicationAuthRes as i32 => {
                self.event_tx
                    .send(StreamEvent::ApplicationAuthorized(String::from(
                        "Application authorized successfully.",
                    )))
                    .await?;
            }

            //this handles the response from the ProtoOaAccountAuthReq
            x if x == super::ProtoOaPayloadType::ProtoOaAccountAuthRes as i32 => {
                println!("Account authorization response received from server.");
                println!("Full message: {:#?}", msg);
                let decoded_msg = super::ProtoOaAccountAuthRes::decode(&msg.payload.unwrap()[..])?;
                println!("Decoded message: {:#?}", decoded_msg);
                self.event_tx
                    .send(StreamEvent::AccountAuthorized(String::from(
                        "Account authorized successfully.",
                    )))
                    .await?;
            }

            //this handles the response from the ProtoOaGetAccountsListByAccessTokenReq
            x if x == super::ProtoOaPayloadType::ProtoOaGetAccountsByAccessTokenRes as i32 => {
                let mut accounts: Vec<Account> = Vec::new();
                let data = msg.payload.unwrap();
                let res = super::ProtoOaGetAccountListByAccessTokenRes::decode(&data[..])?;

                for account in &res.ctid_trader_account {
                    let acc: Account = Account {
                        id: account.ctid_trader_account_id,
                        broker: account.broker_title_short.clone().unwrap(),
                        is_live: account.is_live.unwrap(),
                        scope: if res.permission_scope == Some(1_i32) {
                            Scope::Trading
                        } else {
                            Scope::Accounts
                        },
                    };
                    accounts.push(acc);
                }

                self.event_tx
                    .send(StreamEvent::AccountsData(accounts))
                    .await?;
            }

            //this handles the response from the ProtoOaGetSymbolsReq
            x if x == super::ProtoOaPayloadType::ProtoOaSymbolsListRes as i32 => {
                let mut symbols = Vec::<Symbol>::new();
                let data = msg.payload.unwrap();
                let symbols_res = super::ProtoOaSymbolsListRes::decode(&data[..])?;
                for symbol in &symbols_res.symbol {
                    let sym = Symbol {
                        symbol_name: symbol.symbol_name.clone().unwrap(),
                        symbol_id: symbol.symbol_id as u64,
                    };
                    symbols.push(sym);
                }

                self.event_tx
                    .send(StreamEvent::SymbolsData(symbols))
                    .await?;
            }

            //this handles the response from the ProtoOaGetHistoricalTrendbarsReq
            x if x == super::ProtoOaPayloadType::ProtoOaGetTrendbarsRes as i32 => {
                let mut trendbars = Vec::<super::BarData>::new();
                let data = msg.payload.unwrap();
                let trendbars_res = super::ProtoOaGetTrendbarsRes::decode(&data[..])?;

                for bar in &trendbars_res.trendbar {
                    let bar_data = BarData {
                        open: bar.delta_open.unwrap() as f64,
                        close: handle_option_value(bar.delta_close.map(|v| v as f64)),
                        high: bar.delta_high.unwrap() as f64,
                        low: bar.low.unwrap() as f64,
                        volume: bar.volume as u64,
                        timestamp: bar.utc_timestamp_in_minutes.unwrap() as u64,
                    };
                    let bar_data = bar_data.change_to_actual_symbol_price(
                        bar.low.unwrap() as u64,
                        bar.delta_high.unwrap() as u64,
                        handle_option_value(bar.delta_close.map(|v| v as u64)),
                        bar.delta_open.unwrap() as u64,
                        bar.volume as u64,
                        bar.utc_timestamp_in_minutes.unwrap() as u64,
                    );
                    trendbars.push(bar_data);
                }

                //you can send trendbars via event channel if needed
                self.event_tx
                    .send(StreamEvent::TrendbarsData(trendbars))
                    .await?;
            }

            //this handles any error response from the stream
            x if x == super::ProtoOaPayloadType::ProtoOaErrorRes as i32 => {
                let data = msg.payload.unwrap();
                let err_res = super::ProtoOaErrorRes::decode(&data[..])?;
                let err_msg = err_res.description.unwrap_or_default();
                let _ = self.event_tx.send(StreamEvent::Error(err_msg)).await;
            }

            //this handles heartbeat messages from the server
            x if x == super::ProtoPayloadType::HeartbeatEvent as i32 => {
                println!("Heartbeat from the server received.");
            }

            //this handles any generic error response from the server
            x if x == super::ProtoPayloadType::ErrorRes as i32 => {
                println!("Error response received from server.");
                println!("Full message: {:#?}", msg);
            }

            //this handles spot data updates if you have subscribed to them
            x if x == super::ProtoOaPayloadType::ProtoOaSubscribeSpotsRes as i32 => {
                self.event_tx
                    .send(StreamEvent::SubscribeSpotsData(String::from(
                        "Subscribed to spot data successfully.",
                    )))
                    .await?;
            }

            //this handles the live_bars subscribed response
            x if x == super::ProtoOaPayloadType::ProtoOaSubscribeLiveTrendbarRes as i32 => {
                self.event_tx
                    .send(StreamEvent::SubscribeLiveBarsData(String::from(
                        "Subscribed to live bars data successfully.",
                    )))
                    .await?;
            }

            //this handles the ProtoSpotEvent
            x if x == super::ProtoOaPayloadType::ProtoOaSpotEvent as i32 => {
                self.keep_alive().await?; //send a keep-alive message to prevent disconnection due to inactivity
                let data = msg.payload.unwrap();
                let spot_event = super::ProtoOaSpotEvent::decode(&data[..])?;
                let quote = Quote {
                    symbol_id: spot_event.symbol_id,
                    bid: handle_option_value(spot_event.bid.map(|v| v as f64)),
                    ask: handle_option_value(spot_event.ask.map(|v| v as f64)),
                    timestamp: spot_event.timestamp.unwrap() as u64,
                };
                //here check if the vector is empty or not before unwrapping, otherwise it will panic if the server sends an empty vector for some reason
                if !spot_event.trendbar.is_empty() {
                    let bar = spot_event.trendbar.last().unwrap(); //get the latest bar data from the vector
                    let bar_data = BarData {
                        open: bar.delta_open.unwrap() as f64,
                        close: handle_option_value(bar.delta_close.map(|v| v as f64)),
                        high: bar.delta_high.unwrap() as f64,
                        low: bar.low.unwrap() as f64,
                        volume: bar.volume as u64,
                        timestamp: bar.utc_timestamp_in_minutes.unwrap() as u64,
                    };
                    let bar_data = bar_data.change_to_actual_symbol_price(
                        bar.low.unwrap() as u64,
                        bar.delta_high.unwrap() as u64,
                        handle_option_value(bar.delta_close.map(|v| v as u64)),
                        bar.delta_open.unwrap() as u64,
                        bar.volume as u64,
                        bar.utc_timestamp_in_minutes.unwrap() as u64,
                    );
                    self.event_tx
                        .send(StreamEvent::LiveData((Some(quote), Some(bar_data))))
                        .await?;
                } else {
                    self.event_tx.send(StreamEvent::LiveData((Some(quote), None))).await?;
                }
            }

            //this handles the trade response from the server after placing a new order or closing a position

            //this handles disconnection notifications
            x if x == super::ProtoOaPayloadType::ProtoOaClientDisconnectEvent as i32 => {
                println!("Disconnection notification received from server.");
                println!("Full message: {:#?}", msg);
            }

            //this handles the order event errors 


            //handles the exectuion event 


            //catch-all for unhandled message types
            _ => {
                println!("Received unhandled message type: {}", msg.payload_type);
                println!("Full message: {:#?}", msg);
            }
        }

        Ok(())
    }
}
