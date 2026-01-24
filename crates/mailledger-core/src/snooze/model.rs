//! Snooze data models.

use chrono::{DateTime, Datelike, Duration, Local, Utc};

use crate::AccountId;

/// A snoozed message that will reappear at a specified time.
#[derive(Debug, Clone)]
pub struct SnoozedMessage {
    /// Account ID this message belongs to.
    pub account_id: AccountId,
    /// Message UID in the original folder.
    pub message_uid: u32,
    /// Original folder path where the message was.
    pub folder_path: String,
    /// When the snooze expires and message should reappear.
    pub snooze_until: DateTime<Utc>,
    /// When the message was snoozed.
    pub snoozed_at: DateTime<Utc>,
    /// Message subject for display.
    pub subject: String,
    /// Message sender for display.
    pub from: String,
}

impl SnoozedMessage {
    /// Creates a new snoozed message.
    #[must_use]
    pub fn new(
        account_id: AccountId,
        message_uid: u32,
        folder_path: impl Into<String>,
        snooze_until: DateTime<Utc>,
        subject: impl Into<String>,
        from: impl Into<String>,
    ) -> Self {
        Self {
            account_id,
            message_uid,
            folder_path: folder_path.into(),
            snooze_until,
            snoozed_at: Utc::now(),
            subject: subject.into(),
            from: from.into(),
        }
    }

    /// Returns true if the snooze has expired.
    #[must_use]
    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.snooze_until
    }

    /// Returns the remaining time until the snooze expires.
    #[must_use]
    pub fn time_remaining(&self) -> Option<Duration> {
        let now = Utc::now();
        if now >= self.snooze_until {
            None
        } else {
            Some(self.snooze_until - now)
        }
    }
}

/// Preset snooze duration options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnoozeDuration {
    /// Snooze for 3 hours.
    LaterToday,
    /// Snooze until tomorrow morning (9 AM).
    Tomorrow,
    /// Snooze until next week (Monday 9 AM).
    NextWeek,
    /// Custom date/time.
    Custom(DateTime<Utc>),
}

impl SnoozeDuration {
    /// Calculates the snooze expiry time from now.
    #[must_use]
    #[allow(clippy::expect_used)] // 9:00:00 is always a valid time
    pub fn expiry_time(&self) -> DateTime<Utc> {
        let now = Local::now();

        match self {
            Self::LaterToday => {
                // 3 hours from now
                (now + Duration::hours(3)).with_timezone(&Utc)
            }
            Self::Tomorrow => {
                // Tomorrow at 9 AM local time
                let tomorrow = now.date_naive() + chrono::Days::new(1);
                tomorrow
                    .and_hms_opt(9, 0, 0)
                    .and_then(|t| t.and_local_timezone(Local).single())
                    .map_or_else(
                        || (now + Duration::hours(24)).with_timezone(&Utc),
                        |t| t.with_timezone(&Utc),
                    )
            }
            Self::NextWeek => {
                // Next Monday at 9 AM local time
                let days_until_monday = (8 - now.weekday().num_days_from_monday()) % 7;
                let days_until_monday = if days_until_monday == 0 {
                    7
                } else {
                    days_until_monday
                };
                let next_monday =
                    now.date_naive() + chrono::Days::new(u64::from(days_until_monday));
                next_monday
                    .and_hms_opt(9, 0, 0)
                    .and_then(|t| t.and_local_timezone(Local).single())
                    .map_or_else(
                        || (now + Duration::days(i64::from(days_until_monday))).with_timezone(&Utc),
                        |t| t.with_timezone(&Utc),
                    )
            }
            Self::Custom(dt) => *dt,
        }
    }

    /// Returns a human-readable description of the snooze duration.
    #[must_use]
    pub fn description(&self) -> String {
        match self {
            Self::LaterToday => "Later today (3 hours)".to_string(),
            Self::Tomorrow => "Tomorrow morning".to_string(),
            Self::NextWeek => "Next Monday".to_string(),
            Self::Custom(dt) => {
                let local: DateTime<Local> = dt.with_timezone(&Local);
                local.format("%a, %b %d at %H:%M").to_string()
            }
        }
    }
}
