use crate::{BarData, Endpoint, StreamEvent, TimeFrame, Order, RelativeBarData, Quote, utilities};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, mpsc};
use tokio_rustls::client::TlsStream;

use prost::Message;

//the proto definition of different messages to be sent and decoded
use crate::open_api::{
    ProtoMessage, ProtoOaAccountAuthReq, ProtoOaAccountAuthRes, ProtoOaApplicationAuthReq,
    ProtoOaErrorRes, ProtoOaGetAccountListByAccessTokenReq,
    ProtoOaGetAccountListByAccessTokenRes, ProtoOaGetTrendbarsReq, ProtoOaGetTrendbarsRes,
    ProtoOaPayloadType, ProtoOaSymbolsListReq, ProtoOaSymbolsListRes,
    ProtoPayloadType, ProtoHeartbeatEvent, ProtoOaSubscribeSpotsReq, ProtoOaUnsubscribeSpotsReq,
    ProtoOaSubscribeLiveTrendbarReq, ProtoOaUnsubscribeLiveTrendbarReq,
    ProtoOaSpotEvent, ProtoOaSymbolByIdReq, ProtoOaSymbolByIdRes,
    ProtoOaNewOrderReq, ProtoOaClosePositionReq,
    ProtoOaTradeSide, ProtoOaOrderType, ProtoOaOrderErrorEvent, ProtoOaExecutionEvent,
    ProtoOaOrderListReq, ProtoOaOrderListRes
};

//the stream builder module
pub mod handler_functions;
pub mod stream_builder;

pub struct CtraderClient {
    stream: Arc<Mutex<TlsStream<TcpStream>>>,
    access_token: String,
    client_secret: String,
    client_id: String,
    event_tx: mpsc::Sender<StreamEvent>,

    // cached symbol metadata (max/min volume, digits, etc).  keyed by symbol
    // id so that we can avoid redundant round‑trips when subscribing to live
    // data.  a `Mutex` is fine because the typical access pattern is
    // ``lock->check->insert`` which keeps the critical section tiny.
    pub symbol_data: Mutex<HashMap<u64, crate::types::SymbolData>>,

    // live state used by the spot/live-bar logic.  These are stored on the
    // client instance so that successive calls to `handle_proto_message` can
    // compare the previous bar/quote timestamp instead of resetting on each
    // invocation.
    last_bar: Mutex<Option<RelativeBarData>>,
    last_quote: Mutex<Option<Quote>>,
    last_bar_ts: Mutex<Option<u64>>,
}

impl CtraderClient {
    pub async fn connect(
        client_id: &str,
        client_secret: &str,
        access_token: &str,
        endpoint: Endpoint,
    ) -> Result<(Arc<Self>, mpsc::Receiver<StreamEvent>), Box<dyn std::error::Error>> {
        let host = match endpoint {
            Endpoint::Demo => "demo.ctraderapi.com",
            Endpoint::Live => "live.ctraderapi.com",
        };
        let stream_ = stream_builder::initialize_stream(host, 5035)
            .await
            .expect("Failed to initialize the stream.");

        let (event_tx, event_rx) = mpsc::channel(100);

        let client = Self {
            stream: Arc::new(Mutex::new(stream_)),
            access_token: access_token.to_string(),
            client_secret: client_secret.to_string(),
            client_id: client_id.to_string(),
            event_tx,
            symbol_data: Mutex::new(HashMap::new()),
            // initialize live-bar state as empty; the first spot event will
            // populate them.
            last_bar: Mutex::new(None),
            last_quote: Mutex::new(None),
            last_bar_ts: Mutex::new(None),
        };

        Ok((Arc::new(client), event_rx))
    }

    pub async fn send_message<M: Message>(
        &self,
        payload_type: u32,
        payload: M,
        client_msg_id: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let payload_encoded = payload.encode_to_vec();
        let message = ProtoMessage {
            payload_type,
            payload: Some(payload_encoded),
            client_msg_id,
        };

        let mut stream = self.stream.lock().await;
        let encoded_msg = message.encode_to_vec();
        let n = encoded_msg.len() as u32;

        stream.write_u32(n).await?;
        stream.write_all(&encoded_msg).await?;

        Ok(())
    }

    pub async fn start(self: &Arc<Self>) {
        println!("Starting read loop...");
        let client = Arc::clone(self);

        tokio::spawn(async move {
            if let Err(e) = client.read_loop().await {
                let _ = client
                    .event_tx
                    .send(StreamEvent::Error(e.to_string()))
                    .await;
            }
        });
    }

    async fn read_loop(&self) -> anyhow::Result<()> {
        loop {
            match self.read_proto_message().await {
                Ok(msg) => {
                    if let Err(e) = self.handle_proto_message(msg).await {
                        println!("Error handling proto message: {}", e);
                        return Err(anyhow::anyhow!(e.to_string()));
                    }
                }
                Err(e) => {
                    println!("Error reading proto message: {}", e);
                    // EOF or connection error → exit loop
                    return Err(anyhow::anyhow!(e.to_string()));
                }
            }
        }
    }

    async fn read_proto_message(&self) -> anyhow::Result<ProtoMessage> {
        let mut stream = self.stream.lock().await;
        let len = stream.read_u32().await?;

        let mut buf = vec![0u8; len as usize];
        stream.read_exact(&mut buf).await?;

        let msg = ProtoMessage::decode(&buf[..])?;
        Ok(msg)
    }

    pub async fn authorize_application(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Authorizing application...");
        let req = ProtoOaApplicationAuthReq {
            payload_type: Some(ProtoOaPayloadType::ProtoOaApplicationAuthReq as i32),
            client_id: self.client_id.clone(),
            client_secret: self.client_secret.clone(),
        };

        self.send_message(
            ProtoOaPayloadType::ProtoOaApplicationAuthReq as u32,
            req,
            Some(String::from("application authorization")),
        )
        .await?;
        Ok(())
    }

    pub async fn get_accounts(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Getting accounts...");
        let req = ProtoOaGetAccountListByAccessTokenReq {
            payload_type: Some(ProtoOaPayloadType::ProtoOaGetAccountsByAccessTokenReq as i32),
            access_token: self.access_token.clone(),
        };

        self.send_message(
            ProtoOaPayloadType::ProtoOaGetAccountsByAccessTokenReq as u32,
            req,
            Some(String::from("get accounts by access token")),
        )
        .await?;
        Ok(())
    }

    pub async fn authorize_account(
        &self,
        account_id: i64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("Authorizing account with ID: {}...", account_id);
        let req = ProtoOaAccountAuthReq {
            payload_type: Some(ProtoOaPayloadType::ProtoOaAccountAuthReq as i32),
            ctid_trader_account_id: account_id,
            access_token: self.access_token.clone(),
        };

        self.send_message(
            ProtoOaPayloadType::ProtoOaAccountAuthReq as u32,
            req,
            Some(format!("account authorization {}", account_id)),
        )
        .await?;
        Ok(())
    }

    pub async fn get_symbols(
        &self,
        account_id: i64,
        include_archived: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("Getting symbols...");
        let req = ProtoOaSymbolsListReq {
            payload_type: Some(ProtoOaPayloadType::ProtoOaSymbolsListReq as i32),
            ctid_trader_account_id: account_id,
            include_archived_symbols: Some(include_archived),
        };

        self.send_message(
            ProtoOaPayloadType::ProtoOaSymbolsListReq as u32,
            req,
            Some(String::from("get symbols list")),
        )
        .await?;
        Ok(())
    }

    pub async fn get_trend_bar_data(
        &self,
        symbol_id: i64,
        period: TimeFrame,
        account_id: i64,
        from_timestamp: i64,
        to_timestamp: i64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("Getting historical trend bar data...");

        let period = period.change_proto_trendbar_period();

        let req = ProtoOaGetTrendbarsReq {
            payload_type: Some(ProtoOaPayloadType::ProtoOaGetTrendbarsReq as i32),
            symbol_id: symbol_id,
            period: period as i32,
            ctid_trader_account_id: account_id,
            from_timestamp: Some(from_timestamp),
            to_timestamp: Some(to_timestamp),
            count: Some(200),
        };

        self.send_message(
            ProtoOaPayloadType::ProtoOaGetTrendbarsReq as u32,
            req,
            Some(String::from("get trend bar data")),
        )
        .await?;
        Ok(())
    }

    pub async fn keep_alive(&self) -> Result<(), Box<dyn std::error::Error>> {
        let heartbeat = ProtoHeartbeatEvent {
            payload_type: Some(ProtoPayloadType::HeartbeatEvent as i32),
        };

        self.send_message(ProtoPayloadType::HeartbeatEvent as u32, heartbeat, Some(String::from("keep alive"))).await?;

        Ok(())
    }

    pub async fn subscribe_spot(&self, account_id: i64, symbol_id: i64) -> Result<(), Box<dyn std::error::Error>> {
        let req = ProtoOaSubscribeSpotsReq {
            payload_type: Some(ProtoOaPayloadType::ProtoOaSubscribeSpotsReq as i32),
            ctid_trader_account_id: account_id,
            symbol_id: vec![symbol_id],
            subscribe_to_spot_timestamp: Some(true),
        };

        self.send_message(
            ProtoOaPayloadType::ProtoOaSubscribeSpotsReq as u32,
            req,
            Some(String::from("subscribe to spot events")),
        )
        .await?;
        Ok(())
    }

    // note: we accept `self: &Arc<Self>` so that a cloned handle can be moved
    // into the spawned task without borrowing `&self` itself.
    pub async fn subscribe_live_bars(self: &Arc<Self>, account_id: i64, symbol_id: i64, timeframe: TimeFrame) -> Result<(), Box<dyn std::error::Error>> {
        println!("Subscribing to live bars for symbol ID: {} and timeframe: {:?}...", symbol_id, timeframe);
        self.subscribe_spot(account_id, symbol_id).await?;

        // ensure we have the data cached; only ask once per symbol. spawn a
        // background task that we await after we finish sending the subscription
        // message. by cloning `Arc<Self>` we avoid lifetime issues.
        let maybe_task = {
            let client = Arc::clone(self);
            let map = client.symbol_data.lock().await;
            if !map.contains_key(&(symbol_id as u64)) {
                let client_message = Some(String::from("sent by live_bar_subscription"));
                drop(map);
                Some(tokio::spawn(async move {
                    client.get_symbol_by_id(account_id, symbol_id, client_message).await
                }))
            } else {
                None
            }
        };
        println!("Sending live bar subscription message...");

        let req = ProtoOaSubscribeLiveTrendbarReq {
            payload_type: Some(ProtoOaPayloadType::ProtoOaSubscribeLiveTrendbarReq as i32),
            ctid_trader_account_id: account_id,
            symbol_id: symbol_id,
            period: timeframe.change_proto_trendbar_period() as i32,
        };

        self.send_message(
            ProtoOaPayloadType::ProtoOaSubscribeLiveTrendbarReq as u32,
            req,
            Some(String::from("subscribe to live bars")),
        )
        .await?;

        println!("Live bar subscription message sent successfully.");

        if let Some(handle) = maybe_task {
            // propagate any error from the spawned symbol request
            handle.await??;
        }
        println!("Live bar subscription setup complete.");

        Ok(())
    }

    pub async fn new_order(&self, order: Order) -> Result<(), Box<dyn std::error::Error>> {
        // Ensure we have the symbol data cached
        let symbol_id_u64 = order.symbol_id;
        let account_id = order.account_id as i64;
        let symbol_id = order.symbol_id as i64;

        

        // Compute volume in protocol units (0.01 of a unit)
        let volume = utilities::lots_to_protocol_std_volume(self, symbol_id, account_id, order.lotsize).await?;

        let new_order_request = ProtoOaNewOrderReq {
            payload_type: Some(ProtoOaPayloadType::ProtoOaNewOrderReq as i32),
            ctid_trader_account_id: order.account_id as i64,
            symbol_id: order.symbol_id as i64,
            order_type: match order.order_type {
                crate::types::OrderType::Market => ProtoOaOrderType::Market as i32,
                crate::types::OrderType::Limit => ProtoOaOrderType::Limit as i32,
                crate::types::OrderType::Stop => ProtoOaOrderType::Stop as i32,
                crate::types::OrderType::StopLimit => ProtoOaOrderType::StopLimit as i32,
            },
            trade_side: match order.trade_side {
                crate::types::TradeSide::Buy => ProtoOaTradeSide::Buy as i32,
                crate::types::TradeSide::Sell => ProtoOaTradeSide::Sell as i32,
            },
            volume,
            limit_price: order.limit_price,
            stop_price: order.stop_price,
            time_in_force: None, // You can map order.time_in_force if needed
            expiration_timestamp: order.expiration_timestamp,
            stop_loss: order.relative_stop_loss.map(|x| x as f64),
            take_profit: order.relative_take_profit.map(|x| x as f64),
            comment: order.comment.clone(),
            base_slippage_price: None,
            slippage_in_points: order.slippage_in_points,
            label: order.label.clone(),
            position_id: None,
            client_order_id: order.client_order_id.clone(),
            relative_stop_loss: order.relative_stop_loss,
            relative_take_profit: order.relative_take_profit,
            guaranteed_stop_loss: order.guaranteed_stop_loss,
            trailing_stop_loss: order.trailing_stop_loss,
            stop_trigger_method: None,
        };

        println!("Sending new order request...");
        self.send_message(
            ProtoOaPayloadType::ProtoOaNewOrderReq as u32,
            new_order_request,
            Some(String::from("new order request")),
        )
        .await?;
        println!("New order request sent successfully.");
        Ok(())
    }

    pub async fn close_position(
        &self,
        account_id: i64,
        position_id: i64,
        volume: i64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("Closing position {}...", position_id);



        let req = ProtoOaClosePositionReq {
            payload_type: Some(ProtoOaPayloadType::ProtoOaClosePositionReq as i32),
            ctid_trader_account_id: account_id,
            position_id,
            volume
        };

        self.send_message(
            ProtoOaPayloadType::ProtoOaClosePositionReq as u32,
            req,
            Some(String::from("close position request")),
        )
        .await?;

        println!("Close position request sent successfully.");
        Ok(())
    }

    //getting the open positions for an account
    pub async fn get_open_positions(&self, account_id: i64) -> Result<(), Box<dyn std::error::Error>> {
        println!("Getting open positions for account ID: {}...", account_id);

        ///send the order list request message to get the open positions for the account
        let req = ProtoOaOrderListReq {
            payload_type: Some(ProtoOaPayloadType::ProtoOaOrderListReq as i32),
            ctid_trader_account_id: account_id,
            from_timestamp: None,
            to_timestamp: None,
        };

        self.send_message(
            ProtoOaPayloadType::ProtoOaOrderListReq as u32,
            req,
            Some(String::from("get open positions")),
        ).await?;

        Ok(())
    }

    pub async fn get_symbol_by_id(&self, account_id: i64, symbol_id: i64, client_msg: Option<String>) -> anyhow::Result<()> {
        println!("Getting symbol by ID...");
        let req = ProtoOaSymbolByIdReq {
            payload_type: Some(ProtoOaPayloadType::ProtoOaSymbolByIdReq as i32),
            ctid_trader_account_id: account_id,
            symbol_id: vec![symbol_id],
        };

        
        match self.send_message(
            ProtoOaPayloadType::ProtoOaSymbolByIdReq as u32,
            req,
            if client_msg.is_some() { client_msg} else {Some(String::from("get symbol by id"))},
        ).await {
                Ok(()) => {

                        println!("the symbol request sent successfully");
                        return Ok(());
                        
                    }
                
                Err(e) => {
                    println!("Error reading proto message");

                    return Err(anyhow::format_err!(e.to_string()));
                    
                }
            }
        
    }

    /// Return a clone of the cached symbol metadata, if available.
    pub async fn symbol_data(&self, symbol_id: u64) -> Option<crate::types::SymbolData> {
        self.symbol_data.lock().await.get(&symbol_id).cloned()
    }


}
