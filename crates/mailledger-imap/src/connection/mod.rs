//! IMAP connection management.
//!
//! This module provides connection handling for IMAP servers, including:
//! - Configuration (host, port, security mode)
//! - TLS/plaintext stream abstraction
//! - Framed I/O for IMAP protocol
//! - Type-state connection wrapper
//! - IDLE support for real-time notifications
//! - High-level session with auto-reconnect

mod client;
mod config;
mod framed;
mod idle;
mod session;
mod stream;

pub use client::{Authenticated, Client, NotAuthenticated, Selected};
pub use config::{Config, ConfigBuilder, Security};
pub use framed::{FramedStream, ResponseAccumulator};
pub use idle::{IdleEvent, IdleHandle};
pub use session::{Session, SessionConfig};
pub use stream::{ImapStream, connect_plain, connect_tls, create_tls_connector};
