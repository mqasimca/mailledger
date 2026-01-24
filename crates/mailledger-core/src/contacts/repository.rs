//! Contact storage repository.

use sqlx::Row;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

use super::model::Contact;
use crate::Result;

/// Repository for contact storage and retrieval.
pub struct ContactRepository {
    pool: SqlitePool,
}

impl ContactRepository {
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
            CREATE TABLE IF NOT EXISTS contacts (
                email TEXT PRIMARY KEY NOT NULL,
                name TEXT NOT NULL DEFAULT '',
                use_count INTEGER NOT NULL DEFAULT 1,
                last_used TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            ",
        )
        .execute(&self.pool)
        .await?;

        // Create index for faster prefix searches
        sqlx::query(
            r"
            CREATE INDEX IF NOT EXISTS idx_contacts_name ON contacts(name COLLATE NOCASE)
            ",
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Record a contact (insert or update use count).
    ///
    /// If the contact already exists, increments the use count and updates `last_used`.
    /// If new, inserts with `use_count` = 1.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn record(&self, email: &str, name: &str) -> Result<()> {
        // Normalize email to lowercase
        let email_normalized = email.trim().to_lowercase();
        let name_trimmed = name.trim();

        sqlx::query(
            r"
            INSERT INTO contacts (email, name, use_count, last_used)
            VALUES (?, ?, 1, CURRENT_TIMESTAMP)
            ON CONFLICT(email) DO UPDATE SET
                name = CASE
                    WHEN excluded.name != '' THEN excluded.name
                    ELSE contacts.name
                END,
                use_count = contacts.use_count + 1,
                last_used = CURRENT_TIMESTAMP
            ",
        )
        .bind(&email_normalized)
        .bind(name_trimmed)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Search contacts matching a query string.
    ///
    /// Returns contacts where email or name contains the query (case-insensitive).
    /// Results are ordered by `use_count` descending (most used first).
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn search(&self, query: &str, limit: u32) -> Result<Vec<Contact>> {
        let query_pattern = format!("%{}%", query.trim().to_lowercase());

        let rows = sqlx::query(
            r"
            SELECT email, name, use_count
            FROM contacts
            WHERE LOWER(email) LIKE ? OR LOWER(name) LIKE ?
            ORDER BY use_count DESC, last_used DESC
            LIMIT ?
            ",
        )
        .bind(&query_pattern)
        .bind(&query_pattern)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let contacts = rows
            .iter()
            .map(|row| Contact {
                email: row.get("email"),
                name: row.get("name"),
                use_count: row.get::<i64, _>("use_count") as u32,
            })
            .collect();

        Ok(contacts)
    }

    /// Get all contacts ordered by use count.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn list(&self, limit: u32) -> Result<Vec<Contact>> {
        let rows = sqlx::query(
            r"
            SELECT email, name, use_count
            FROM contacts
            ORDER BY use_count DESC, last_used DESC
            LIMIT ?
            ",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let contacts = rows
            .iter()
            .map(|row| Contact {
                email: row.get("email"),
                name: row.get("name"),
                use_count: row.get::<i64, _>("use_count") as u32,
            })
            .collect();

        Ok(contacts)
    }

    /// Delete a contact.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn delete(&self, email: &str) -> Result<()> {
        sqlx::query("DELETE FROM contacts WHERE email = ?")
            .bind(email.to_lowercase())
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_record_and_search() {
        let repo = ContactRepository::in_memory().await.unwrap();

        repo.record("alice@example.com", "Alice Smith")
            .await
            .unwrap();
        repo.record("bob@example.com", "Bob Jones").await.unwrap();

        let results = repo.search("alice", 10).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].email, "alice@example.com");
        assert_eq!(results[0].name, "Alice Smith");
    }

    #[tokio::test]
    async fn test_increment_use_count() {
        let repo = ContactRepository::in_memory().await.unwrap();

        repo.record("test@example.com", "Test User").await.unwrap();
        repo.record("test@example.com", "").await.unwrap();
        repo.record("test@example.com", "").await.unwrap();

        let results = repo.search("test", 10).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].use_count, 3);
        // Name should be preserved
        assert_eq!(results[0].name, "Test User");
    }

    #[tokio::test]
    async fn test_search_by_name() {
        let repo = ContactRepository::in_memory().await.unwrap();

        repo.record("john@example.com", "John Doe").await.unwrap();

        let results = repo.search("doe", 10).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].email, "john@example.com");
    }

    #[tokio::test]
    async fn test_case_insensitive_search() {
        let repo = ContactRepository::in_memory().await.unwrap();

        repo.record("Test@Example.COM", "Test User").await.unwrap();

        let results = repo.search("TEST", 10).await.unwrap();
        assert_eq!(results.len(), 1);
        // Email should be normalized to lowercase
        assert_eq!(results[0].email, "test@example.com");
    }

    #[tokio::test]
    async fn test_order_by_use_count() {
        let repo = ContactRepository::in_memory().await.unwrap();

        repo.record("rare@example.com", "Rare").await.unwrap();

        repo.record("frequent@example.com", "Frequent")
            .await
            .unwrap();
        repo.record("frequent@example.com", "").await.unwrap();
        repo.record("frequent@example.com", "").await.unwrap();

        let results = repo.search("example", 10).await.unwrap();
        assert_eq!(results.len(), 2);
        // Frequent should come first
        assert_eq!(results[0].email, "frequent@example.com");
        assert_eq!(results[1].email, "rare@example.com");
    }
}
