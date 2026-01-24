//! Message cache storage repository.

use chrono::{DateTime, Utc};
use sqlx::Row;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

use super::model::{CachedMessageContent, CachedMessageSummary};
use crate::{AccountId, Result};

/// Repository for message cache storage and retrieval.
pub struct CacheRepository {
    pool: SqlitePool,
}

impl CacheRepository {
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
        // Message summaries table (for list view)
        sqlx::query(
            r"
            CREATE TABLE IF NOT EXISTS cached_message_summaries (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                account_id INTEGER NOT NULL,
                folder_path TEXT NOT NULL,
                uid INTEGER NOT NULL,
                from_name TEXT NOT NULL DEFAULT '',
                from_email TEXT NOT NULL DEFAULT '',
                subject TEXT NOT NULL DEFAULT '',
                snippet TEXT NOT NULL DEFAULT '',
                date TEXT NOT NULL DEFAULT '',
                is_read INTEGER NOT NULL DEFAULT 0,
                is_flagged INTEGER NOT NULL DEFAULT 0,
                has_attachments INTEGER NOT NULL DEFAULT 0,
                cached_at TEXT NOT NULL,
                UNIQUE(account_id, folder_path, uid)
            )
            ",
        )
        .execute(&self.pool)
        .await?;

        // Message content table (for viewing)
        sqlx::query(
            r"
            CREATE TABLE IF NOT EXISTS cached_message_content (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                account_id INTEGER NOT NULL,
                folder_path TEXT NOT NULL,
                uid INTEGER NOT NULL,
                from_full TEXT NOT NULL DEFAULT '',
                to_recipients TEXT NOT NULL DEFAULT '',
                cc_recipients TEXT NOT NULL DEFAULT '',
                subject TEXT NOT NULL DEFAULT '',
                date TEXT NOT NULL DEFAULT '',
                body_text TEXT,
                body_html TEXT,
                attachments_json TEXT,
                cached_at TEXT NOT NULL,
                UNIQUE(account_id, folder_path, uid)
            )
            ",
        )
        .execute(&self.pool)
        .await?;

        // Indexes for efficient lookups
        sqlx::query(
            r"
            CREATE INDEX IF NOT EXISTS idx_summaries_folder
            ON cached_message_summaries(account_id, folder_path)
            ",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r"
            CREATE INDEX IF NOT EXISTS idx_content_lookup
            ON cached_message_content(account_id, folder_path, uid)
            ",
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Cache a message summary.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn cache_summary(&self, summary: &CachedMessageSummary) -> Result<()> {
        sqlx::query(
            r"
            INSERT INTO cached_message_summaries
                (account_id, folder_path, uid, from_name, from_email, subject, snippet,
                 date, is_read, is_flagged, has_attachments, cached_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(account_id, folder_path, uid) DO UPDATE SET
                from_name = excluded.from_name,
                from_email = excluded.from_email,
                subject = excluded.subject,
                snippet = excluded.snippet,
                date = excluded.date,
                is_read = excluded.is_read,
                is_flagged = excluded.is_flagged,
                has_attachments = excluded.has_attachments,
                cached_at = excluded.cached_at
            ",
        )
        .bind(summary.account_id.0)
        .bind(&summary.folder_path)
        .bind(summary.uid)
        .bind(&summary.from_name)
        .bind(&summary.from_email)
        .bind(&summary.subject)
        .bind(&summary.snippet)
        .bind(&summary.date)
        .bind(summary.is_read)
        .bind(summary.is_flagged)
        .bind(summary.has_attachments)
        .bind(summary.cached_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Cache multiple message summaries in a batch.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn cache_summaries(&self, summaries: &[CachedMessageSummary]) -> Result<()> {
        for summary in summaries {
            self.cache_summary(summary).await?;
        }
        Ok(())
    }

    /// Get cached summaries for a folder.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn get_summaries(
        &self,
        account_id: AccountId,
        folder_path: &str,
    ) -> Result<Vec<CachedMessageSummary>> {
        let rows = sqlx::query(
            r"
            SELECT account_id, folder_path, uid, from_name, from_email, subject, snippet,
                   date, is_read, is_flagged, has_attachments, cached_at
            FROM cached_message_summaries
            WHERE account_id = ? AND folder_path = ?
            ORDER BY uid DESC
            ",
        )
        .bind(account_id.0)
        .bind(folder_path)
        .fetch_all(&self.pool)
        .await?;

        let summaries = rows
            .iter()
            .filter_map(|row| {
                let cached_at_str: String = row.get("cached_at");
                let cached_at = DateTime::parse_from_rfc3339(&cached_at_str)
                    .ok()?
                    .with_timezone(&Utc);

                Some(CachedMessageSummary {
                    account_id: AccountId(row.get::<i64, _>("account_id")),
                    folder_path: row.get("folder_path"),
                    uid: row.get::<u32, _>("uid"),
                    from_name: row.get("from_name"),
                    from_email: row.get("from_email"),
                    subject: row.get("subject"),
                    snippet: row.get("snippet"),
                    date: row.get("date"),
                    is_read: row.get::<bool, _>("is_read"),
                    is_flagged: row.get::<bool, _>("is_flagged"),
                    has_attachments: row.get::<bool, _>("has_attachments"),
                    cached_at,
                })
            })
            .collect();

        Ok(summaries)
    }

    /// Cache message content.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn cache_content(&self, content: &CachedMessageContent) -> Result<()> {
        sqlx::query(
            r"
            INSERT INTO cached_message_content
                (account_id, folder_path, uid, from_full, to_recipients, cc_recipients,
                 subject, date, body_text, body_html, attachments_json, cached_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(account_id, folder_path, uid) DO UPDATE SET
                from_full = excluded.from_full,
                to_recipients = excluded.to_recipients,
                cc_recipients = excluded.cc_recipients,
                subject = excluded.subject,
                date = excluded.date,
                body_text = excluded.body_text,
                body_html = excluded.body_html,
                attachments_json = excluded.attachments_json,
                cached_at = excluded.cached_at
            ",
        )
        .bind(content.account_id.0)
        .bind(&content.folder_path)
        .bind(content.uid)
        .bind(&content.from)
        .bind(&content.to)
        .bind(&content.cc)
        .bind(&content.subject)
        .bind(&content.date)
        .bind(&content.body_text)
        .bind(&content.body_html)
        .bind(&content.attachments_json)
        .bind(content.cached_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get cached content for a message.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn get_content(
        &self,
        account_id: AccountId,
        folder_path: &str,
        uid: u32,
    ) -> Result<Option<CachedMessageContent>> {
        let row = sqlx::query(
            r"
            SELECT account_id, folder_path, uid, from_full, to_recipients, cc_recipients,
                   subject, date, body_text, body_html, attachments_json, cached_at
            FROM cached_message_content
            WHERE account_id = ? AND folder_path = ? AND uid = ?
            ",
        )
        .bind(account_id.0)
        .bind(folder_path)
        .bind(uid)
        .fetch_optional(&self.pool)
        .await?;

        let content = row.and_then(|row| {
            let cached_at_str: String = row.get("cached_at");
            let cached_at = DateTime::parse_from_rfc3339(&cached_at_str)
                .ok()?
                .with_timezone(&Utc);

            Some(CachedMessageContent {
                account_id: AccountId(row.get::<i64, _>("account_id")),
                folder_path: row.get("folder_path"),
                uid: row.get::<u32, _>("uid"),
                from: row.get("from_full"),
                to: row.get("to_recipients"),
                cc: row.get("cc_recipients"),
                subject: row.get("subject"),
                date: row.get("date"),
                body_text: row.get("body_text"),
                body_html: row.get("body_html"),
                attachments_json: row.get("attachments_json"),
                cached_at,
            })
        });

        Ok(content)
    }

    /// Clear cache for a specific folder.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn clear_folder(&self, account_id: AccountId, folder_path: &str) -> Result<()> {
        sqlx::query(
            r"DELETE FROM cached_message_summaries WHERE account_id = ? AND folder_path = ?",
        )
        .bind(account_id.0)
        .bind(folder_path)
        .execute(&self.pool)
        .await?;

        sqlx::query(r"DELETE FROM cached_message_content WHERE account_id = ? AND folder_path = ?")
            .bind(account_id.0)
            .bind(folder_path)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Clear all cache for an account.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn clear_account(&self, account_id: AccountId) -> Result<()> {
        sqlx::query(r"DELETE FROM cached_message_summaries WHERE account_id = ?")
            .bind(account_id.0)
            .execute(&self.pool)
            .await?;

        sqlx::query(r"DELETE FROM cached_message_content WHERE account_id = ?")
            .bind(account_id.0)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Update message flags in cache (read/flagged status).
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn update_flags(
        &self,
        account_id: AccountId,
        folder_path: &str,
        uid: u32,
        is_read: bool,
        is_flagged: bool,
    ) -> Result<()> {
        sqlx::query(
            r"
            UPDATE cached_message_summaries
            SET is_read = ?, is_flagged = ?
            WHERE account_id = ? AND folder_path = ? AND uid = ?
            ",
        )
        .bind(is_read)
        .bind(is_flagged)
        .bind(account_id.0)
        .bind(folder_path)
        .bind(uid)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Check if we have cached data for a folder.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn has_cached_folder(
        &self,
        account_id: AccountId,
        folder_path: &str,
    ) -> Result<bool> {
        let row = sqlx::query(
            r"
            SELECT COUNT(*) as count
            FROM cached_message_summaries
            WHERE account_id = ? AND folder_path = ?
            ",
        )
        .bind(account_id.0)
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

    #[tokio::test]
    async fn test_cache_and_retrieve_summary() {
        let repo = CacheRepository::in_memory().await.unwrap();

        let summary = CachedMessageSummary {
            account_id: AccountId(1),
            folder_path: "INBOX".to_string(),
            uid: 123,
            from_name: "John Doe".to_string(),
            from_email: "john@example.com".to_string(),
            subject: "Test Subject".to_string(),
            snippet: "This is a test...".to_string(),
            date: "Jan 24".to_string(),
            is_read: false,
            is_flagged: true,
            has_attachments: false,
            cached_at: Utc::now(),
        };

        repo.cache_summary(&summary).await.unwrap();

        let summaries = repo.get_summaries(AccountId(1), "INBOX").await.unwrap();
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].uid, 123);
        assert_eq!(summaries[0].subject, "Test Subject");
        assert!(!summaries[0].is_read);
        assert!(summaries[0].is_flagged);
    }

    #[tokio::test]
    async fn test_cache_and_retrieve_content() {
        let repo = CacheRepository::in_memory().await.unwrap();

        let content = CachedMessageContent {
            account_id: AccountId(1),
            folder_path: "INBOX".to_string(),
            uid: 123,
            from: "John Doe <john@example.com>".to_string(),
            to: "me@example.com".to_string(),
            cc: "".to_string(),
            subject: "Test Subject".to_string(),
            date: "Fri, 24 Jan 2026 10:00:00 +0000".to_string(),
            body_text: Some("Hello, this is the message body.".to_string()),
            body_html: Some("<p>Hello, this is the message body.</p>".to_string()),
            attachments_json: None,
            cached_at: Utc::now(),
        };

        repo.cache_content(&content).await.unwrap();

        let retrieved = repo.get_content(AccountId(1), "INBOX", 123).await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.subject, "Test Subject");
        assert_eq!(
            retrieved.body_text,
            Some("Hello, this is the message body.".to_string())
        );
    }

    #[tokio::test]
    async fn test_update_flags() {
        let repo = CacheRepository::in_memory().await.unwrap();

        let summary = CachedMessageSummary {
            account_id: AccountId(1),
            folder_path: "INBOX".to_string(),
            uid: 123,
            from_name: "Test".to_string(),
            from_email: "test@example.com".to_string(),
            subject: "Test".to_string(),
            snippet: "...".to_string(),
            date: "Jan 24".to_string(),
            is_read: false,
            is_flagged: false,
            has_attachments: false,
            cached_at: Utc::now(),
        };

        repo.cache_summary(&summary).await.unwrap();

        // Update flags
        repo.update_flags(AccountId(1), "INBOX", 123, true, true)
            .await
            .unwrap();

        let summaries = repo.get_summaries(AccountId(1), "INBOX").await.unwrap();
        assert!(summaries[0].is_read);
        assert!(summaries[0].is_flagged);
    }

    #[tokio::test]
    async fn test_clear_folder() {
        let repo = CacheRepository::in_memory().await.unwrap();

        let summary = CachedMessageSummary {
            account_id: AccountId(1),
            folder_path: "INBOX".to_string(),
            uid: 123,
            from_name: "Test".to_string(),
            from_email: "test@example.com".to_string(),
            subject: "Test".to_string(),
            snippet: "...".to_string(),
            date: "Jan 24".to_string(),
            is_read: false,
            is_flagged: false,
            has_attachments: false,
            cached_at: Utc::now(),
        };

        repo.cache_summary(&summary).await.unwrap();
        assert!(repo.has_cached_folder(AccountId(1), "INBOX").await.unwrap());

        repo.clear_folder(AccountId(1), "INBOX").await.unwrap();
        assert!(!repo.has_cached_folder(AccountId(1), "INBOX").await.unwrap());
    }
}
