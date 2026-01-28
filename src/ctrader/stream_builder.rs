use tokio_rustls::rustls::{ClientConfig, RootCertStore};
use tokio_rustls::TlsConnector;
use rustls_pki_types::ServerName;
use tokio::net::TcpStream;
use std::sync::Arc;



fn build_tls_config() -> ClientConfig{
    let mut root_cert_store = RootCertStore::empty();
    root_cert_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let config = ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();

    config
}

pub async fn initialize_stream(host: &str, port: i64) -> Result<tokio_rustls::client::TlsStream<TcpStream>, Box<dyn std::error::Error>> {
    let config = build_tls_config();
    let tls_connector = TlsConnector::from(Arc::new(config));    
    let server_name = ServerName::try_from(String::from(host))?;
    let stream = TcpStream::connect((String::from(host), port as u16)).await?;
    let tls_stream = tls_connector.connect(server_name, stream).await?; 

    Ok(tls_stream)
}
