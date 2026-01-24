//! Snooze storage repository.

use chrono::{DateTime, Utc};
use sqlx::Row;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

use super::model::SnoozedMessage;
use crate::{AccountId, Result};

/// Repository for snooze storage and retrieval.
pub struct SnoozeRepository {
    pool: SqlitePool,
}

impl SnoozeRepository {
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
    #[allow(dead_code)]
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
            CREATE TABLE IF NOT EXISTS snoozed_messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                account_id INTEGER NOT NULL,
                message_uid INTEGER NOT NULL,
                folder_path TEXT NOT NULL,
                snooze_until TEXT NOT NULL,
                snoozed_at TEXT NOT NULL,
                subject TEXT NOT NULL DEFAULT '',
                from_address TEXT NOT NULL DEFAULT '',
                UNIQUE(account_id, message_uid, folder_path)
            )
            ",
        )
        .execute(&self.pool)
        .await?;

        // Create index for finding expired snoozes
        sqlx::query(
            r"
            CREATE INDEX IF NOT EXISTS idx_snoozed_until ON snoozed_messages(snooze_until)
            ",
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Snooze a message.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn snooze(&self, message: &SnoozedMessage) -> Result<()> {
        sqlx::query(
            r"
            INSERT INTO snoozed_messages
                (account_id, message_uid, folder_path, snooze_until, snoozed_at, subject, from_address)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(account_id, message_uid, folder_path) DO UPDATE SET
                snooze_until = excluded.snooze_until,
                snoozed_at = excluded.snoozed_at
            ",
        )
        .bind(message.account_id.0)
        .bind(message.message_uid)
        .bind(&message.folder_path)
        .bind(message.snooze_until.to_rfc3339())
        .bind(message.snoozed_at.to_rfc3339())
        .bind(&message.subject)
        .bind(&message.from)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get all expired snoozes (ready to be shown again).
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn get_expired(&self) -> Result<Vec<SnoozedMessage>> {
        let now = Utc::now().to_rfc3339();

        let rows = sqlx::query(
            r"
            SELECT account_id, message_uid, folder_path, snooze_until, snoozed_at, subject, from_address
            FROM snoozed_messages
            WHERE snooze_until <= ?
            ORDER BY snooze_until ASC
            ",
        )
        .bind(&now)
        .fetch_all(&self.pool)
        .await?;

        let messages = rows
            .iter()
            .filter_map(|row| {
                let snooze_until_str: String = row.get("snooze_until");
                let snoozed_at_str: String = row.get("snoozed_at");

                let snooze_until = DateTime::parse_from_rfc3339(&snooze_until_str)
                    .ok()?
                    .with_timezone(&Utc);
                let snoozed_at = DateTime::parse_from_rfc3339(&snoozed_at_str)
                    .ok()?
                    .with_timezone(&Utc);

                Some(SnoozedMessage {
                    account_id: AccountId(row.get::<i64, _>("account_id")),
                    message_uid: row.get::<u32, _>("message_uid"),
                    folder_path: row.get("folder_path"),
                    snooze_until,
                    snoozed_at,
                    subject: row.get("subject"),
                    from: row.get("from_address"),
                })
            })
            .collect();

        Ok(messages)
    }

    /// Get all snoozed messages for an account.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn list_for_account(&self, account_id: AccountId) -> Result<Vec<SnoozedMessage>> {
        let rows = sqlx::query(
            r"
            SELECT account_id, message_uid, folder_path, snooze_until, snoozed_at, subject, from_address
            FROM snoozed_messages
            WHERE account_id = ?
            ORDER BY snooze_until ASC
            ",
        )
        .bind(account_id.0)
        .fetch_all(&self.pool)
        .await?;

        let messages = rows
            .iter()
            .filter_map(|row| {
                let snooze_until_str: String = row.get("snooze_until");
                let snoozed_at_str: String = row.get("snoozed_at");

                let snooze_until = DateTime::parse_from_rfc3339(&snooze_until_str)
                    .ok()?
                    .with_timezone(&Utc);
                let snoozed_at = DateTime::parse_from_rfc3339(&snoozed_at_str)
                    .ok()?
                    .with_timezone(&Utc);

                Some(SnoozedMessage {
                    account_id: AccountId(row.get::<i64, _>("account_id")),
                    message_uid: row.get::<u32, _>("message_uid"),
                    folder_path: row.get("folder_path"),
                    snooze_until,
                    snoozed_at,
                    subject: row.get("subject"),
                    from: row.get("from_address"),
                })
            })
            .collect();

        Ok(messages)
    }

    /// Remove a snooze (unsnooze a message).
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn unsnooze(
        &self,
        account_id: AccountId,
        message_uid: u32,
        folder_path: &str,
    ) -> Result<()> {
        sqlx::query(
            r"
            DELETE FROM snoozed_messages
            WHERE account_id = ? AND message_uid = ? AND folder_path = ?
            ",
        )
        .bind(account_id.0)
        .bind(message_uid)
        .bind(folder_path)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Check if a message is currently snoozed.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn is_snoozed(
        &self,
        account_id: AccountId,
        message_uid: u32,
        folder_path: &str,
    ) -> Result<bool> {
        let row = sqlx::query(
            r"
            SELECT COUNT(*) as count
            FROM snoozed_messages
            WHERE account_id = ? AND message_uid = ? AND folder_path = ?
            ",
        )
        .bind(account_id.0)
        .bind(message_uid)
        .bind(folder_path)
        .fetch_one(&self.pool)
        .await?;

        let count: i64 = row.get("count");
        Ok(count > 0)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[tokio::test]
    async fn test_snooze_and_retrieve() {
        let repo = SnoozeRepository::in_memory().await.unwrap();

        let msg = SnoozedMessage::new(
            AccountId(1),
            123,
            "INBOX",
            Utc::now() + Duration::hours(1),
            "Test Subject",
            "sender@example.com",
        );

        repo.snooze(&msg).await.unwrap();

        let snoozed = repo.list_for_account(AccountId(1)).await.unwrap();
        assert_eq!(snoozed.len(), 1);
        assert_eq!(snoozed[0].message_uid, 123);
        assert_eq!(snoozed[0].subject, "Test Subject");
    }

    #[tokio::test]
    async fn test_get_expired() {
        let repo = SnoozeRepository::in_memory().await.unwrap();

        // Create an already expired snooze
        let expired = SnoozedMessage::new(
            AccountId(1),
            100,
            "INBOX",
            Utc::now() - Duration::hours(1),
            "Expired",
            "test@example.com",
        );

        // Create a future snooze
        let future = SnoozedMessage::new(
            AccountId(1),
            200,
            "INBOX",
            Utc::now() + Duration::hours(1),
            "Future",
            "test@example.com",
        );

        repo.snooze(&expired).await.unwrap();
        repo.snooze(&future).await.unwrap();

        let expired_list = repo.get_expired().await.unwrap();
        assert_eq!(expired_list.len(), 1);
        assert_eq!(expired_list[0].message_uid, 100);
    }

    #[tokio::test]
    async fn test_unsnooze() {
        let repo = SnoozeRepository::in_memory().await.unwrap();

        let msg = SnoozedMessage::new(
            AccountId(1),
            123,
            "INBOX",
            Utc::now() + Duration::hours(1),
            "Test",
            "test@example.com",
        );

        repo.snooze(&msg).await.unwrap();
        assert!(repo.is_snoozed(AccountId(1), 123, "INBOX").await.unwrap());

        repo.unsnooze(AccountId(1), 123, "INBOX").await.unwrap();
        assert!(!repo.is_snoozed(AccountId(1), 123, "INBOX").await.unwrap());
    }
}
