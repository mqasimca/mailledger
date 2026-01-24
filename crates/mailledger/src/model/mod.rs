//! Data models for the email client.

mod account_setup;
mod compose;
mod folder;
mod inline_image;
mod message;
mod settings;
mod thread;

pub use account_setup::AccountSetupState;
pub use compose::{AutocompleteField, ComposeState};
pub use folder::{Folder, FolderId, FolderType};
pub use inline_image::{InlineImage, InlineImageState};
#[allow(unused_imports)] // Attachment is part of MessageContent's public API
pub use message::{Attachment, MessageContent, MessageId, MessageSummary};
pub use settings::{AppSettings, FontSize, ListDensity, SettingsSection, SettingsState};
pub use thread::{Thread, ViewMode, group_into_threads};
