//! Account management module.
//!
//! Provides account configuration, storage, and validation.

pub mod credentials;
mod model;
mod repository;
mod validation;

pub use credentials::{CredentialError, CredentialResult};
pub use model::{Account, AccountId, ImapConfig, Security, SmtpConfig};
pub use repository::AccountRepository;
pub use validation::{ValidationError, ValidationResult, validate_account};
