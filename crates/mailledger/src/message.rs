//! Message types for application events.
//!
//! In the Elm architecture, Messages are events that trigger state changes.

use crate::model::{AppSettings, Folder, FolderId, MessageId, MessageSummary};

/// Application messages (events).
#[derive(Debug, Clone)]
#[allow(dead_code)] // Some variants will be used when features are implemented
#[allow(clippy::enum_variant_names)] // Variant names are clear in context
pub enum Message {
    // Navigation
    /// Navigate to a different view.
    NavigateTo(View),
    /// Toggle sidebar visibility.
    ToggleSidebar,

    // Folder Operations
    /// Select a folder to view its messages.
    SelectFolder(FolderId),
    /// Refresh folder list.
    RefreshFolders,

    // Message List Operations
    /// Select a message to view its content.
    SelectMessage(MessageId),
    /// Toggle read/unread status.
    ToggleRead(MessageId),
    /// Toggle flagged/starred status.
    ToggleFlag(MessageId),
    /// Delete a message.
    DeleteMessage(MessageId),
    /// Delete the currently selected message.
    DeleteSelected,

    // Compose
    /// Start composing a new message.
    ComposeNew,
    /// Reply to the selected message.
    Reply,
    /// Reply all to the selected message.
    ReplyAll,
    /// Forward the selected message.
    Forward,
    /// Compose form messages.
    Compose(ComposeMessage),
    /// Email sent result.
    EmailSent(Result<(), String>),

    // Settings
    /// Settings screen messages.
    Settings(SettingsMessage),

    // Account Setup
    /// Account setup messages.
    AccountSetup(AccountSetupMessage),
    /// Account saved successfully.
    AccountSaved(Result<(), String>),
    /// Connection test completed.
    ConnectionTested(Result<(), String>),
    /// Settings saved.
    SettingsSaved(Result<(), String>),
    /// Settings loaded.
    SettingsLoaded(Result<AppSettings, String>),

    // IMAP Operations
    /// Load account from database.
    LoadAccount,
    /// Account loaded from database.
    AccountLoaded(Result<Option<mailledger_core::Account>, String>),
    /// Folders loaded from IMAP server.
    FoldersLoaded(Result<Vec<Folder>, String>),
    /// Messages loaded from IMAP server.
    MessagesLoaded(Result<Vec<MessageSummary>, String>),
    /// Message content loaded from IMAP server.
    MessageContentLoaded(Result<Option<crate::model::MessageContent>, String>),

    // IDLE Operations
    /// Start IDLE monitoring on the current folder.
    StartIdle,
    /// IDLE event received.
    IdleReceived(Result<mailledger_core::IdleEvent, String>),

    // Search
    /// Search query changed.
    SearchQueryChanged(String),
    /// Execute search.
    SearchExecute,

    // UI Events
    /// Window resized.
    WindowResized(u32, u32),

    // Keyboard Events
    /// Keyboard shortcut pressed.
    KeyPressed(KeyboardAction),
}

/// Keyboard actions that can be triggered by shortcuts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyboardAction {
    /// Compose new message (Ctrl+N).
    ComposeNew,
    /// Reply to selected message (Ctrl+R).
    Reply,
    /// Reply all (Ctrl+Shift+R).
    ReplyAll,
    /// Forward message (Ctrl+Shift+F).
    Forward,
    /// Delete selected message (Delete).
    Delete,
    /// Refresh folders (F5 or Ctrl+Shift+R when not in Inbox).
    Refresh,
    /// Cancel/close current view (Escape).
    Cancel,
    /// Send message in compose view (Ctrl+Enter).
    Send,
    /// Open settings (Ctrl+,).
    Settings,
    /// Toggle sidebar (Ctrl+B).
    ToggleSidebar,
}

/// Messages for account setup form.
#[derive(Debug, Clone)]
pub enum AccountSetupMessage {
    /// Account name changed.
    NameChanged(String),
    /// Email changed.
    EmailChanged(String),
    /// IMAP host changed.
    ImapHostChanged(String),
    /// IMAP port changed.
    ImapPortChanged(String),
    /// IMAP security changed.
    ImapSecurityChanged(String),
    /// IMAP username changed.
    ImapUsernameChanged(String),
    /// IMAP password changed.
    ImapPasswordChanged(String),
    /// SMTP host changed.
    SmtpHostChanged(String),
    /// SMTP port changed.
    SmtpPortChanged(String),
    /// SMTP security changed.
    SmtpSecurityChanged(String),
    /// SMTP username changed.
    SmtpUsernameChanged(String),
    /// SMTP password changed.
    SmtpPasswordChanged(String),
    /// Save account.
    Save,
    /// Test connection.
    TestConnection,
    /// Cancel setup.
    Cancel,
}

/// Messages for settings screen.
#[derive(Debug, Clone, Copy)]
pub enum SettingsMessage {
    /// Select a settings section.
    SelectSection(crate::model::SettingsSection),
    /// Toggle between light and dark theme.
    ToggleTheme,
}

/// Messages for compose form.
#[derive(Debug, Clone)]
pub enum ComposeMessage {
    /// To field changed.
    ToChanged(String),
    /// CC field changed.
    CcChanged(String),
    /// BCC field changed.
    BccChanged(String),
    /// Subject changed.
    SubjectChanged(String),
    /// Body changed.
    BodyChanged(String),
    /// Send the message.
    Send,
    /// Cancel composing.
    Cancel,
}

/// Application views/screens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)] // Variants will be used as features are implemented
pub enum View {
    /// Main inbox view with three panes.
    #[default]
    Inbox,
    /// Compose new message.
    Compose,
    /// Settings screen.
    Settings,
    /// Account setup wizard.
    AccountSetup,
}
