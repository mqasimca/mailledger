//! Message types for application events.
//!
//! In the Elm architecture, Messages are events that trigger state changes.

use crate::model::{AppSettings, Folder, FolderId, MessageId, MessageSummary};

/// Re-export snooze duration for use in messages.
pub use mailledger_core::SnoozeDuration;

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
    /// Refresh messages in the current folder.
    RefreshMessages,

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
    /// Archive a message (move to Archive folder).
    ArchiveMessage(MessageId),
    /// Archive the currently selected message.
    ArchiveSelected,
    /// Message archived result.
    MessageArchived(Result<(), String>),
    /// Download an attachment.
    DownloadAttachment {
        /// Message ID containing the attachment.
        message_id: MessageId,
        /// IMAP part number.
        part_number: String,
        /// Filename to save as.
        filename: String,
        /// Content-Transfer-Encoding.
        encoding: String,
    },
    /// Attachment download complete.
    AttachmentDownloaded(Result<(String, Vec<u8>), String>),

    // Snooze
    /// Snooze the currently selected message.
    SnoozeSelected(SnoozeDuration),
    /// Snooze operation completed.
    MessageSnoozed(Result<(), String>),
    /// Check for expired snoozes.
    CheckExpiredSnoozes,
    /// Expired snoozes loaded.
    ExpiredSnoozesLoaded(Result<Vec<mailledger_core::SnoozedMessage>, String>),
    /// Toggle snooze dropdown visibility.
    ToggleSnoozeDropdown,

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

    // Triage / Screener
    /// Screener action messages.
    Screener(ScreenerMessage),
    /// Pending senders loaded.
    PendingSendersLoaded(Result<Vec<mailledger_core::ScreenedSender>, String>),
    /// Sender decision saved.
    SenderDecisionSaved(Result<(), String>),
    /// Record a new sender from incoming email.
    RecordSender {
        /// Email address.
        email: String,
        /// Display name.
        display_name: Option<String>,
    },

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

    // Account Management
    /// Load all accounts from database.
    LoadAccounts,
    /// All accounts loaded from database.
    AccountsLoaded(Result<Vec<mailledger_core::Account>, String>),
    /// Switch to a different account.
    SwitchAccount(mailledger_core::AccountId),
    /// Toggle account switcher dropdown.
    ToggleAccountSwitcher,
    /// Add new account (go to account setup).
    AddAccount,

    // IMAP Operations
    /// Load account from database (legacy, for single account).
    LoadAccount,
    /// Account loaded from database.
    AccountLoaded(Result<Option<mailledger_core::Account>, String>),
    /// Folders loaded from IMAP server.
    FoldersLoaded(Result<Vec<Folder>, String>),
    /// Messages loaded from IMAP server.
    MessagesLoaded(Result<Vec<MessageSummary>, String>),
    /// Message content loaded from IMAP server.
    MessageContentLoaded(Result<Option<crate::model::MessageContent>, String>),
    /// Open the current message HTML in an external viewer.
    OpenHtml,
    /// HTML open completed.
    HtmlOpened(Result<(), String>),
    /// Link clicked in message content (markdown).
    LinkClicked(String),
    /// Inline image loaded from a remote source.
    InlineImageLoaded {
        /// Image source URL.
        url: String,
        /// Image bytes or error.
        result: Result<Vec<u8>, String>,
    },

    // IDLE Operations
    /// Start IDLE monitoring on the current folder.
    StartIdle,
    /// IDLE event received.
    IdleReceived(Result<mailledger_core::IdleEvent, String>),

    // Offline Support
    /// Connection state changed (online/offline).
    ConnectionStateChanged(bool),
    /// Cached messages loaded when offline.
    CachedMessagesLoaded(Result<Vec<mailledger_core::CachedMessageSummary>, String>),
    /// Cached content loaded when offline.
    CachedContentLoaded(Result<Option<mailledger_core::CachedMessageContent>, String>),

    // Search
    /// Search query changed.
    SearchQueryChanged(String),
    /// Execute search.
    SearchExecute,
    /// Search results loaded (UIDs of matching messages).
    SearchResultsLoaded(Result<Vec<mailledger_imap::Uid>, String>),
    /// Toggle a search filter chip.
    ToggleSearchFilter(SearchFilter),

    // Threading
    /// Toggle between flat and threaded view.
    ToggleViewMode,
    /// Expand or collapse a thread.
    ToggleThread(String),

    // Quote Collapse
    /// Toggle expanded state for quoted text in message view.
    ToggleQuotedText,

    // UI Events
    /// Window resized.
    WindowResized(u32, u32),
    /// Message list scrolled (for virtual scrolling).
    MessageListScrolled(iced::widget::scrollable::Viewport),
    /// Start dragging a pane divider.
    StartPaneDrag(PaneDivider),
    /// Stop dragging a pane divider.
    StopPaneDrag,
    /// Mouse moved during pane drag.
    PaneDragMoved(f32),

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
    /// Delete selected message (Delete or #).
    Delete,
    /// Archive selected message (e).
    Archive,
    /// Toggle star/flag (s).
    ToggleStar,
    /// Mark message unread (u).
    MarkUnread,
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
    /// Select next message (Down arrow or J).
    SelectNextMessage,
    /// Select previous message (Up arrow or K).
    SelectPreviousMessage,
    /// Focus search box (/).
    FocusSearch,
    /// Show keyboard shortcuts help (?).
    ShowHelp,
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
    /// Change font size.
    SetFontSize(crate::model::FontSize),
    /// Change list density.
    SetDensity(crate::model::ListDensity),
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
    /// Body changed (for compatibility, not used with `text_editor`).
    #[allow(dead_code)]
    BodyChanged(String),
    /// Body text editor action.
    BodyAction(iced::widget::text_editor::Action),
    /// Insert markdown formatting at cursor.
    InsertFormatting(FormattingStyle),
    /// Send the message.
    Send,
    /// Cancel composing.
    Cancel,
    /// Contact suggestions loaded from database.
    #[allow(dead_code)] // Will be used when contact repository is integrated
    SuggestionsLoaded(Vec<mailledger_core::Contact>),
    /// Select a suggestion from the autocomplete dropdown.
    SelectSuggestion(usize),
    /// Dismiss the autocomplete dropdown.
    #[allow(dead_code)] // Will be used for keyboard escape handling
    DismissSuggestions,
}

/// Markdown formatting styles for the toolbar.
#[derive(Debug, Clone, Copy)]
pub enum FormattingStyle {
    /// Bold text (**text**)
    Bold,
    /// Italic text (*text*)
    Italic,
    /// Link [text](url)
    Link,
}

/// Application views/screens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)] // Variants will be used as features are implemented
pub enum View {
    /// Main inbox view with three panes.
    #[default]
    Inbox,
    /// The Screener - approve or block new senders.
    Screener,
    /// Compose new message.
    Compose,
    /// Settings screen.
    Settings,
    /// Account setup wizard.
    AccountSetup,
}

/// Messages for the triage/screener system.
#[derive(Debug, Clone)]
pub enum ScreenerMessage {
    /// Approve a sender and route to Imbox.
    ApproveToImbox(String),
    /// Approve a sender and route to Feed (newsletters).
    ApproveToFeed(String),
    /// Approve a sender and route to Paper Trail (receipts).
    ApproveToPaperTrail(String),
    /// Block a sender.
    Block(String),
    /// Reset a sender back to pending.
    #[allow(dead_code)] // Will be used for undo functionality
    Reset(String),
}

/// Quick search filters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SearchFilter {
    /// Show only unread messages.
    Unread,
    /// Show only flagged/starred messages.
    Flagged,
    /// Show only messages with attachments.
    HasAttachments,
}

/// Pane dividers that can be dragged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaneDivider {
    /// Divider between sidebar and message list.
    SidebarMessageList,
    /// Divider between message list and message view.
    MessageListMessageView,
}
