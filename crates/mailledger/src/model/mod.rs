//! Data models for the email client.

mod account_setup;
mod compose;
mod folder;
mod inline_image;
mod message;
mod settings;

pub use account_setup::AccountSetupState;
pub use compose::ComposeState;
pub use folder::{Folder, FolderId, FolderType};
pub use inline_image::{InlineImage, InlineImageState};
pub use message::{MessageContent, MessageId, MessageSummary};
pub use settings::{AppSettings, SettingsSection, SettingsState};
