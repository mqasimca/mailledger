//! Triage repository for persistent storage of sender decisions.

use sqlx::Row;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

use super::model::{InboxCategory, ScreenedSender, SenderDecision};
use crate::Result;
use crate::account::AccountId;

/// Repository for triage data (screened senders and categories).
pub struct TriageRepository {
    pool: SqlitePool,
}

impl TriageRepository {
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
            CREATE TABLE IF NOT EXISTS screened_senders (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                account_id INTEGER NOT NULL,
                email TEXT NOT NULL,
                display_name TEXT,
                decision TEXT NOT NULL DEFAULT 'pending',
                category TEXT NOT NULL DEFAULT 'imbox',
                note TEXT,
                email_count INTEGER NOT NULL DEFAULT 1,
                first_seen TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                last_seen TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(account_id, email)
            )
            ",
        )
        .execute(&self.pool)
        .await?;

        // Index for fast lookups
        sqlx::query(
            r"
            CREATE INDEX IF NOT EXISTS idx_screened_senders_account_email 
            ON screened_senders(account_id, email)
            ",
        )
        .execute(&self.pool)
        .await?;

        // Index for listing pending senders
        sqlx::query(
            r"
            CREATE INDEX IF NOT EXISTS idx_screened_senders_pending 
            ON screened_senders(account_id, decision) WHERE decision = 'pending'
            ",
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get sender status by email address.
    ///
    /// Returns `None` if this sender has never been seen before.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn get_sender(
        &self,
        account_id: AccountId,
        email: &str,
    ) -> Result<Option<ScreenedSender>> {
        let normalized_email = email.to_lowercase();

        let row = sqlx::query(
            r"
            SELECT id, account_id, email, display_name, decision, category, 
                   note, email_count, first_seen, last_seen
            FROM screened_senders
            WHERE account_id = ? AND email = ?
            ",
        )
        .bind(account_id.0)
        .bind(&normalized_email)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| row_to_sender(&r)))
    }

    /// Get all pending senders for an account (The Screener).
    ///
    /// Returns senders who haven't been approved or blocked yet.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn get_pending_senders(&self, account_id: AccountId) -> Result<Vec<ScreenedSender>> {
        let rows = sqlx::query(
            r"
            SELECT id, account_id, email, display_name, decision, category,
                   note, email_count, first_seen, last_seen
            FROM screened_senders
            WHERE account_id = ? AND decision = 'pending'
            ORDER BY last_seen DESC
            ",
        )
        .bind(account_id.0)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.iter().map(row_to_sender).collect())
    }

    /// Get all approved senders in a specific category.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn get_senders_by_category(
        &self,
        account_id: AccountId,
        category: InboxCategory,
    ) -> Result<Vec<ScreenedSender>> {
        let rows = sqlx::query(
            r"
            SELECT id, account_id, email, display_name, decision, category,
                   note, email_count, first_seen, last_seen
            FROM screened_senders
            WHERE account_id = ? AND decision = 'approved' AND category = ?
            ORDER BY email_count DESC
            ",
        )
        .bind(account_id.0)
        .bind(category.as_str())
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.iter().map(row_to_sender).collect())
    }

    /// Get all blocked senders for an account.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn get_blocked_senders(&self, account_id: AccountId) -> Result<Vec<ScreenedSender>> {
        let rows = sqlx::query(
            r"
            SELECT id, account_id, email, display_name, decision, category,
                   note, email_count, first_seen, last_seen
            FROM screened_senders
            WHERE account_id = ? AND decision = 'blocked'
            ORDER BY last_seen DESC
            ",
        )
        .bind(account_id.0)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.iter().map(row_to_sender).collect())
    }

    /// Record a new sender or update existing sender's `last_seen` and `email_count`.
    ///
    /// If the sender is new, they'll be added as pending.
    /// If the sender exists, their `last_seen` and `email_count` will be updated.
    ///
    /// Returns the sender record (new or updated).
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails.
    pub async fn record_sender(
        &self,
        account_id: AccountId,
        email: &str,
        display_name: Option<&str>,
    ) -> Result<ScreenedSender> {
        let normalized_email = email.to_lowercase();

        // Try to update existing sender
        let updated = sqlx::query(
            r"
            UPDATE screened_senders
            SET last_seen = CURRENT_TIMESTAMP,
                email_count = email_count + 1,
                display_name = COALESCE(?, display_name),
                updated_at = CURRENT_TIMESTAMP
            WHERE account_id = ? AND email = ?
            ",
        )
        .bind(display_name)
        .bind(account_id.0)
        .bind(&normalized_email)
        .execute(&self.pool)
        .await?;

        if updated.rows_affected() == 0 {
            // New sender - insert as pending
            sqlx::query(
                r"
                INSERT INTO screened_senders (account_id, email, display_name, decision, category)
                VALUES (?, ?, ?, 'pending', 'imbox')
                ",
            )
            .bind(account_id.0)
            .bind(&normalized_email)
            .bind(display_name)
            .execute(&self.pool)
            .await?;
        }

        // Return the sender record
        self.get_sender(account_id, &normalized_email)
            .await?
            .ok_or_else(|| crate::Error::Config("Failed to retrieve sender after insert".into()))
    }

    /// Approve a sender and assign them to a category.
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails.
    pub async fn approve_sender(
        &self,
        account_id: AccountId,
        email: &str,
        category: InboxCategory,
    ) -> Result<()> {
        let normalized_email = email.to_lowercase();

        sqlx::query(
            r"
            UPDATE screened_senders
            SET decision = 'approved',
                category = ?,
                updated_at = CURRENT_TIMESTAMP
            WHERE account_id = ? AND email = ?
            ",
        )
        .bind(category.as_str())
        .bind(account_id.0)
        .bind(&normalized_email)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Block a sender.
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails.
    pub async fn block_sender(&self, account_id: AccountId, email: &str) -> Result<()> {
        let normalized_email = email.to_lowercase();

        sqlx::query(
            r"
            UPDATE screened_senders
            SET decision = 'blocked',
                updated_at = CURRENT_TIMESTAMP
            WHERE account_id = ? AND email = ?
            ",
        )
        .bind(account_id.0)
        .bind(&normalized_email)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Change a sender's category (only for approved senders).
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails.
    pub async fn change_category(
        &self,
        account_id: AccountId,
        email: &str,
        category: InboxCategory,
    ) -> Result<()> {
        let normalized_email = email.to_lowercase();

        sqlx::query(
            r"
            UPDATE screened_senders
            SET category = ?,
                updated_at = CURRENT_TIMESTAMP
            WHERE account_id = ? AND email = ? AND decision = 'approved'
            ",
        )
        .bind(category.as_str())
        .bind(account_id.0)
        .bind(&normalized_email)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Reset a sender back to pending (undo approve/block).
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails.
    pub async fn reset_sender(&self, account_id: AccountId, email: &str) -> Result<()> {
        let normalized_email = email.to_lowercase();

        sqlx::query(
            r"
            UPDATE screened_senders
            SET decision = 'pending',
                updated_at = CURRENT_TIMESTAMP
            WHERE account_id = ? AND email = ?
            ",
        )
        .bind(account_id.0)
        .bind(&normalized_email)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Delete a sender record entirely.
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails.
    pub async fn delete_sender(&self, account_id: AccountId, email: &str) -> Result<()> {
        let normalized_email = email.to_lowercase();

        sqlx::query(
            r"
            DELETE FROM screened_senders
            WHERE account_id = ? AND email = ?
            ",
        )
        .bind(account_id.0)
        .bind(&normalized_email)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get statistics about screened senders.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn get_stats(&self, account_id: AccountId) -> Result<TriageStats> {
        let row = sqlx::query(
            r"
            SELECT 
                COUNT(*) as total,
                SUM(CASE WHEN decision = 'pending' THEN 1 ELSE 0 END) as pending,
                SUM(CASE WHEN decision = 'approved' THEN 1 ELSE 0 END) as approved,
                SUM(CASE WHEN decision = 'blocked' THEN 1 ELSE 0 END) as blocked,
                SUM(CASE WHEN decision = 'approved' AND category = 'imbox' THEN 1 ELSE 0 END) as imbox,
                SUM(CASE WHEN decision = 'approved' AND category = 'feed' THEN 1 ELSE 0 END) as feed,
                SUM(CASE WHEN decision = 'approved' AND category = 'paper_trail' THEN 1 ELSE 0 END) as paper_trail
            FROM screened_senders
            WHERE account_id = ?
            ",
        )
        .bind(account_id.0)
        .fetch_one(&self.pool)
        .await?;

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        Ok(TriageStats {
            total_senders: row.get::<i64, _>("total") as u32,
            pending_count: row.get::<i64, _>("pending") as u32,
            approved_count: row.get::<i64, _>("approved") as u32,
            blocked_count: row.get::<i64, _>("blocked") as u32,
            imbox_count: row.get::<i64, _>("imbox") as u32,
            feed_count: row.get::<i64, _>("feed") as u32,
            paper_trail_count: row.get::<i64, _>("paper_trail") as u32,
        })
    }

    /// Check if a sender's emails should be shown (not blocked).
    ///
    /// Returns `true` if the sender is approved or pending (new).
    /// Returns `false` if the sender is blocked.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn should_show_email(&self, account_id: AccountId, email: &str) -> Result<bool> {
        let sender = self.get_sender(account_id, email).await?;
        Ok(sender.is_none_or(|s| !s.is_blocked()))
    }

    /// Get the category for an approved sender's emails.
    ///
    /// Returns `None` if sender is not approved (pending or blocked).
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn get_email_category(
        &self,
        account_id: AccountId,
        email: &str,
    ) -> Result<Option<InboxCategory>> {
        let sender = self.get_sender(account_id, email).await?;
        Ok(sender.and_then(|s| {
            if s.is_approved() {
                Some(s.category)
            } else {
                None
            }
        }))
    }
}

/// Statistics about triage state.
#[derive(Debug, Clone, Default)]
pub struct TriageStats {
    /// Total number of unique senders.
    pub total_senders: u32,
    /// Senders waiting for screening.
    pub pending_count: u32,
    /// Approved senders.
    pub approved_count: u32,
    /// Blocked senders.
    pub blocked_count: u32,
    /// Approved senders routed to Imbox.
    pub imbox_count: u32,
    /// Approved senders routed to Feed.
    pub feed_count: u32,
    /// Approved senders routed to Paper Trail.
    pub paper_trail_count: u32,
}

/// Convert a database row to a `ScreenedSender`.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn row_to_sender(row: &sqlx::sqlite::SqliteRow) -> ScreenedSender {
    ScreenedSender {
        id: Some(row.get("id")),
        account_id: AccountId::new(row.get("account_id")),
        email: row.get("email"),
        display_name: row.get("display_name"),
        decision: SenderDecision::parse(row.get("decision")),
        category: InboxCategory::parse(row.get("category")),
        note: row.get("note"),
        email_count: row.get::<i64, _>("email_count") as u32,
        first_seen: row.get("first_seen"),
        last_seen: row.get("last_seen"),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_record_new_sender() {
        let repo = TriageRepository::in_memory().await.unwrap();
        let account_id = AccountId::new(1);

        let sender = repo
            .record_sender(account_id, "test@example.com", Some("Test User"))
            .await
            .unwrap();

        assert_eq!(sender.email, "test@example.com");
        assert_eq!(sender.display_name, Some("Test User".to_string()));
        assert!(sender.is_pending());
        assert_eq!(sender.email_count, 1);
    }

    #[tokio::test]
    async fn test_record_existing_sender_increments_count() {
        let repo = TriageRepository::in_memory().await.unwrap();
        let account_id = AccountId::new(1);

        repo.record_sender(account_id, "test@example.com", None)
            .await
            .unwrap();
        let sender = repo
            .record_sender(account_id, "test@example.com", None)
            .await
            .unwrap();

        assert_eq!(sender.email_count, 2);
    }

    #[tokio::test]
    async fn test_approve_sender() {
        let repo = TriageRepository::in_memory().await.unwrap();
        let account_id = AccountId::new(1);

        repo.record_sender(account_id, "newsletter@example.com", None)
            .await
            .unwrap();
        repo.approve_sender(account_id, "newsletter@example.com", InboxCategory::Feed)
            .await
            .unwrap();

        let sender = repo
            .get_sender(account_id, "newsletter@example.com")
            .await
            .unwrap()
            .unwrap();
        assert!(sender.is_approved());
        assert_eq!(sender.category, InboxCategory::Feed);
    }

    #[tokio::test]
    async fn test_block_sender() {
        let repo = TriageRepository::in_memory().await.unwrap();
        let account_id = AccountId::new(1);

        repo.record_sender(account_id, "spam@example.com", None)
            .await
            .unwrap();
        repo.block_sender(account_id, "spam@example.com")
            .await
            .unwrap();

        let sender = repo
            .get_sender(account_id, "spam@example.com")
            .await
            .unwrap()
            .unwrap();
        assert!(sender.is_blocked());
    }

    #[tokio::test]
    async fn test_get_pending_senders() {
        let repo = TriageRepository::in_memory().await.unwrap();
        let account_id = AccountId::new(1);

        repo.record_sender(account_id, "pending1@example.com", None)
            .await
            .unwrap();
        repo.record_sender(account_id, "pending2@example.com", None)
            .await
            .unwrap();
        repo.record_sender(account_id, "approved@example.com", None)
            .await
            .unwrap();
        repo.approve_sender(account_id, "approved@example.com", InboxCategory::Imbox)
            .await
            .unwrap();

        let pending = repo.get_pending_senders(account_id).await.unwrap();
        assert_eq!(pending.len(), 2);
    }

    #[tokio::test]
    async fn test_should_show_email() {
        let repo = TriageRepository::in_memory().await.unwrap();
        let account_id = AccountId::new(1);

        // New sender - should show
        assert!(
            repo.should_show_email(account_id, "new@example.com")
                .await
                .unwrap()
        );

        // Pending sender - should show
        repo.record_sender(account_id, "pending@example.com", None)
            .await
            .unwrap();
        assert!(
            repo.should_show_email(account_id, "pending@example.com")
                .await
                .unwrap()
        );

        // Approved sender - should show
        repo.record_sender(account_id, "approved@example.com", None)
            .await
            .unwrap();
        repo.approve_sender(account_id, "approved@example.com", InboxCategory::Imbox)
            .await
            .unwrap();
        assert!(
            repo.should_show_email(account_id, "approved@example.com")
                .await
                .unwrap()
        );

        // Blocked sender - should NOT show
        repo.record_sender(account_id, "blocked@example.com", None)
            .await
            .unwrap();
        repo.block_sender(account_id, "blocked@example.com")
            .await
            .unwrap();
        assert!(
            !repo
                .should_show_email(account_id, "blocked@example.com")
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn test_stats() {
        let repo = TriageRepository::in_memory().await.unwrap();
        let account_id = AccountId::new(1);

        repo.record_sender(account_id, "pending@example.com", None)
            .await
            .unwrap();
        repo.record_sender(account_id, "imbox@example.com", None)
            .await
            .unwrap();
        repo.approve_sender(account_id, "imbox@example.com", InboxCategory::Imbox)
            .await
            .unwrap();
        repo.record_sender(account_id, "feed@example.com", None)
            .await
            .unwrap();
        repo.approve_sender(account_id, "feed@example.com", InboxCategory::Feed)
            .await
            .unwrap();
        repo.record_sender(account_id, "blocked@example.com", None)
            .await
            .unwrap();
        repo.block_sender(account_id, "blocked@example.com")
            .await
            .unwrap();

        let stats = repo.get_stats(account_id).await.unwrap();
        assert_eq!(stats.total_senders, 4);
        assert_eq!(stats.pending_count, 1);
        assert_eq!(stats.approved_count, 2);
        assert_eq!(stats.blocked_count, 1);
        assert_eq!(stats.imbox_count, 1);
        assert_eq!(stats.feed_count, 1);
    }

    #[tokio::test]
    async fn test_email_normalization() {
        let repo = TriageRepository::in_memory().await.unwrap();
        let account_id = AccountId::new(1);

        repo.record_sender(account_id, "TEST@EXAMPLE.COM", None)
            .await
            .unwrap();

        // Should find with different case
        let sender = repo
            .get_sender(account_id, "test@example.com")
            .await
            .unwrap();
        assert!(sender.is_some());
        assert_eq!(sender.unwrap().email, "test@example.com");
    }
}
