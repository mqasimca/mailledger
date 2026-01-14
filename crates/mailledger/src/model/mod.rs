//! Data models for the email client.

mod account_setup;
mod compose;
mod folder;
mod message;
mod settings;

pub use account_setup::AccountSetupState;
pub use compose::ComposeState;
pub use folder::{Folder, FolderId, FolderType};
pub use message::{MessageContent, MessageId, MessageSummary};
pub use settings::{AppSettings, SettingsSection, SettingsState};
