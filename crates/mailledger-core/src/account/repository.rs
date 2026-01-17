//! Account storage repository.

use sqlx::Row;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use tracing::{debug, warn};

use super::credentials;
use super::model::{Account, AccountId, ImapConfig, Security, SmtpConfig};
use crate::Result;

/// Repository for account storage and retrieval.
pub struct AccountRepository {
    pool: SqlitePool,
}

impl AccountRepository {
    /// Create a new repository with the given database path.
    ///
    /// Creates the database and tables if they don't exist.
    ///
    /// # Errors
    ///
    /// Returns an error if the database connection fails or schema creation fails.
    pub async fn new(database_path: &str) -> Result<Self> {
        let url = format!("sqlite:{database_path}?mode=rwc");
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&url)
            .await?;

        let repo = Self { pool };
        repo.initialize().await?;
        Ok(repo)
    }

    /// Create an in-memory repository for testing.
    ///
    /// # Errors
    ///
    /// Returns an error if the database connection fails or schema creation fails.
    pub async fn in_memory() -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await?;

        let repo = Self { pool };
        repo.initialize().await?;
        Ok(repo)
    }

    /// Initialize database schema.
    async fn initialize(&self) -> Result<()> {
        sqlx::query(
            r"
            CREATE TABLE IF NOT EXISTS accounts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                email TEXT NOT NULL UNIQUE,
                imap_host TEXT NOT NULL,
                imap_port INTEGER NOT NULL,
                imap_security TEXT NOT NULL,
                imap_username TEXT NOT NULL,
                imap_password TEXT NOT NULL,
                smtp_host TEXT NOT NULL,
                smtp_port INTEGER NOT NULL,
                smtp_security TEXT NOT NULL,
                smtp_username TEXT NOT NULL,
                smtp_password TEXT NOT NULL,
                is_default INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            ",
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get all accounts.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn list(&self) -> Result<Vec<Account>> {
        let rows = sqlx::query(
            r"
            SELECT id, name, email,
                   imap_host, imap_port, imap_security, imap_username, imap_password,
                   smtp_host, smtp_port, smtp_security, smtp_username, smtp_password,
                   is_default
            FROM accounts
            ORDER BY is_default DESC, name ASC
            ",
        )
        .fetch_all(&self.pool)
        .await?;

        let accounts = rows.iter().map(row_to_account).collect();
        Ok(accounts)
    }

    /// Get account by ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn get(&self, id: AccountId) -> Result<Option<Account>> {
        let row = sqlx::query(
            r"
            SELECT id, name, email,
                   imap_host, imap_port, imap_security, imap_username, imap_password,
                   smtp_host, smtp_port, smtp_security, smtp_username, smtp_password,
                   is_default
            FROM accounts
            WHERE id = ?
            ",
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.as_ref().map(row_to_account))
    }

    /// Get the default account.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn get_default(&self) -> Result<Option<Account>> {
        let row = sqlx::query(
            r"
            SELECT id, name, email,
                   imap_host, imap_port, imap_security, imap_username, imap_password,
                   smtp_host, smtp_port, smtp_security, smtp_username, smtp_password,
                   is_default
            FROM accounts
            WHERE is_default = 1
            LIMIT 1
            ",
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.as_ref().map(row_to_account))
    }

    /// Save an account (insert or update).
    ///
    /// Passwords are stored securely in the system keyring.
    /// The database stores placeholder values for password fields.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn save(&self, account: &mut Account) -> Result<()> {
        // Store empty strings in DB - passwords go to keyring
        let db_password_placeholder = "";

        if let Some(id) = account.id {
            // Update existing
            sqlx::query(
                r"
                UPDATE accounts SET
                    name = ?, email = ?,
                    imap_host = ?, imap_port = ?, imap_security = ?,
                    imap_username = ?, imap_password = ?,
                    smtp_host = ?, smtp_port = ?, smtp_security = ?,
                    smtp_username = ?, smtp_password = ?,
                    is_default = ?,
                    updated_at = CURRENT_TIMESTAMP
                WHERE id = ?
                ",
            )
            .bind(&account.name)
            .bind(&account.email)
            .bind(&account.imap.host)
            .bind(i64::from(account.imap.port))
            .bind(security_to_string(account.imap.security))
            .bind(&account.imap.username)
            .bind(db_password_placeholder)
            .bind(&account.smtp.host)
            .bind(i64::from(account.smtp.port))
            .bind(security_to_string(account.smtp.security))
            .bind(&account.smtp.username)
            .bind(db_password_placeholder)
            .bind(account.is_default)
            .bind(id.0)
            .execute(&self.pool)
            .await?;

            // Store passwords in keyring
            store_passwords_in_keyring(id, &account.imap.password, &account.smtp.password)?;
        } else {
            // Insert new
            let result = sqlx::query(
                r"
                INSERT INTO accounts (
                    name, email,
                    imap_host, imap_port, imap_security, imap_username, imap_password,
                    smtp_host, smtp_port, smtp_security, smtp_username, smtp_password,
                    is_default
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                ",
            )
            .bind(&account.name)
            .bind(&account.email)
            .bind(&account.imap.host)
            .bind(i64::from(account.imap.port))
            .bind(security_to_string(account.imap.security))
            .bind(&account.imap.username)
            .bind(db_password_placeholder)
            .bind(&account.smtp.host)
            .bind(i64::from(account.smtp.port))
            .bind(security_to_string(account.smtp.security))
            .bind(&account.smtp.username)
            .bind(db_password_placeholder)
            .bind(account.is_default)
            .execute(&self.pool)
            .await?;

            let new_id = AccountId::new(result.last_insert_rowid());
            account.id = Some(new_id);

            // Store passwords in keyring
            store_passwords_in_keyring(new_id, &account.imap.password, &account.smtp.password)?;
        }

        // If this account is default, unset others
        if account.is_default
            && let Some(id) = account.id
        {
            sqlx::query("UPDATE accounts SET is_default = 0 WHERE id != ?")
                .bind(id.0)
                .execute(&self.pool)
                .await?;
        }

        Ok(())
    }

    /// Delete an account.
    ///
    /// Also removes credentials from the system keyring.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn delete(&self, id: AccountId) -> Result<()> {
        sqlx::query("DELETE FROM accounts WHERE id = ?")
            .bind(id.0)
            .execute(&self.pool)
            .await?;

        // Delete credentials from keyring
        if let Err(e) = credentials::delete_credentials(id) {
            warn!("Failed to delete credentials from keyring: {e}");
        }

        Ok(())
    }
}

/// Convert a database row to an Account.
///
/// Loads passwords from the system keyring first, falling back to database
/// values for backward compatibility with existing accounts.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn row_to_account(row: &sqlx::sqlite::SqliteRow) -> Account {
    let id = AccountId::new(row.get("id"));

    // Try to load passwords from keyring, fall back to DB for migration
    let (imap_password, smtp_password) = load_passwords_from_keyring(id, row);

    Account {
        id: Some(id),
        name: row.get("name"),
        email: row.get("email"),
        imap: ImapConfig {
            host: row.get("imap_host"),
            port: row.get::<i64, _>("imap_port") as u16,
            security: string_to_security(row.get("imap_security")),
            username: row.get("imap_username"),
            password: imap_password,
        },
        smtp: SmtpConfig {
            host: row.get("smtp_host"),
            port: row.get::<i64, _>("smtp_port") as u16,
            security: string_to_security(row.get("smtp_security")),
            username: row.get("smtp_username"),
            password: smtp_password,
        },
        is_default: row.get::<i64, _>("is_default") != 0,
    }
}

/// Store passwords securely in the system keyring.
///
/// # Errors
///
/// Returns an error if storing either password fails.
fn store_passwords_in_keyring(
    account_id: AccountId,
    imap_password: &str,
    smtp_password: &str,
) -> crate::Result<()> {
    credentials::store_imap_password(account_id, imap_password)?;
    credentials::store_smtp_password(account_id, smtp_password)?;
    debug!("Stored credentials in keyring for account {}", account_id.0);
    Ok(())
}

/// Load passwords from keyring with fallback to database.
fn load_passwords_from_keyring(
    account_id: AccountId,
    row: &sqlx::sqlite::SqliteRow,
) -> (String, String) {
    // Try keyring first
    let imap_password = match credentials::get_imap_password(account_id) {
        Ok(Some(pass)) => {
            debug!(
                "Loaded IMAP password from keyring for account {}",
                account_id.0
            );
            pass
        }
        Ok(None) => {
            // Fall back to database (for migration)
            let db_pass: String = row.get("imap_password");
            if !db_pass.is_empty() {
                debug!(
                    "Using IMAP password from database for account {} (migration needed)",
                    account_id.0
                );
            }
            db_pass
        }
        Err(e) => {
            warn!("Failed to load IMAP password from keyring: {e}");
            row.get("imap_password")
        }
    };

    let smtp_password = match credentials::get_smtp_password(account_id) {
        Ok(Some(pass)) => {
            debug!(
                "Loaded SMTP password from keyring for account {}",
                account_id.0
            );
            pass
        }
        Ok(None) => {
            let db_pass: String = row.get("smtp_password");
            if !db_pass.is_empty() {
                debug!(
                    "Using SMTP password from database for account {} (migration needed)",
                    account_id.0
                );
            }
            db_pass
        }
        Err(e) => {
            warn!("Failed to load SMTP password from keyring: {e}");
            row.get("smtp_password")
        }
    };

    (imap_password, smtp_password)
}

const fn security_to_string(security: Security) -> &'static str {
    match security {
        Security::None => "none",
        Security::Tls => "tls",
        Security::StartTls => "starttls",
    }
}

fn string_to_security(s: &str) -> Security {
    match s {
        "none" => Security::None,
        "starttls" => Security::StartTls,
        _ => Security::Tls,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::redundant_clone, clippy::manual_string_new, clippy::needless_collect, clippy::unreadable_literal, clippy::used_underscore_items, clippy::similar_names)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_and_retrieve_account() {
        let repo = AccountRepository::in_memory().await.unwrap();

        let mut account = Account::with_email("test@example.com");
        account.imap.password = "secret".to_string();
        account.smtp.password = "secret".to_string();

        repo.save(&mut account).await.unwrap();
        assert!(account.id.is_some());

        let retrieved = repo.get(account.id.unwrap()).await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.email, "test@example.com");
    }

    #[tokio::test]
    async fn test_list_accounts() {
        let repo = AccountRepository::in_memory().await.unwrap();

        let mut account1 = Account::with_email("user1@example.com");
        account1.imap.password = "secret".to_string();
        account1.smtp.password = "secret".to_string();
        repo.save(&mut account1).await.unwrap();

        let mut account2 = Account::with_email("user2@example.com");
        account2.imap.password = "secret".to_string();
        account2.smtp.password = "secret".to_string();
        repo.save(&mut account2).await.unwrap();

        let accounts = repo.list().await.unwrap();
        assert_eq!(accounts.len(), 2);
    }

    #[tokio::test]
    async fn test_default_account() {
        let repo = AccountRepository::in_memory().await.unwrap();

        let mut account = Account::with_email("default@example.com");
        account.imap.password = "secret".to_string();
        account.smtp.password = "secret".to_string();
        account.is_default = true;
        repo.save(&mut account).await.unwrap();

        let default = repo.get_default().await.unwrap();
        assert!(default.is_some());
        assert_eq!(default.unwrap().email, "default@example.com");
    }
}
