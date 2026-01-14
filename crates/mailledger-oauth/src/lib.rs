//! # mailledger-oauth
//!
//! `OAuth2` authentication library for email protocols (IMAP/SMTP).
//!
//! ## Features
//!
//! - **Authorization flows**: Authorization Code Flow (with PKCE) and Device Flow
//! - **Token management**: Automatic refresh, expiration checking
//! - **Provider configurations**: Pre-configured for Gmail, Outlook, Yahoo
//! - **SASL mechanisms**: OAUTHBEARER (RFC 7628) and XOAUTH2 (proprietary)
//!
//! ## Quick Start
//!
//! ### Authorization Code Flow (Desktop/Web Apps)
//!
//! ```ignore
//! use mailledger_oauth::{Provider, OAuthClient, AuthorizationCodeFlow};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Configure for Gmail
//!     let provider = Provider::google()?;
//!     let client = OAuthClient::new("your_client_id", provider)
//!         .with_client_secret("your_secret")
//!         .with_redirect_uri("http://localhost:8080");
//!
//!     // Create flow with PKCE for security
//!     let flow = AuthorizationCodeFlow::new(client).with_pkce();
//!
//!     // Generate authorization URL
//!     let auth_url = flow.authorization_url(None, Some("random_state"))?;
//!     println!("Visit: {}", auth_url);
//!
//!     // After user authorizes, exchange code for token
//!     let code = "authorization_code_from_redirect";
//!     let token = flow.exchange_code(code, None).await?;
//!
//!     println!("Access token: {}", token.access_token);
//!     Ok(())
//! }
//! ```
//!
//! ### Device Flow (CLI/IoT Apps)
//!
//! ```ignore
//! use mailledger_oauth::{Provider, OAuthClient, DeviceFlow};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let provider = Provider::google()?;
//!     let client = OAuthClient::new("your_client_id", provider);
//!     let flow = DeviceFlow::new(client);
//!
//!     // Request device authorization
//!     let auth = flow.request_device_authorization(None).await?;
//!
//!     println!("Visit: {}", auth.verification_uri);
//!     println!("Enter code: {}", auth.user_code);
//!
//!     // Poll for token (with automatic retry)
//!     let (_, token) = flow.authorize(None, 120).await?;
//!     println!("Authorized! Token: {}", token.access_token);
//!     Ok(())
//! }
//! ```
//!
//! ### Using with IMAP/SMTP
//!
//! ```ignore
//! use mailledger_oauth::sasl::{oauthbearer_response, xoauth2_response};
//!
//! // OAUTHBEARER (RFC 7628 standard)
//! let auth_string = oauthbearer_response("user@gmail.com", &token.access_token);
//! // Send: AUTHENTICATE OAUTHBEARER {auth_string}
//!
//! // XOAUTH2 (Google/Microsoft proprietary)
//! let auth_string = xoauth2_response("user@gmail.com", &token.access_token);
//! // Send: AUTHENTICATE XOAUTH2 {auth_string}
//! ```
//!
//! ### Token Refresh
//!
//! ```ignore
//! // Check if token needs refresh
//! if token.is_expired() {
//!     let new_token = client.refresh_token(&token).await?;
//!     // Use new_token
//! }
//! ```
//!
//! ## Provider Support
//!
//! - **Gmail** - Full support with `https://mail.google.com/` scope
//! - **Outlook/Microsoft** - Full support with IMAP/SMTP scopes
//! - **Yahoo** - Full support with `mail-r` and `mail-w` scopes
//! - **Custom** - Configure any `OAuth2` provider

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

mod error;
pub mod flow;
pub mod provider;
pub mod sasl;
pub mod token;

pub use error::{Error, Result};
pub use flow::{AuthorizationCodeFlow, DeviceFlow, OAuthClient, PkceChallenge};
pub use provider::Provider;
pub use token::Token;
