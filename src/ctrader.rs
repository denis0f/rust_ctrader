use crate::{Account, Endpoint, StreamEvent, Scope, Tokens, BarData};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, mpsc};
use tokio_rustls::TlsConnector;
use tokio_rustls::client::TlsStream;

use prost::Message;

//the proto definition of different messages to be sent and decoded
use crate::open_api::{
    ProtoMessage, ProtoOaAccountAuthReq, ProtoOaAccountAuthRes, ProtoOaApplicationAuthReq,
    ProtoOaApplicationAuthRes, ProtoOaGetAccountListByAccessTokenReq,
    ProtoOaGetAccountListByAccessTokenRes, ProtoOaPayloadType, ProtoOaSymbolsListReq,
    ProtoOaSymbolsListRes, ProtoOaTrendbarPeriod, ProtoOaErrorRes, ProtoOaGetTrendbarsReq, ProtoOaGetTrendbarsRes
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
        endpoint: Endpoint,
        access_token: &str,
        client_id: &str,
        client_secret: &str,
    ) -> Result<(Arc<Self>, mpsc::Receiver<StreamEvent>), Box<dyn std::error::Error>> {
        let host = match endpoint {
            Endpoint::Demo => "demo.ctraderapi.com",
            Endpoint::Live => "live.ctraderapi.com",
        };
        let stream_ = stream_builder::initialize_stream(host, 5035)
            .await
            .expect("Failed to initialize the stream.");

        let (event_tx, mut event_rx) = mpsc::channel(100);

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
                let _ = client.event_tx.send(StreamEvent::Error(e.to_string())).await;
            }
        });
    }

    async fn read_loop(&self) -> anyhow::Result<()> {
        loop {
            let msg = self.read_proto_message().await.unwrap();
            self.handle_proto_message(msg).await.unwrap();
        }
    }

    async fn read_proto_message(&self) -> Result<ProtoMessage, Box<dyn std::error::Error>> {
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

        println!("the message is now sent to the reader loop");

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

        println!("the get accounts message is now sent to the reader loop");

        Ok(())
    }

    pub async fn authorize_account(
        &self,
        account_id: i64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("Authorizing account with ID: {}", account_id);
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

        println!("the account authorization message is now sent to the reader loop");

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

        println!("the get symbols message is now sent to the reader loop");

        Ok(())
    }


    pub async fn get_trend_bar_data(
        &self,
        symbol_id: i64,
        period: ProtoOaTrendbarPeriod,
        account_id: i64,
        from_timestamp: i64,
        to_timestamp: i64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("Getting trend bar data...");

        let req = ProtoOaGetTrendbarsReq {
            payload_type: Some(ProtoOaPayloadType::ProtoOaGetTrendbarsReq as i32),
            symbol_id: symbol_id,
            period: period as i32,
            ctid_trader_account_id: account_id,
            from_timestamp: Some(from_timestamp),
            to_timestamp: Some(to_timestamp),
            count: Some(200)
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

        println!("the get trend bar data message is now sent to the reader loop");
        // Implementation for getting historical bar data
        Ok(())
    }


}
