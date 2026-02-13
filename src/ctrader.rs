use crate::{BarData, Endpoint, StreamEvent, TimeFrame, Order};
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
    ProtoPayloadType, ProtoHeartbeatEvent, ProtoOaSubscribeSpotsReq, ProtoOaSubscribeSpotsRes, ProtoOaUnsubscribeSpotsReq, ProtoOaUnsubscribeSpotsRes,
    ProtoOaSubscribeLiveTrendbarReq, ProtoOaSubscribeLiveTrendbarRes, ProtoOaUnsubscribeLiveTrendbarReq, ProtoOaUnsubscribeLiveTrendbarRes,
    ProtoOaSpotEvent, 
    ProtoOaTrader, ProtoOaTraderReq, ProtoOaTraderRes, ProtoOaTradeData, ProtoOaNewOrderReq, ProtoOaClosePositionReq,
    ProtoOaTradeSide, ProtoOaOrderType, ProtoOaOrderErrorEvent, ProtoOaExecutionEvent
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
        };

        Ok((Arc::new(client), event_rx))
    }

    pub async fn send_message(
        &self,
        message: ProtoMessage,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut stream = self.stream.lock().await;
        let n = message.encode_to_vec().len() as u32;

        stream.write_u32(n).await?;
        let encoded_msg = message.encode_to_vec();
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
        // Implementation for authorizing the application
        let _auth_request = ProtoOaApplicationAuthReq {
            payload_type: Some(ProtoOaPayloadType::ProtoOaApplicationAuthReq as i32),
            client_id: self.client_id.clone(),
            client_secret: self.client_secret.clone(),
        };

        let _auth_request_encoded = _auth_request.encode_to_vec();

        let _message = ProtoMessage {
            payload_type: ProtoOaPayloadType::ProtoOaApplicationAuthReq as u32,
            payload: Some(_auth_request_encoded),
            client_msg_id: Some(String::from("application authorization")),
        };

        self.send_message(_message)
            .await
            .expect("Failed to send application authorization request");

        Ok(())
    }

    pub async fn get_accounts(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Getting accounts...");
        // Implementation for getting accounts
        let _accounts_request = ProtoOaGetAccountListByAccessTokenReq {
            payload_type: Some(ProtoOaPayloadType::ProtoOaGetAccountsByAccessTokenReq as i32),
            access_token: self.access_token.clone(),
        };

        let _accounts_request_encoded = _accounts_request.encode_to_vec();

        let _message = ProtoMessage {
            payload_type: ProtoOaPayloadType::ProtoOaGetAccountsByAccessTokenReq as u32,
            payload: Some(_accounts_request_encoded),
            client_msg_id: Some(String::from("get accounts by access token")),
        };

        self.send_message(_message)
            .await
            .expect("Failed to send get accounts request");

        Ok(())
    }

    pub async fn authorize_account(
        &self,
        account_id: i64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("Authorizing account with ID: {}...", account_id);
        // Implementation for authorizing the account
        let _auth_request = ProtoOaAccountAuthReq {
            payload_type: Some(ProtoOaPayloadType::ProtoOaAccountAuthReq as i32),
            ctid_trader_account_id: account_id,
            access_token: self.access_token.clone(),
        };

        let _auth_request_encoded = _auth_request.encode_to_vec();

        let _message = ProtoMessage {
            payload_type: ProtoOaPayloadType::ProtoOaAccountAuthReq as u32,
            payload: Some(_auth_request_encoded),
            client_msg_id: Some(format!("account authorization {}", account_id)),
        };

        self.send_message(_message)
            .await
            .expect("Failed to send account authorization request");

        Ok(())
    }

    pub async fn get_symbols(
        &self,
        account_id: i64,
        include_archived: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("Getting symbols...");
        // Implementation for getting symbols
        let _symbols_request = ProtoOaSymbolsListReq {
            payload_type: Some(ProtoOaPayloadType::ProtoOaSymbolsListReq as i32),
            ctid_trader_account_id: account_id,
            include_archived_symbols: Some(include_archived),
        };

        let _symbols_request_encoded = _symbols_request.encode_to_vec();

        let _message = ProtoMessage {
            payload_type: ProtoOaPayloadType::ProtoOaSymbolsListReq as u32,
            payload: Some(_symbols_request_encoded),
            client_msg_id: Some(String::from("get symbols list")),
        };

        self.send_message(_message)
            .await
            .expect("Failed to send get symbols request");

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

        let req_encoded = req.encode_to_vec();
        let message = ProtoMessage {
            payload_type: ProtoOaPayloadType::ProtoOaGetTrendbarsReq as u32,
            payload: Some(req_encoded),
            client_msg_id: Some(String::from("get trend bar data")),
        };
        self.send_message(message)
            .await
            .expect("Failed to send get trend bar data request");

        Ok(())
    }
    pub async fn keep_alive(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Sending keep-alive heartbeat...");
        // Implementation for keep-alive mechanism
        let heartbeat = ProtoHeartbeatEvent {
            payload_type: Some(ProtoPayloadType::HeartbeatEvent as i32),
        };

        self.send_message(ProtoMessage {
            payload_type: ProtoPayloadType::HeartbeatEvent as u32,
            payload: Some(heartbeat.encode_to_vec()),
            client_msg_id: Some(String::from("keep alive")),
        })
        .await
        .expect("Failed to send heartbeat message");

        Ok(())
    }


    pub async fn subscribe_spot(&self, account_id: i64, symbol_id: i64) -> Result<(), Box<dyn std::error::Error>> {
        let subscribe_request = ProtoOaSubscribeSpotsReq {
            payload_type: Some(ProtoOaPayloadType::ProtoOaSubscribeSpotsReq as i32),
            ctid_trader_account_id: account_id, //account_id,
            symbol_id: vec![symbol_id],
            subscribe_to_spot_timestamp: Some(true),
        };

        let subscribe_request_encoded = subscribe_request.encode_to_vec();

        let message = ProtoMessage {
            payload_type: ProtoOaPayloadType::ProtoOaSubscribeSpotsReq as u32,
            payload: Some(subscribe_request_encoded),
            client_msg_id: Some(String::from("subscribe to spot events")),
        };

        self.send_message(message)
            .await
            .expect("Failed to send subscribe to spot events request");

        Ok(())
    }

    pub async fn subscribe_live_bars(&self, account_id: i64, symbol_id: i64, timeframe: TimeFrame) -> Result<(), Box<dyn std::error::Error>> {

        //first subscribe to the live spot events for the symbol
        self.subscribe_spot(account_id, symbol_id).await?;

        let subscribe_request = ProtoOaSubscribeLiveTrendbarReq {
            payload_type: Some(ProtoOaPayloadType::ProtoOaSubscribeLiveTrendbarReq as i32),
            ctid_trader_account_id: account_id,
            symbol_id: symbol_id,
            period: timeframe.change_proto_trendbar_period() as i32,
        };

        let subscribe_request_encoded = subscribe_request.encode_to_vec();

        let message = ProtoMessage {
            payload_type: ProtoOaPayloadType::ProtoOaSubscribeLiveTrendbarReq as u32,
            payload: Some(subscribe_request_encoded),
            client_msg_id: Some(String::from("subscribe to live bars")),
        };

        self.send_message(message)
            .await
            .expect("Failed to send subscribe to live bars request");

        Ok(())
    }




}
