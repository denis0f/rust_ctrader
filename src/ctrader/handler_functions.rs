use crate::types::{Account, StreamEvent, Scope, Symbol, BarData};
use prost::Message;


impl super::CtraderClient {
    pub async fn handle_proto_message(
        &self,
        msg: super::ProtoMessage,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match msg.payload_type as i32 {
            //this handles the reponse the ProtoOaApplicationAuthReq
            x if x == super::ProtoOaPayloadType::ProtoOaApplicationAuthRes as i32 => {
                println!("Application authorized successfully.");
                println!("Received Application Auth Response: {:#?}", msg);
            }

            //this handles the response from the ProtoOaGetAccountsListByAccessTokenReq
            x if x == super::ProtoOaPayloadType::ProtoOaGetAccountsByAccessTokenRes as i32 => {
                let mut accounts: Vec<Account> = Vec::new();

                println!("Accounts retreived successfully");
                println!("Received accounts by access token: {:?}", msg);
                let data = msg.payload.unwrap();
                let res = super::ProtoOaGetAccountListByAccessTokenRes::decode(&data[..])?;
                println!("the data now we have got is : {:#?}", res);

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

                self.event_tx.send(StreamEvent::AccountsData(accounts)).await?;
            }

            //this handles the response from the ProtoOaAccountAuthReq
            x if x == super::ProtoOaPayloadType::ProtoOaAccountAuthRes as i32 => {
                println!("Accounts authorized successfully.");
            }

            //this handles the response from the ProtoOaGetSymbolsReq
            x if x == super::ProtoOaPayloadType::ProtoOaSymbolsListRes as i32 => {
                let mut symbols = Vec::<Symbol>::new();
                println!("Symbols retreived successfully.");
                let data = msg.payload.unwrap();
                let symbols_res = super::ProtoOaSymbolsListRes::decode(&data[..])?;
                for symbol in &symbols_res.symbol {
                    let sym = Symbol {
                        symbol_name: symbol.symbol_name.clone().unwrap(),
                        symbol_id: symbol.symbol_id as u64,
                    };
                    symbols.push(sym);
                }

                self.event_tx.send(StreamEvent::SymbolsData(symbols)).await?;
            }

            //this handles the response from the ProtoOaGetHistoricalTrendbarsReq
            x if x == super::ProtoOaPayloadType::ProtoOaGetTrendbarsRes as i32 => {
                let mut trendbars = Vec::<super::BarData>::new();
                println!("Historical trendbars received successfully.");
                println!("the msg received for trendbars is {:#?}", msg);
                let data = msg.payload.unwrap();
                let trendbars_res = super::ProtoOaGetTrendbarsRes::decode(&data[..])?;
                println!(
                    "the trendbars response received is {:#?}",
                    trendbars_res
                );
                
                for bar in &trendbars_res.trendbar {
                    let bar_data = BarData {
                        open: bar.delta_open.unwrap() as f64,
                        close: bar.delta_close.unwrap() as f64,
                        high: bar.delta_high.unwrap() as f64,
                        low: bar.low.unwrap() as f64,
                        volume: bar.volume as u64,
                        timestamp: bar.utc_timestamp_in_minutes.unwrap() as u64,
                    };
                    let bar_data = bar_data.change_to_actual_symbol_price(
                        bar.low.unwrap() as u64,
                        bar.delta_high.unwrap() as u64,
                        bar.delta_close.unwrap() as u64,
                        bar.delta_open.unwrap() as u64,
                        bar.volume as u64,
                        bar.utc_timestamp_in_minutes.unwrap() as u64,
                    );
                    trendbars.push(bar_data);
                }

                //you can send trendbars via event channel if needed
                self.event_tx.send(StreamEvent::TrendbarsData(trendbars)).await?;
            }

            //this handles any error response from the stream 
            x if x == super::ProtoOaPayloadType::ProtoOaErrorRes as i32 => {
                let data = msg.payload.unwrap();
                let err_res = super::ProtoOaErrorRes::decode(&data[..])?;
                let err_msg = err_res.description.unwrap_or_default();
                eprintln!("Error received from server: {}", err_msg);
                let _ = self.event_tx.send(StreamEvent::Error(err_msg)).await;
            }

            // x if x == super::ProtoPayloadType::HeartbeatEvent as i32 => {
            //     println!("Ping request received from server.");
            // }


            _ => {
                println!("Received unhandled message type: {}", msg.payload_type);
                println!("Full message: {:#?}", msg);
            }
        }

        Ok(())
    }
}
