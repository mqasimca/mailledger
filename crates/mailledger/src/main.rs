//! `MailLedger` - Cross-platform desktop email client
//!
//! Built with Rust, iced GUI framework, and custom IMAP implementation.

#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

mod message;
mod model;
mod style;
mod view;

use iced::keyboard::{self, Key, Modifiers};
use iced::widget::{column, row};
use iced::{Element, Length, Subscription, Task};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use std::collections::HashMap;

use message::{
    AccountSetupMessage, ComposeMessage, KeyboardAction, Message, SettingsMessage, View,
};
use model::{
    AccountSetupState, AppSettings, ComposeState, Folder, FolderId, FolderType, MessageContent,
    MessageId, MessageSummary, SettingsState,
};
use style::widgets::palette::ThemeMode;

fn main() -> iced::Result {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mailledger=debug,mailledger_imap=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting MailLedger");

    iced::application(MailLedger::new, MailLedger::update, MailLedger::view)
        .title("MailLedger")
        .subscription(MailLedger::subscription)
        .run()
}

/// Main application state.
#[allow(clippy::struct_excessive_bools)] // State flags are clearer as bools for now
struct MailLedger {
    /// Current view/screen.
    current_view: View,
    /// Whether sidebar is visible.
    sidebar_visible: bool,
    /// List of folders.
    folders: Vec<Folder>,
    /// Currently selected folder.
    selected_folder: Option<FolderId>,
    /// Messages in the current folder (filtered).
    messages: Vec<MessageSummary>,
    /// All messages in the current folder (unfiltered).
    all_messages: Vec<MessageSummary>,
    /// Currently selected message.
    selected_message: Option<MessageId>,
    /// Content of the selected message.
    message_content: Option<MessageContent>,
    /// Search query.
    search_query: String,
    /// Account setup state.
    account_setup: AccountSetupState,
    /// Compose message state.
    compose_state: ComposeState,
    /// Settings state.
    settings_state: SettingsState,
    /// Current account (if logged in).
    current_account: Option<mailledger_core::Account>,
    /// Mapping from folder ID to IMAP folder path.
    folder_paths: HashMap<FolderId, String>,
    /// Whether we're currently loading folders.
    is_loading_folders: bool,
    /// Whether we're currently loading messages.
    is_loading_messages: bool,
    /// Error message to display.
    error_message: Option<String>,
    /// Whether IDLE monitoring is active.
    is_idle_active: bool,
    /// Current theme mode (light/dark).
    theme_mode: ThemeMode,
}

impl Default for MailLedger {
    fn default() -> Self {
        Self {
            current_view: View::Inbox,
            sidebar_visible: true,
            folders: Vec::new(),
            selected_folder: None,
            messages: Vec::new(),
            all_messages: Vec::new(),
            selected_message: None,
            message_content: None,
            search_query: String::new(),
            account_setup: AccountSetupState::new(),
            compose_state: ComposeState::new(),
            settings_state: SettingsState::new(),
            current_account: None,
            folder_paths: HashMap::new(),
            is_loading_folders: false,
            is_loading_messages: false,
            error_message: None,
            is_idle_active: false,
            theme_mode: ThemeMode::Dark, // Default to dark mode
        }
    }
}

impl MailLedger {
    /// Applies the current theme mode to the global palette.
    fn apply_theme(&self) {
        style::widgets::palette::set_theme(self.theme_mode);
    }

    /// Filter messages based on the current search query.
    fn filter_messages(&mut self) {
        if self.search_query.is_empty() {
            // No search query - show all messages
            self.messages = self.all_messages.clone();
        } else {
            // Filter by subject, sender name, sender email
            let query = self.search_query.to_lowercase();
            self.messages = self
                .all_messages
                .iter()
                .filter(|msg| {
                    msg.subject.to_lowercase().contains(&query)
                        || msg.from_name.to_lowercase().contains(&query)
                        || msg.from_email.to_lowercase().contains(&query)
                        || msg.snippet.to_lowercase().contains(&query)
                })
                .cloned()
                .collect();
        }
    }
}

impl MailLedger {
    /// Create new application instance.
    fn new() -> (Self, Task<Message>) {
        // On startup, load settings first, then load account
        let app = Self::default();
        app.apply_theme(); // Apply default theme initially
        let settings_task = Task::perform(load_settings(), Message::SettingsLoaded);
        let account_task = Task::perform(load_account(), Message::AccountLoaded);
        (app, Task::batch([settings_task, account_task]))
    }

    /// Update state based on message.
    #[allow(clippy::needless_pass_by_value)]
    #[allow(clippy::too_many_lines)] // Large match is idiomatic for Elm architecture
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NavigateTo(view) => {
                self.current_view = view;
                if view == View::AccountSetup {
                    self.account_setup = AccountSetupState::new();
                }
            }
            Message::ToggleSidebar => {
                self.sidebar_visible = !self.sidebar_visible;
            }
            Message::SelectFolder(folder_id) => {
                self.selected_folder = Some(folder_id);
                self.selected_message = None;
                self.message_content = None;

                // Load messages from IMAP if we have an account and folder path
                if let Some(account) = self.current_account.clone()
                    && let Some(folder) = self.folders.iter().find(|f| f.id == folder_id)
                {
                    let folder_path = folder.path.clone();
                    self.is_loading_messages = true;
                    self.messages.clear();
                    return Task::perform(
                        load_messages(account, folder_path, folder_id),
                        Message::MessagesLoaded,
                    );
                }
            }
            Message::RefreshFolders => {
                // Reload folders from IMAP
                if let Some(account) = self.current_account.clone() {
                    self.is_loading_folders = true;
                    return Task::perform(load_folders(account), Message::FoldersLoaded);
                }
            }
            Message::SearchQueryChanged(query) => {
                self.search_query = query;
                // Filter messages locally for instant feedback
                self.filter_messages();
            }
            Message::SearchExecute => {
                // For now, just filter locally (IMAP SEARCH requires library enhancements)
                self.filter_messages();
            }
            Message::WindowResized(_, _) => {
                // No action needed
            }
            Message::SelectMessage(message_id) => {
                self.selected_message = Some(message_id);
                self.message_content = None; // Clear while loading

                // Fetch full message content from IMAP
                if let Some(account) = self.current_account.clone()
                    && let Some(folder_id) = self.selected_folder
                    && let Some(folder_path) = self.folder_paths.get(&folder_id).cloned()
                {
                    return Task::perform(
                        load_message_content(account, folder_path, message_id.0),
                        Message::MessageContentLoaded,
                    );
                }
            }
            Message::MessageContentLoaded(result) => match result {
                Ok(Some(content)) => {
                    self.message_content = Some(content);
                }
                Ok(None) => {
                    // Message not found, show error or fallback
                    if let Some(summary) = self
                        .selected_message
                        .and_then(|id| self.messages.iter().find(|m| m.id == id))
                    {
                        self.message_content = Some(MessageContent::mock_content(summary));
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to load message content: {}", e);
                    // Fallback to mock content on error
                    if let Some(summary) = self
                        .selected_message
                        .and_then(|id| self.messages.iter().find(|m| m.id == id))
                    {
                        self.message_content = Some(MessageContent::mock_content(summary));
                    }
                }
            },
            Message::ToggleRead(message_id) => {
                if let Some(msg) = self.messages.iter_mut().find(|m| m.id == message_id) {
                    msg.is_read = !msg.is_read;
                }
            }
            Message::ToggleFlag(message_id) => {
                if let Some(msg) = self.messages.iter_mut().find(|m| m.id == message_id) {
                    msg.is_flagged = !msg.is_flagged;
                }
            }
            Message::DeleteMessage(message_id) => {
                self.messages.retain(|m| m.id != message_id);
                if self.selected_message == Some(message_id) {
                    self.selected_message = None;
                    self.message_content = None;
                }
            }
            Message::DeleteSelected => {
                if let Some(message_id) = self.selected_message {
                    self.messages.retain(|m| m.id != message_id);
                    self.selected_message = None;
                    self.message_content = None;
                }
            }
            Message::ComposeNew => {
                self.compose_state = ComposeState::new();
                self.current_view = View::Compose;
            }
            Message::Reply => {
                if let Some(ref content) = self.message_content {
                    self.compose_state = ComposeState::reply(
                        &content.from_email,
                        &content.subject,
                        content.body_text.as_deref().unwrap_or(""),
                    );
                } else {
                    self.compose_state = ComposeState::new();
                }
                self.current_view = View::Compose;
            }
            Message::ReplyAll => {
                // For now, same as Reply
                if let Some(ref content) = self.message_content {
                    self.compose_state = ComposeState::reply(
                        &content.from_email,
                        &content.subject,
                        content.body_text.as_deref().unwrap_or(""),
                    );
                } else {
                    self.compose_state = ComposeState::new();
                }
                self.current_view = View::Compose;
            }
            Message::Forward => {
                if let Some(ref content) = self.message_content {
                    self.compose_state = ComposeState::forward(
                        &content.subject,
                        content.body_text.as_deref().unwrap_or(""),
                        &content.from_email,
                    );
                } else {
                    self.compose_state = ComposeState::new();
                }
                self.current_view = View::Compose;
            }
            Message::Compose(msg) => {
                return self.handle_compose(msg);
            }
            Message::EmailSent(result) => {
                self.compose_state.is_sending = false;
                match result {
                    Ok(()) => {
                        info!("Email sent successfully");
                        self.compose_state.send_success = true;
                        self.compose_state.send_error = None;
                    }
                    Err(e) => {
                        self.compose_state.send_error = Some(e);
                    }
                }
            }
            Message::Settings(msg) => {
                return self.handle_settings(msg);
            }
            Message::AccountSetup(msg) => {
                return self.handle_account_setup(msg);
            }
            Message::AccountSaved(result) => {
                self.account_setup.is_saving = false;
                match result {
                    Ok(()) => {
                        self.current_view = View::Inbox;
                        // Reload the account to connect to IMAP
                        return Task::perform(load_account(), Message::AccountLoaded);
                    }
                    Err(e) => {
                        self.account_setup.save_error = Some(e);
                    }
                }
            }
            Message::ConnectionTested(result) => {
                self.account_setup.is_testing = false;
                self.account_setup.test_result = Some(result.clone());
                if let Err(e) = result {
                    self.account_setup.save_error = Some(format!("Connection failed: {e}"));
                } else {
                    self.account_setup.save_error = Some("Connection successful!".to_string());
                }
            }
            Message::LoadAccount => {
                return Task::perform(load_account(), Message::AccountLoaded);
            }
            Message::AccountLoaded(result) => match result {
                Ok(Some(account)) => {
                    info!("Account loaded: {}", account.email);
                    self.current_account = Some(account.clone());
                    self.is_loading_folders = true;
                    self.error_message = None;
                    return Task::perform(load_folders(account), Message::FoldersLoaded);
                }
                Ok(None) => {
                    info!("No account configured, showing account setup");
                    self.current_view = View::AccountSetup;
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to load account: {e}"));
                }
            },
            Message::SettingsLoaded(result) => match result {
                Ok(settings) => {
                    info!("Settings loaded: theme={:?}", settings.theme_mode);
                    self.theme_mode = settings.theme_mode;
                    self.apply_theme();
                }
                Err(e) => {
                    info!("Failed to load settings, using defaults: {}", e);
                    // Keep default settings
                }
            },
            Message::SettingsSaved(result) => {
                if let Err(e) = result {
                    self.error_message = Some(format!("Failed to save settings: {e}"));
                }
            }
            Message::FoldersLoaded(result) => {
                self.is_loading_folders = false;
                match result {
                    Ok(folders) => {
                        // Build folder path mapping
                        self.folder_paths.clear();
                        for folder in &folders {
                            self.folder_paths.insert(folder.id, folder.path.clone());
                        }

                        self.folders = folders;

                        // Auto-select inbox if present
                        if let Some(inbox) = self
                            .folders
                            .iter()
                            .find(|f| f.folder_type == FolderType::Inbox)
                        {
                            self.selected_folder = Some(inbox.id);
                            // Load messages for inbox
                            if let Some(account) = self.current_account.clone() {
                                let folder_path = inbox.path.clone();
                                let folder_id = inbox.id;
                                self.is_loading_messages = true;
                                return Task::perform(
                                    load_messages(account, folder_path, folder_id),
                                    Message::MessagesLoaded,
                                );
                            }
                        }
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to load folders: {e}"));
                    }
                }
            }
            Message::MessagesLoaded(result) => {
                self.is_loading_messages = false;
                match result {
                    Ok(messages) => {
                        self.all_messages = messages;
                        self.filter_messages();
                        self.selected_message = None;
                        self.message_content = None;
                        // Start IDLE monitoring after messages are loaded
                        return Task::done(Message::StartIdle);
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to load messages: {e}"));
                    }
                }
            }
            Message::StartIdle => {
                // Start IDLE monitoring on the current folder
                if let Some(account) = self.current_account.clone()
                    && let Some(folder_id) = self.selected_folder
                    && let Some(folder_path) = self.folder_paths.get(&folder_id).cloned()
                    && !self.is_idle_active
                {
                    self.is_idle_active = true;
                    info!("Starting IDLE monitoring on {}", folder_path);
                    return Task::perform(start_idle(account, folder_path), Message::IdleReceived);
                }
            }
            Message::IdleReceived(result) => {
                self.is_idle_active = false;
                match result {
                    Ok(event) => {
                        info!("IDLE event received: {:?}", event);
                        match event {
                            mailledger_core::IdleEvent::NewMail(_) => {
                                // New mail arrived - refresh messages
                                if let Some(account) = self.current_account.clone()
                                    && let Some(folder_id) = self.selected_folder
                                    && let Some(folder_path) =
                                        self.folder_paths.get(&folder_id).cloned()
                                {
                                    self.is_loading_messages = true;
                                    return Task::perform(
                                        load_messages(account, folder_path, folder_id),
                                        Message::MessagesLoaded,
                                    );
                                }
                            }
                            mailledger_core::IdleEvent::Expunge
                            | mailledger_core::IdleEvent::FlagsChanged => {
                                // Refresh messages on expunge or flag change
                                if let Some(account) = self.current_account.clone()
                                    && let Some(folder_id) = self.selected_folder
                                    && let Some(folder_path) =
                                        self.folder_paths.get(&folder_id).cloned()
                                {
                                    self.is_loading_messages = true;
                                    return Task::perform(
                                        load_messages(account, folder_path, folder_id),
                                        Message::MessagesLoaded,
                                    );
                                }
                            }
                            mailledger_core::IdleEvent::Timeout
                            | mailledger_core::IdleEvent::Disconnected(_) => {
                                // Restart IDLE on timeout or disconnect
                                return Task::done(Message::StartIdle);
                            }
                        }
                    }
                    Err(e) => {
                        // Log error but try to restart IDLE
                        info!("IDLE error: {}, restarting...", e);
                        return Task::done(Message::StartIdle);
                    }
                }
            }
            Message::KeyPressed(action) => {
                return self.handle_keyboard_action(action);
            }
        }
        Task::none()
    }

    /// Handle keyboard shortcut actions.
    fn handle_keyboard_action(&mut self, action: KeyboardAction) -> Task<Message> {
        match action {
            KeyboardAction::ComposeNew => {
                self.compose_state = ComposeState::new();
                self.current_view = View::Compose;
            }
            KeyboardAction::Reply => {
                if self.current_view == View::Inbox && self.message_content.is_some() {
                    return Task::done(Message::Reply);
                }
            }
            KeyboardAction::ReplyAll => {
                if self.current_view == View::Inbox && self.message_content.is_some() {
                    return Task::done(Message::ReplyAll);
                }
            }
            KeyboardAction::Forward => {
                if self.current_view == View::Inbox && self.message_content.is_some() {
                    return Task::done(Message::Forward);
                }
            }
            KeyboardAction::Delete => {
                if self.current_view == View::Inbox
                    && let Some(message_id) = self.selected_message
                {
                    return Task::done(Message::DeleteMessage(message_id));
                }
            }
            KeyboardAction::Refresh => {
                if self.current_view == View::Inbox {
                    return Task::done(Message::RefreshFolders);
                }
            }
            KeyboardAction::Cancel => match self.current_view {
                View::Compose => {
                    self.compose_state = ComposeState::new();
                    self.current_view = View::Inbox;
                }
                View::Settings => {
                    self.current_view = View::Inbox;
                }
                View::AccountSetup => {
                    // Don't allow cancel if no account configured
                    if self.current_account.is_some() {
                        self.current_view = View::Inbox;
                    }
                }
                View::Inbox => {
                    // Clear selection
                    self.selected_message = None;
                    self.message_content = None;
                }
            },
            KeyboardAction::Send => {
                if self.current_view == View::Compose {
                    return Task::done(Message::Compose(ComposeMessage::Send));
                }
            }
            KeyboardAction::Settings => {
                self.current_view = View::Settings;
            }
            KeyboardAction::ToggleSidebar => {
                self.sidebar_visible = !self.sidebar_visible;
            }
        }
        Task::none()
    }

    /// Handle account setup messages.
    fn handle_account_setup(&mut self, msg: AccountSetupMessage) -> Task<Message> {
        match msg {
            AccountSetupMessage::NameChanged(name) => {
                self.account_setup.name = name;
            }
            AccountSetupMessage::EmailChanged(email) => {
                self.account_setup.email.clone_from(&email);
                // Auto-detect settings when email changes
                if email.contains('@') {
                    self.account_setup.auto_detect_from_email();
                }
            }
            AccountSetupMessage::ImapHostChanged(host) => {
                self.account_setup.imap_host = host;
            }
            AccountSetupMessage::ImapPortChanged(port) => {
                self.account_setup.imap_port = port;
            }
            AccountSetupMessage::ImapSecurityChanged(security) => {
                self.account_setup.imap_security = security;
            }
            AccountSetupMessage::ImapUsernameChanged(username) => {
                self.account_setup.imap_username = username;
            }
            AccountSetupMessage::ImapPasswordChanged(password) => {
                self.account_setup.imap_password = password;
            }
            AccountSetupMessage::SmtpHostChanged(host) => {
                self.account_setup.smtp_host = host;
            }
            AccountSetupMessage::SmtpPortChanged(port) => {
                self.account_setup.smtp_port = port;
            }
            AccountSetupMessage::SmtpSecurityChanged(security) => {
                self.account_setup.smtp_security = security;
            }
            AccountSetupMessage::SmtpUsernameChanged(username) => {
                self.account_setup.smtp_username = username;
            }
            AccountSetupMessage::SmtpPasswordChanged(password) => {
                self.account_setup.smtp_password = password;
            }
            AccountSetupMessage::Save => {
                if self.account_setup.validate() {
                    self.account_setup.is_saving = true;
                    self.account_setup.save_error = None;
                    let account = self.account_setup.to_account();
                    return Task::perform(save_account(account), Message::AccountSaved);
                }
            }
            AccountSetupMessage::TestConnection => {
                if self.account_setup.validate() {
                    self.account_setup.is_testing = true;
                    self.account_setup.save_error = None;
                    let account = self.account_setup.to_account();
                    return Task::perform(test_connection(account), Message::ConnectionTested);
                }
            }
            AccountSetupMessage::Cancel => {
                self.current_view = View::Inbox;
            }
        }
        Task::none()
    }

    /// Handle compose messages.
    fn handle_compose(&mut self, msg: ComposeMessage) -> Task<Message> {
        match msg {
            ComposeMessage::ToChanged(to) => {
                self.compose_state.to = to;
            }
            ComposeMessage::CcChanged(cc) => {
                self.compose_state.cc = cc;
            }
            ComposeMessage::BccChanged(bcc) => {
                self.compose_state.bcc = bcc;
            }
            ComposeMessage::SubjectChanged(subject) => {
                self.compose_state.subject = subject;
            }
            ComposeMessage::BodyChanged(body) => {
                self.compose_state.body = body;
            }
            ComposeMessage::Send => {
                if let Some(error) = self.compose_state.validate() {
                    self.compose_state.send_error = Some(error);
                } else if let Some(account) = self.current_account.clone() {
                    self.compose_state.is_sending = true;
                    self.compose_state.send_error = None;
                    self.compose_state.send_success = false;
                    let message = self.compose_state.to_outgoing(&account.email);
                    return Task::perform(send_email(account, message), Message::EmailSent);
                } else {
                    self.compose_state.send_error =
                        Some("No account configured. Please set up an account first.".to_string());
                }
            }
            ComposeMessage::Cancel => {
                self.compose_state = ComposeState::new();
                self.current_view = View::Inbox;
            }
        }
        Task::none()
    }

    /// Handle settings messages.
    fn handle_settings(&mut self, msg: SettingsMessage) -> Task<Message> {
        match msg {
            SettingsMessage::SelectSection(section) => {
                self.settings_state.selected_section = section;
            }
            SettingsMessage::ToggleTheme => {
                self.theme_mode = match self.theme_mode {
                    ThemeMode::Light => ThemeMode::Dark,
                    ThemeMode::Dark => ThemeMode::Light,
                };
                self.apply_theme();
                info!("Theme changed to {:?}", self.theme_mode);
                // Save settings
                let settings = AppSettings {
                    theme_mode: self.theme_mode,
                };
                return Task::perform(save_settings(settings), Message::SettingsSaved);
            }
        }
        Task::none()
    }

    /// Render current state as UI.
    fn view(&self) -> Element<'_, Message> {
        match self.current_view {
            View::Inbox => self.view_inbox(),
            View::Compose => self.view_compose(),
            View::Settings => self.view_settings(),
            View::AccountSetup => self.view_account_setup(),
        }
    }

    /// Main inbox view with three-pane layout.
    fn view_inbox(&self) -> Element<'_, Message> {
        let header = view::view_header(&self.search_query);

        let mut main_content = row![];

        // Sidebar (folder list)
        if self.sidebar_visible {
            main_content =
                main_content.push(view::view_sidebar(&self.folders, self.selected_folder));
        }

        // Message list
        main_content = main_content.push(view::view_message_list(
            &self.messages,
            self.selected_message,
        ));

        // Message content
        main_content = main_content.push(view::view_message_content(self.message_content.as_ref()));

        column![header, main_content.height(Length::Fill)]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// Compose view.
    fn view_compose(&self) -> Element<'_, Message> {
        view::view_compose(&self.compose_state)
    }

    /// Settings view.
    fn view_settings(&self) -> Element<'_, Message> {
        view::view_settings(
            &self.settings_state,
            self.current_account.as_ref(),
            self.theme_mode,
        )
    }

    /// Account setup view.
    fn view_account_setup(&self) -> Element<'_, Message> {
        view::view_account_setup(&self.account_setup)
    }

    /// Subscribe to keyboard events for shortcuts.
    #[allow(clippy::unused_self)] // Required signature for iced subscription
    fn subscription(&self) -> Subscription<Message> {
        keyboard::listen().map(|event| {
            if let keyboard::Event::KeyPressed { key, modifiers, .. } = event {
                handle_key_press(key, modifiers).unwrap_or(Message::WindowResized(0, 0))
            } else {
                Message::WindowResized(0, 0) // Ignore other keyboard events
            }
        })
    }
}

/// Handle keyboard shortcuts and return appropriate message.
fn handle_key_press(key: Key, modifiers: Modifiers) -> Option<Message> {
    let ctrl = modifiers.command(); // Ctrl on Linux/Windows, Cmd on macOS
    let shift = modifiers.shift();

    match key {
        // Ctrl+N: Compose new message
        Key::Character(c) if ctrl && !shift && c.as_str() == "n" => {
            Some(Message::KeyPressed(KeyboardAction::ComposeNew))
        }
        // Ctrl+R: Reply (without shift)
        Key::Character(c) if ctrl && !shift && c.as_str() == "r" => {
            Some(Message::KeyPressed(KeyboardAction::Reply))
        }
        // Ctrl+Shift+R: Reply All
        Key::Character(c) if ctrl && shift && c.as_str() == "r" => {
            Some(Message::KeyPressed(KeyboardAction::ReplyAll))
        }
        // Ctrl+Shift+F: Forward
        Key::Character(c) if ctrl && shift && c.as_str() == "f" => {
            Some(Message::KeyPressed(KeyboardAction::Forward))
        }
        // Ctrl+Enter: Send (in compose view)
        Key::Named(keyboard::key::Named::Enter) if ctrl => {
            Some(Message::KeyPressed(KeyboardAction::Send))
        }
        // Escape: Cancel
        Key::Named(keyboard::key::Named::Escape) => {
            Some(Message::KeyPressed(KeyboardAction::Cancel))
        }
        // Delete: Delete selected message
        Key::Named(keyboard::key::Named::Delete) => {
            Some(Message::KeyPressed(KeyboardAction::Delete))
        }
        // F5: Refresh
        Key::Named(keyboard::key::Named::F5) => Some(Message::KeyPressed(KeyboardAction::Refresh)),
        // Ctrl+B: Toggle sidebar
        Key::Character(c) if ctrl && !shift && c.as_str() == "b" => {
            Some(Message::KeyPressed(KeyboardAction::ToggleSidebar))
        }
        // Ctrl+,: Settings
        Key::Character(c) if ctrl && c.as_str() == "," => {
            Some(Message::KeyPressed(KeyboardAction::Settings))
        }
        _ => None,
    }
}

/// Load application settings from file.
async fn load_settings() -> Result<AppSettings, String> {
    let settings_path = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("mailledger")
        .join("settings.json");

    if !settings_path.exists() {
        return Ok(AppSettings::default());
    }

    let contents = tokio::fs::read_to_string(&settings_path)
        .await
        .map_err(|e| e.to_string())?;

    serde_json::from_str(&contents).map_err(|e| e.to_string())
}

/// Save application settings to file.
async fn save_settings(settings: AppSettings) -> Result<(), String> {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("mailledger");

    tokio::fs::create_dir_all(&config_dir)
        .await
        .map_err(|e| e.to_string())?;

    let settings_path = config_dir.join("settings.json");
    let contents = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;

    tokio::fs::write(&settings_path, contents)
        .await
        .map_err(|e| e.to_string())?;

    tracing::info!("Settings saved to {:?}", settings_path);
    Ok(())
}

/// Save account to database.
async fn save_account(account: mailledger_core::Account) -> Result<(), String> {
    use mailledger_core::AccountRepository;

    // Get data directory
    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("mailledger");

    // Create directory if it doesn't exist
    std::fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;

    let db_path = data_dir.join("mailledger.db");
    let repo = AccountRepository::new(db_path.to_str().unwrap_or("mailledger.db"))
        .await
        .map_err(|e| e.to_string())?;

    let mut account = account;
    repo.save(&mut account).await.map_err(|e| e.to_string())?;

    tracing::info!("Account saved: {}", account.email);
    Ok(())
}

/// Test IMAP connection.
async fn test_connection(account: mailledger_core::Account) -> Result<(), String> {
    use mailledger_imap::connection::{Security, connect_tls};

    let security = match account.imap.security {
        mailledger_core::Security::Tls => Security::Implicit,
        mailledger_core::Security::StartTls => Security::StartTls,
        mailledger_core::Security::None => Security::None,
    };

    // For now, only support implicit TLS
    if security != Security::Implicit {
        return Err("Only SSL/TLS connections are currently supported".to_string());
    }

    let stream = connect_tls(&account.imap.host, account.imap.port)
        .await
        .map_err(|e| e.to_string())?;

    let client = mailledger_imap::connection::Client::from_stream(stream)
        .await
        .map_err(|e| e.to_string())?;

    let _auth_client = client
        .login(&account.imap.username, &account.imap.password)
        .await
        .map_err(|e| e.to_string())?;

    tracing::info!("Connection test successful for {}", account.email);
    Ok(())
}

/// Load default account from database.
async fn load_account() -> Result<Option<mailledger_core::Account>, String> {
    use mailledger_core::AccountRepository;

    // Get data directory
    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("mailledger");

    let db_path = data_dir.join("mailledger.db");

    // Check if database exists
    if !db_path.exists() {
        return Ok(None);
    }

    let repo = AccountRepository::new(db_path.to_str().unwrap_or("mailledger.db"))
        .await
        .map_err(|e| e.to_string())?;

    // Get default account or first account
    let account = repo.get_default().await.map_err(|e| e.to_string())?;

    if account.is_some() {
        return Ok(account);
    }

    // If no default, get first account
    let accounts = repo.list().await.map_err(|e| e.to_string())?;
    Ok(accounts.into_iter().next())
}

/// Load folders from IMAP server.
async fn load_folders(account: mailledger_core::Account) -> Result<Vec<Folder>, String> {
    use mailledger_core::{connect_and_login, list_folders};

    let mut client = connect_and_login(&account)
        .await
        .map_err(|e| e.to_string())?;

    let core_folders = list_folders(&mut client).await.map_err(|e| e.to_string())?;

    // Convert core folders to GUI folders with sequential IDs
    let folders: Vec<Folder> = core_folders
        .iter()
        .enumerate()
        .filter(|(_, f)| f.selectable) // Only show selectable folders
        .map(|(idx, f)| {
            #[allow(clippy::cast_possible_truncation)]
            Folder::from_core((idx + 1) as u32, f)
        })
        .collect();

    tracing::info!("Loaded {} folders", folders.len());
    Ok(folders)
}

/// Load messages from a folder.
async fn load_messages(
    account: mailledger_core::Account,
    folder_path: String,
    folder_id: FolderId,
) -> Result<Vec<MessageSummary>, String> {
    use mailledger_core::{connect_and_login, fetch_messages, select_folder};
    use mailledger_imap::types::UidSet;

    let client = connect_and_login(&account)
        .await
        .map_err(|e| e.to_string())?;

    let (mut selected_client, status) = select_folder(client, &folder_path)
        .await
        .map_err(|e| e.to_string())?;

    // Get the number of messages
    let total = status.exists;
    if total == 0 {
        return Ok(Vec::new());
    }

    // Fetch the most recent messages (up to 50)
    let fetch_count = total.min(50);
    let start_uid = if total > fetch_count {
        total - fetch_count + 1
    } else {
        1
    };

    let uid_set = UidSet::range(
        mailledger_imap::types::Uid::new(start_uid).ok_or("Invalid UID")?,
        mailledger_imap::types::Uid::new(total).ok_or("Invalid UID")?,
    );

    let core_messages = fetch_messages(&mut selected_client, &uid_set)
        .await
        .map_err(|e| e.to_string())?;

    // Convert core messages to GUI messages
    let messages: Vec<MessageSummary> = core_messages
        .iter()
        .map(|m| MessageSummary::from_core(folder_id, m))
        .collect();

    tracing::info!("Loaded {} messages from {}", messages.len(), folder_path);
    Ok(messages)
}

/// Send an email via SMTP.
async fn send_email(
    account: mailledger_core::Account,
    message: mailledger_core::OutgoingMessage,
) -> Result<(), String> {
    mailledger_core::send_email(&account, message)
        .await
        .map_err(|e| e.to_string())
}

/// Start IDLE monitoring on a folder.
///
/// Uses a 5-minute timeout (RFC 2177 recommends re-issuing IDLE every 29 minutes,
/// but we use shorter intervals for responsiveness and to handle server timeouts).
async fn start_idle(
    account: mailledger_core::Account,
    folder_path: String,
) -> Result<mailledger_core::IdleEvent, String> {
    const IDLE_TIMEOUT_SECS: u64 = 300; // 5 minutes

    mailledger_core::idle_monitor(&account, &folder_path, IDLE_TIMEOUT_SECS)
        .await
        .map_err(|e| e.to_string())
}

/// Load full message content from IMAP.
async fn load_message_content(
    account: mailledger_core::Account,
    folder_path: String,
    uid: u32,
) -> Result<Option<MessageContent>, String> {
    use mailledger_core::{connect_and_login, fetch_message_content, select_folder};
    use mailledger_imap::types::Uid;

    let client = connect_and_login(&account)
        .await
        .map_err(|e| e.to_string())?;

    let (mut selected_client, _status) = select_folder(client, &folder_path)
        .await
        .map_err(|e| e.to_string())?;

    let imap_uid = Uid::new(uid).ok_or("Invalid UID")?;

    let content = fetch_message_content(&mut selected_client, imap_uid)
        .await
        .map_err(|e| e.to_string())?;

    Ok(content.map(|c| MessageContent::from_core(&c)))
}
