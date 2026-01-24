//! Stream types for IMAP connections.

#![allow(clippy::missing_errors_doc)]

use std::io;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use rustls::pki_types::ServerName;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use tokio_rustls::client::TlsStream;

use crate::{Error, Result};

/// A stream that can be either plaintext or TLS.
pub enum ImapStream {
    /// Plaintext TCP stream.
    Plain(TcpStream),
    /// TLS-encrypted stream (boxed to reduce enum size).
    Tls(Box<TlsStream<TcpStream>>),
}

impl ImapStream {
    /// Creates a new plaintext stream.
    pub const fn plain(stream: TcpStream) -> Self {
        Self::Plain(stream)
    }

    /// Creates a new TLS stream.
    pub fn tls(stream: TlsStream<TcpStream>) -> Self {
        Self::Tls(Box::new(stream))
    }

    /// Upgrades a plaintext stream to TLS using STARTTLS.
    pub async fn upgrade_to_tls(self, host: &str) -> Result<Self> {
        match self {
            Self::Plain(tcp) => {
                let connector = create_tls_connector()?;
                let server_name = ServerName::try_from(host.to_string())?;
                let tls = connector.connect(server_name, tcp).await?;
                Ok(Self::Tls(Box::new(tls)))
            }
            Self::Tls(_) => Err(Error::InvalidState("Stream is already TLS".to_string())),
        }
    }

    /// Returns true if the stream is TLS-encrypted.
    #[must_use]
    pub const fn is_tls(&self) -> bool {
        matches!(self, Self::Tls(_))
    }
}

impl AsyncRead for ImapStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        match self.get_mut() {
            Self::Plain(stream) => Pin::new(stream).poll_read(cx, buf),
            Self::Tls(stream) => Pin::new(stream).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for ImapStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        match self.get_mut() {
            Self::Plain(stream) => Pin::new(stream).poll_write(cx, buf),
            Self::Tls(stream) => Pin::new(stream).poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.get_mut() {
            Self::Plain(stream) => Pin::new(stream).poll_flush(cx),
            Self::Tls(stream) => Pin::new(stream).poll_flush(cx),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.get_mut() {
            Self::Plain(stream) => Pin::new(stream).poll_shutdown(cx),
            Self::Tls(stream) => Pin::new(stream).poll_shutdown(cx),
        }
    }
}

/// Creates a TLS connector with default root certificates.
pub fn create_tls_connector() -> Result<TlsConnector> {
    let root_store = rustls::RootCertStore {
        roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
    };

    let config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    Ok(TlsConnector::from(Arc::new(config)))
}

/// Connects to a server with TLS from the start.
pub async fn connect_tls(host: &str, port: u16) -> Result<ImapStream> {
    let addr = format!("{host}:{port}");
    let tcp = TcpStream::connect(&addr).await?;

    let connector = create_tls_connector()?;
    let server_name = ServerName::try_from(host.to_string())?;
    let tls = connector.connect(server_name, tcp).await?;

    Ok(ImapStream::Tls(Box::new(tls)))
}

/// Connects to a server without TLS (for STARTTLS or testing).
pub async fn connect_plain(host: &str, port: u16) -> Result<ImapStream> {
    let addr = format!("{host}:{port}");
    let tcp = TcpStream::connect(&addr).await?;
    Ok(ImapStream::Plain(tcp))
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::redundant_clone,
    clippy::manual_string_new,
    clippy::needless_collect,
    clippy::unreadable_literal,
    clippy::used_underscore_items,
    clippy::similar_names
)]
mod tests {
    use super::*;

    #[test]
    fn test_create_tls_connector() {
        let connector = create_tls_connector();
        assert!(connector.is_ok());
    }
}
