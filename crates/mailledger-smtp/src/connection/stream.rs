//! Low-level SMTP stream handling.

use crate::error::{Error, Result};
use rustls::pki_types::ServerName;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio_rustls::{
    TlsConnector,
    rustls::{ClientConfig, RootCertStore},
};

/// SMTP stream (TCP or TLS).
#[derive(Debug)]
pub enum SmtpStream {
    /// Plain TCP connection.
    Tcp(BufReader<TcpStream>),
    /// TLS-encrypted connection.
    Tls(Box<BufReader<tokio_rustls::client::TlsStream<TcpStream>>>),
}

impl SmtpStream {
    /// Reads a line from the stream.
    ///
    /// # Errors
    ///
    /// Returns an error if the read fails.
    pub async fn read_line(&mut self) -> Result<String> {
        let mut line = String::new();
        match self {
            Self::Tcp(reader) => {
                reader.read_line(&mut line).await?;
            }
            Self::Tls(reader) => {
                reader.read_line(&mut line).await?;
            }
        }
        Ok(line.trim_end().to_string())
    }

    /// Writes data to the stream.
    ///
    /// # Errors
    ///
    /// Returns an error if the write fails.
    pub async fn write_all(&mut self, data: &[u8]) -> Result<()> {
        match self {
            Self::Tcp(reader) => {
                reader.get_mut().write_all(data).await?;
                reader.get_mut().flush().await?;
            }
            Self::Tls(reader) => {
                reader.get_mut().write_all(data).await?;
                reader.get_mut().flush().await?;
            }
        }
        Ok(())
    }

    /// Upgrades a TCP stream to TLS.
    ///
    /// # Errors
    ///
    /// Returns an error if the TLS handshake fails.
    pub async fn upgrade_to_tls(self, hostname: &str) -> Result<Self> {
        let tcp_stream = match self {
            Self::Tcp(reader) => reader.into_inner(),
            Self::Tls(_) => return Err(Error::Protocol("Already using TLS".into())),
        };

        let connector = create_tls_connector();
        let server_name = ServerName::try_from(hostname.to_string())
            .map_err(|_| Error::Protocol(format!("Invalid hostname: {hostname}")))?;

        let tls_stream = connector.connect(server_name, tcp_stream).await?;
        Ok(Self::Tls(Box::new(BufReader::new(tls_stream))))
    }
}

/// Connects to an SMTP server over plain TCP.
///
/// # Errors
///
/// Returns an error if the connection fails.
pub async fn connect(hostname: &str, port: u16) -> Result<SmtpStream> {
    let addr = format!("{hostname}:{port}");
    let stream = TcpStream::connect(&addr).await?;
    Ok(SmtpStream::Tcp(BufReader::new(stream)))
}

/// Connects to an SMTP server over TLS (implicit TLS on port 465).
///
/// # Errors
///
/// Returns an error if the connection or TLS handshake fails.
pub async fn connect_tls(hostname: &str, port: u16) -> Result<SmtpStream> {
    let addr = format!("{hostname}:{port}");
    let tcp_stream = TcpStream::connect(&addr).await?;

    let connector = create_tls_connector();
    let server_name = ServerName::try_from(hostname.to_string())
        .map_err(|_| Error::Protocol(format!("Invalid hostname: {hostname}")))?;

    let tls_stream = connector.connect(server_name, tcp_stream).await?;
    Ok(SmtpStream::Tls(Box::new(BufReader::new(tls_stream))))
}

/// Creates a TLS connector with system root certificates.
fn create_tls_connector() -> TlsConnector {
    let root_store = RootCertStore {
        roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
    };

    let config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    TlsConnector::from(Arc::new(config))
}
