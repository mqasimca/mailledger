//! Secure credential storage using system keyring.
//!
//! Provides secure storage for account passwords using the platform's
//! native credential storage:
//! - Linux: Secret Service (GNOME Keyring, `KWallet`)
//! - macOS: Keychain
//! - Windows: Credential Manager

use keyring::Entry;
use tracing::{debug, warn};

use super::AccountId;

/// Service name used for keyring entries.
const SERVICE_NAME: &str = "mailledger";

/// Credential type identifier for IMAP passwords.
const IMAP_CREDENTIAL: &str = "imap";

/// Credential type identifier for SMTP passwords.
const SMTP_CREDENTIAL: &str = "smtp";

/// Credential type identifier for `OAuth2` tokens.
const OAUTH_TOKEN_CREDENTIAL: &str = "oauth_token";

/// Error type for credential operations.
#[derive(Debug, thiserror::Error)]
pub enum CredentialError {
    /// Failed to access keyring.
    #[error("Keyring error: {0}")]
    Keyring(#[from] keyring::Error),

    /// Account ID is required for credential operations.
    #[error("Account ID is required for credential storage")]
    MissingAccountId,
}

/// Result type for credential operations.
pub type CredentialResult<T> = std::result::Result<T, CredentialError>;

/// Generates the keyring entry key for a credential.
fn credential_key(account_id: AccountId, credential_type: &str) -> String {
    format!("{SERVICE_NAME}_{credential_type}_{}", account_id.0)
}

/// Stores IMAP password securely in the system keyring.
///
/// # Errors
///
/// Returns an error if the keyring operation fails.
pub fn store_imap_password(account_id: AccountId, password: &str) -> CredentialResult<()> {
    let key = credential_key(account_id, IMAP_CREDENTIAL);
    let entry = Entry::new(SERVICE_NAME, &key)?;
    entry.set_password(password)?;
    debug!("Stored IMAP password for account {}", account_id.0);
    Ok(())
}

/// Retrieves IMAP password from the system keyring.
///
/// # Errors
///
/// Returns an error if the keyring operation fails.
pub fn get_imap_password(account_id: AccountId) -> CredentialResult<Option<String>> {
    let key = credential_key(account_id, IMAP_CREDENTIAL);
    let entry = Entry::new(SERVICE_NAME, &key)?;
    match entry.get_password() {
        Ok(password) => Ok(Some(password)),
        Err(keyring::Error::NoEntry) => {
            debug!("No IMAP password found for account {}", account_id.0);
            Ok(None)
        }
        Err(e) => Err(e.into()),
    }
}

/// Stores SMTP password securely in the system keyring.
///
/// # Errors
///
/// Returns an error if the keyring operation fails.
pub fn store_smtp_password(account_id: AccountId, password: &str) -> CredentialResult<()> {
    let key = credential_key(account_id, SMTP_CREDENTIAL);
    let entry = Entry::new(SERVICE_NAME, &key)?;
    entry.set_password(password)?;
    debug!("Stored SMTP password for account {}", account_id.0);
    Ok(())
}

/// Retrieves SMTP password from the system keyring.
///
/// # Errors
///
/// Returns an error if the keyring operation fails.
pub fn get_smtp_password(account_id: AccountId) -> CredentialResult<Option<String>> {
    let key = credential_key(account_id, SMTP_CREDENTIAL);
    let entry = Entry::new(SERVICE_NAME, &key)?;
    match entry.get_password() {
        Ok(password) => Ok(Some(password)),
        Err(keyring::Error::NoEntry) => {
            debug!("No SMTP password found for account {}", account_id.0);
            Ok(None)
        }
        Err(e) => Err(e.into()),
    }
}

/// Deletes all credentials for an account from the keyring.
///
/// This should be called when an account is deleted.
/// Deletes IMAP password, SMTP password, and `OAuth2` token.
///
/// # Errors
///
/// Returns an error if the keyring operation fails (except for missing entries).
pub fn delete_credentials(account_id: AccountId) -> CredentialResult<()> {
    // Delete IMAP password
    let imap_key = credential_key(account_id, IMAP_CREDENTIAL);
    let imap_entry = Entry::new(SERVICE_NAME, &imap_key)?;
    match imap_entry.delete_credential() {
        Ok(()) => debug!("Deleted IMAP password for account {}", account_id.0),
        Err(keyring::Error::NoEntry) => {
            debug!("No IMAP password to delete for account {}", account_id.0);
        }
        Err(e) => {
            warn!("Failed to delete IMAP password: {e}");
            return Err(e.into());
        }
    }

    // Delete SMTP password
    let smtp_key = credential_key(account_id, SMTP_CREDENTIAL);
    let smtp_entry = Entry::new(SERVICE_NAME, &smtp_key)?;
    match smtp_entry.delete_credential() {
        Ok(()) => debug!("Deleted SMTP password for account {}", account_id.0),
        Err(keyring::Error::NoEntry) => {
            debug!("No SMTP password to delete for account {}", account_id.0);
        }
        Err(e) => {
            warn!("Failed to delete SMTP password: {e}");
            return Err(e.into());
        }
    }

    // Delete OAuth2 token
    let oauth_key = credential_key(account_id, OAUTH_TOKEN_CREDENTIAL);
    let oauth_entry = Entry::new(SERVICE_NAME, &oauth_key)?;
    match oauth_entry.delete_credential() {
        Ok(()) => debug!("Deleted OAuth2 token for account {}", account_id.0),
        Err(keyring::Error::NoEntry) => {
            debug!("No OAuth2 token to delete for account {}", account_id.0);
        }
        Err(e) => {
            warn!("Failed to delete OAuth2 token: {e}");
            return Err(e.into());
        }
    }

    Ok(())
}

/// Stores both IMAP and SMTP passwords for an account.
///
/// # Errors
///
/// Returns an error if the account has no ID or keyring operations fail.
pub fn store_account_passwords(
    account_id: Option<AccountId>,
    imap_password: &str,
    smtp_password: &str,
) -> CredentialResult<()> {
    let id = account_id.ok_or(CredentialError::MissingAccountId)?;
    store_imap_password(id, imap_password)?;
    store_smtp_password(id, smtp_password)?;
    Ok(())
}

/// Loads both IMAP and SMTP passwords for an account.
///
/// Returns `(imap_password, smtp_password)` tuple.
///
/// # Errors
///
/// Returns an error if the account has no ID or keyring operations fail.
pub fn load_account_passwords(
    account_id: Option<AccountId>,
) -> CredentialResult<(Option<String>, Option<String>)> {
    let id = account_id.ok_or(CredentialError::MissingAccountId)?;
    let imap = get_imap_password(id)?;
    let smtp = get_smtp_password(id)?;
    Ok((imap, smtp))
}

/// Stores `OAuth2` token securely in the system keyring.
///
/// The token is serialized as JSON for storage.
///
/// # Errors
///
/// Returns an error if the keyring operation fails or serialization fails.
pub fn store_oauth_token(
    account_id: AccountId,
    token: &mailledger_oauth::Token,
) -> CredentialResult<()> {
    let token_json = serde_json::to_string(token)
        .map_err(|e| CredentialError::Keyring(keyring::Error::PlatformFailure(Box::new(e))))?;

    let key = credential_key(account_id, OAUTH_TOKEN_CREDENTIAL);
    let entry = Entry::new(SERVICE_NAME, &key)?;
    entry.set_password(&token_json)?;
    debug!("Stored OAuth2 token for account {}", account_id.0);
    Ok(())
}

/// Retrieves `OAuth2` token from the system keyring.
///
/// # Errors
///
/// Returns an error if the keyring operation fails or deserialization fails.
pub fn get_oauth_token(account_id: AccountId) -> CredentialResult<Option<mailledger_oauth::Token>> {
    let key = credential_key(account_id, OAUTH_TOKEN_CREDENTIAL);
    let entry = Entry::new(SERVICE_NAME, &key)?;
    match entry.get_password() {
        Ok(token_json) => {
            let token = serde_json::from_str(&token_json).map_err(|e| {
                CredentialError::Keyring(keyring::Error::PlatformFailure(Box::new(e)))
            })?;
            Ok(Some(token))
        }
        Err(keyring::Error::NoEntry) => {
            debug!("No OAuth2 token found for account {}", account_id.0);
            Ok(None)
        }
        Err(e) => Err(e.into()),
    }
}

/// Deletes `OAuth2` token for an account from the keyring.
///
/// # Errors
///
/// Returns an error if the keyring operation fails (except for missing entries).
pub fn delete_oauth_token(account_id: AccountId) -> CredentialResult<()> {
    let key = credential_key(account_id, OAUTH_TOKEN_CREDENTIAL);
    let entry = Entry::new(SERVICE_NAME, &key)?;
    match entry.delete_credential() {
        Ok(()) => {
            debug!("Deleted OAuth2 token for account {}", account_id.0);
            Ok(())
        }
        Err(keyring::Error::NoEntry) => {
            debug!("No OAuth2 token to delete for account {}", account_id.0);
            Ok(())
        }
        Err(e) => {
            warn!("Failed to delete OAuth2 token: {e}");
            Err(e.into())
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::redundant_clone, clippy::manual_string_new, clippy::needless_collect, clippy::unreadable_literal, clippy::used_underscore_items, clippy::similar_names)]
mod tests {
    // Note: These tests interact with the actual system keyring.
    // They are marked as ignored by default to avoid polluting the keyring
    // during automated testing. Run manually with `cargo test -- --ignored`

    use super::*;

    #[test]
    #[ignore = "Interacts with system keyring"]
    fn test_store_and_retrieve_imap_password() {
        let account_id = AccountId::new(99999); // Use high ID to avoid conflicts
        let password = "test_imap_password_12345";

        // Store
        store_imap_password(account_id, password).unwrap();

        // Retrieve
        let retrieved = get_imap_password(account_id).unwrap();
        assert_eq!(retrieved, Some(password.to_string()));

        // Cleanup
        delete_credentials(account_id).unwrap();
    }

    #[test]
    #[ignore = "Interacts with system keyring"]
    fn test_store_and_retrieve_smtp_password() {
        let account_id = AccountId::new(99998);
        let password = "test_smtp_password_12345";

        store_smtp_password(account_id, password).unwrap();

        let retrieved = get_smtp_password(account_id).unwrap();
        assert_eq!(retrieved, Some(password.to_string()));

        delete_credentials(account_id).unwrap();
    }

    #[test]
    #[ignore = "Interacts with system keyring"]
    fn test_delete_credentials() {
        let account_id = AccountId::new(99997);

        store_imap_password(account_id, "imap_pass").unwrap();
        store_smtp_password(account_id, "smtp_pass").unwrap();

        delete_credentials(account_id).unwrap();

        assert_eq!(get_imap_password(account_id).unwrap(), None);
        assert_eq!(get_smtp_password(account_id).unwrap(), None);
    }
}
