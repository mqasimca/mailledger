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

use iced::event::{self, Event};
use iced::keyboard::{self, Key, Modifiers};
use iced::mouse;
use iced::widget::{Space, column, container, image, markdown, row, text, text_editor};
use iced::{Background, Border, Element, Length, Subscription, Task};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use std::collections::{HashMap, HashSet};

use message::{
    AccountSetupMessage, ComposeMessage, KeyboardAction, Message, PaneDivider, ScreenerMessage,
    SearchFilter, SettingsMessage, View,
};
use model::{
    AccountSetupState, AppSettings, AutocompleteField, ComposeState, Folder, FolderId, FolderType,
    FontSize, InlineImage, InlineImageState, ListDensity, MessageContent, MessageId,
    MessageSummary, SettingsState, Thread, ViewMode, group_into_threads,
};
use style::widgets::palette::{self, ThemeMode};
use style::widgets::radius;
use view::PendingSender;

/// Sanitizes a filename to prevent path traversal attacks.
///
/// This function removes or replaces dangerous characters that could be used
/// to escape the intended download directory via path traversal sequences.
///
/// # Security
///
/// Prevents CWE-22 (Path Traversal) by:
/// - Removing path separators (`/`, `\`)
/// - Removing null bytes
/// - Stripping leading dots (hidden files)
/// - Removing control characters
/// - Limiting length to 255 characters (filesystem limit)
///
/// # Examples
///
/// ```
/// assert_eq!(sanitize_filename("../../etc/passwd"), "etcpasswd");
/// assert_eq!(sanitize_filename("..\\..\\windows\\system32"), "windowssystem32");
/// assert_eq!(sanitize_filename(".hidden"), "hidden");
/// ```
fn sanitize_filename(filename: &str) -> String {
    // Split by path separators and null bytes, then join segments
    let segments: Vec<&str> = filename.split(['/', '\\', '\0']).collect();

    segments
        .iter()
        .flat_map(|segment| segment.chars())
        .filter(|c| !c.is_control()) // Remove control characters
        .take(255) // Limit to filesystem max length
        .collect::<String>()
        .trim_start_matches('.') // Remove leading dots from final result
        .trim() // Remove leading/trailing whitespace
        .to_string()
}

fn main() -> iced::Result {
    if rustls::crypto::ring::default_provider()
        .install_default()
        .is_err()
    {
        eprintln!("Failed to install rustls crypto provider");
        std::process::exit(1);
    }

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
        .theme(MailLedger::theme)
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
    /// Parsed markdown items for message body display.
    markdown_items: Vec<markdown::Item>,
    /// Inline images extracted from message HTML.
    inline_images: Vec<InlineImage>,
    /// Search query.
    search_query: String,
    /// Account setup state.
    account_setup: AccountSetupState,
    /// Compose message state.
    compose_state: ComposeState,
    /// Settings state.
    settings_state: SettingsState,
    /// All configured accounts.
    accounts: Vec<mailledger_core::Account>,
    /// Currently active account ID.
    active_account_id: Option<mailledger_core::AccountId>,
    /// Current account (if logged in) - convenience accessor.
    current_account: Option<mailledger_core::Account>,
    /// Whether account switcher dropdown is open.
    account_switcher_open: bool,
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
    /// Pending senders for the Screener.
    pending_senders: Vec<PendingSender>,
    /// Whether we're loading pending senders.
    is_loading_screener: bool,
    /// View mode for message list (flat or threaded).
    view_mode: ViewMode,
    /// Threads for threaded view.
    threads: Vec<Thread>,
    /// Expanded thread IDs.
    expanded_threads: std::collections::HashSet<String>,
    /// Compose body editor content (stored separately as Content doesn't impl Clone).
    compose_body: text_editor::Content,
    /// Whether quoted text in message view is expanded.
    quoted_expanded: bool,
    /// Whether snooze dropdown is open.
    snooze_dropdown_open: bool,
    /// Active search filters.
    search_filters: HashSet<SearchFilter>,
    /// Font size preference.
    font_size: FontSize,
    /// List density preference.
    list_density: ListDensity,
    /// Message list scroll offset for virtual scrolling.
    message_list_scroll_offset: f32,
    /// Message list viewport height for virtual scrolling.
    message_list_viewport_height: f32,
    /// Sidebar width for resizable panes.
    sidebar_width: f32,
    /// Message list width for resizable panes.
    message_list_width: f32,
    /// Currently dragging pane divider.
    dragging_divider: Option<PaneDivider>,
    /// Whether we're currently offline (no server connection).
    is_offline: bool,
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
            markdown_items: Vec::new(),
            inline_images: Vec::new(),
            search_query: String::new(),
            account_setup: AccountSetupState::new(),
            compose_state: ComposeState::new(),
            settings_state: SettingsState::new(),
            accounts: Vec::new(),
            active_account_id: None,
            current_account: None,
            account_switcher_open: false,
            folder_paths: HashMap::new(),
            is_loading_folders: false,
            is_loading_messages: false,
            error_message: None,
            is_idle_active: false,
            theme_mode: ThemeMode::Dark, // Default to dark mode
            pending_senders: Vec::new(),
            is_loading_screener: false,
            view_mode: ViewMode::Flat,
            threads: Vec::new(),
            expanded_threads: std::collections::HashSet::new(),
            compose_body: text_editor::Content::new(),
            quoted_expanded: false,
            snooze_dropdown_open: false,
            search_filters: HashSet::new(),
            font_size: FontSize::Medium,
            list_density: ListDensity::Comfortable,
            message_list_scroll_offset: 0.0,
            message_list_viewport_height: 600.0, // Default viewport height
            sidebar_width: 220.0,
            message_list_width: 380.0,
            dragging_divider: None,
            is_offline: false,
        }
    }
}

impl MailLedger {
    /// Returns the iced theme based on current theme mode.
    #[allow(clippy::missing_const_for_fn)] // Cannot be const - match isn't const
    fn theme(&self) -> iced::Theme {
        match self.theme_mode {
            ThemeMode::Light => iced::Theme::Light,
            ThemeMode::Dark => iced::Theme::Dark,
        }
    }

    /// Applies the current theme mode to the global palette.
    fn apply_theme(&self) {
        style::widgets::palette::set_theme(self.theme_mode);
    }

    /// Filter messages based on the current search query and filters.
    fn filter_messages(&mut self) {
        // Start with all messages
        let mut filtered: Vec<MessageSummary> = self.all_messages.clone();

        // Apply search query filter
        if !self.search_query.is_empty() {
            let query = self.search_query.to_lowercase();
            filtered.retain(|msg| {
                msg.subject.to_lowercase().contains(&query)
                    || msg.from_name.to_lowercase().contains(&query)
                    || msg.from_email.to_lowercase().contains(&query)
                    || msg.snippet.to_lowercase().contains(&query)
            });
        }

        // Apply quick filters
        for filter in &self.search_filters {
            match filter {
                SearchFilter::Unread => {
                    filtered.retain(|msg| !msg.is_read);
                }
                SearchFilter::Flagged => {
                    filtered.retain(|msg| msg.is_flagged);
                }
                SearchFilter::HasAttachments => {
                    filtered.retain(|msg| msg.has_attachments);
                }
            }
        }

        self.messages = filtered;
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
                    let mut setup = AccountSetupState::new();
                    if let Some(account) = self.current_account.as_ref() {
                        setup.load_from_account(account);
                    }
                    self.account_setup = setup;
                } else if view == View::Screener {
                    // Load pending senders when navigating to Screener
                    if let Some(account) = self.current_account.as_ref()
                        && let Some(account_id) = account.id
                    {
                        self.is_loading_screener = true;
                        return Task::perform(
                            load_pending_senders(account_id),
                            Message::PendingSendersLoaded,
                        );
                    }
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
            Message::RefreshMessages => {
                // Reload messages from the current folder
                if let Some(account) = self.current_account.clone()
                    && let Some(folder_id) = self.selected_folder
                    && let Some(folder) = self.folders.iter().find(|f| f.id == folder_id)
                {
                    let folder_path = folder.path.clone();
                    self.is_loading_messages = true;
                    return Task::perform(
                        load_messages(account, folder_path, folder_id),
                        Message::MessagesLoaded,
                    );
                }
            }
            Message::SearchQueryChanged(query) => {
                self.search_query = query;
                // Filter messages locally for instant feedback
                self.filter_messages();
            }
            Message::SearchExecute => {
                // If empty search, just filter locally
                if self.search_query.is_empty() && self.search_filters.is_empty() {
                    self.filter_messages();
                } else if let Some(account) = self.current_account.clone() {
                    // Use IMAP SEARCH for server-side search
                    let criteria = self.build_search_criteria();
                    let folder_path = self
                        .selected_folder
                        .and_then(|id| self.folder_paths.get(&id))
                        .cloned()
                        .unwrap_or_else(|| "INBOX".to_string());
                    return Task::perform(
                        execute_search(account, folder_path, criteria),
                        Message::SearchResultsLoaded,
                    );
                } else {
                    // No account, just filter locally
                    self.filter_messages();
                }
            }
            Message::SearchResultsLoaded(result) => match result {
                Ok(uids) => {
                    // Filter messages to only show those matching the UIDs
                    self.messages = self
                        .all_messages
                        .iter()
                        .filter(|msg| uids.iter().any(|uid| uid.get() == msg.id.0))
                        .cloned()
                        .collect();
                }
                Err(e) => {
                    tracing::warn!("Search failed: {}, falling back to local filter", e);
                    self.filter_messages();
                }
            },
            Message::ToggleSearchFilter(filter) => {
                if self.search_filters.contains(&filter) {
                    self.search_filters.remove(&filter);
                } else {
                    self.search_filters.insert(filter);
                }
                self.filter_messages();
            }
            Message::ToggleViewMode => {
                self.view_mode = match self.view_mode {
                    ViewMode::Flat => ViewMode::Threaded,
                    ViewMode::Threaded => ViewMode::Flat,
                };
                // Rebuild threads when switching to threaded view
                if self.view_mode == ViewMode::Threaded {
                    self.threads = group_into_threads(&self.all_messages);
                }
            }
            Message::ToggleThread(thread_id) => {
                if self.expanded_threads.contains(&thread_id) {
                    self.expanded_threads.remove(&thread_id);
                } else {
                    self.expanded_threads.insert(thread_id);
                }
            }
            Message::ToggleQuotedText => {
                self.quoted_expanded = !self.quoted_expanded;
            }
            Message::WindowResized(_, _) => {
                // No action needed
            }
            Message::MessageListScrolled(viewport) => {
                self.message_list_scroll_offset = viewport.absolute_offset().y;
                self.message_list_viewport_height = viewport.bounds().height;
            }
            Message::StartPaneDrag(divider) => {
                self.dragging_divider = Some(divider);
            }
            Message::StopPaneDrag => {
                self.dragging_divider = None;
            }
            Message::PaneDragMoved(x) => {
                if let Some(divider) = self.dragging_divider {
                    match divider {
                        PaneDivider::SidebarMessageList => {
                            // Sidebar width = x position (constrained to 150-400)
                            self.sidebar_width = x.clamp(150.0, 400.0);
                        }
                        PaneDivider::MessageListMessageView => {
                            // Message list width = x - sidebar_width (constrained to 250-600)
                            let new_width = x - self.sidebar_width;
                            self.message_list_width = new_width.clamp(250.0, 600.0);
                        }
                    }
                }
            }
            Message::SelectMessage(message_id) => {
                self.selected_message = Some(message_id);
                self.message_content = None; // Clear while loading
                self.inline_images.clear();
                self.quoted_expanded = false; // Reset quote expansion for new message

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
            Message::MessageContentLoaded(result) => {
                let mut html_body = None;
                let mut cache_task = Task::none();
                match result {
                    Ok(Some(content)) => {
                        html_body.clone_from(&content.body_html);
                        self.is_offline = false;

                        // Cache content for offline use
                        if let Some(account) = self.current_account.as_ref()
                            && let Some(account_id) = account.id
                            && let Some(folder_id) = self.selected_folder
                            && let Some(folder_path) = self.folder_paths.get(&folder_id).cloned()
                        {
                            cache_task = Task::perform(
                                cache_message_content(account_id, folder_path, content.clone()),
                                |_| Message::WindowResized(0, 0), // Ignore result
                            );
                        }
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
                        // Try loading from cache when offline
                        if let Some(account) = self.current_account.as_ref()
                            && let Some(account_id) = account.id
                            && let Some(folder_id) = self.selected_folder
                            && let Some(folder_path) = self.folder_paths.get(&folder_id).cloned()
                            && let Some(selected_id) = self.selected_message
                        {
                            self.is_offline = true;
                            return Task::perform(
                                load_cached_content(account_id, folder_path, selected_id.0),
                                Message::CachedContentLoaded,
                            );
                        }
                        // Fallback to mock content on error if no cache
                        if let Some(summary) = self
                            .selected_message
                            .and_then(|id| self.messages.iter().find(|m| m.id == id))
                        {
                            self.message_content = Some(MessageContent::mock_content(summary));
                        }
                    }
                }

                // Parse markdown from content for rich display
                self.markdown_items =
                    self.message_content
                        .as_ref()
                        .map_or_else(Vec::new, |content| {
                            let md_text = content_to_markdown(content);
                            markdown::parse(&md_text).collect()
                        });

                self.inline_images.clear();
                if let Some(html) = html_body {
                    let urls = extract_image_urls(&html);
                    let (inline_images, task) = prepare_inline_images(urls);
                    self.inline_images = inline_images;
                    return Task::batch([task, cache_task]);
                }
                return cache_task;
            }
            Message::InlineImageLoaded { url, result } => {
                if let Some(entry) = self.inline_images.iter_mut().find(|img| img.url == url) {
                    match result {
                        Ok(bytes) => {
                            entry.state = InlineImageState::Ready(image::Handle::from_bytes(bytes));
                        }
                        Err(err) => {
                            entry.state = InlineImageState::Failed(err);
                        }
                    }
                }
            }
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
            Message::ArchiveMessage(message_id) => {
                // Find the archive folder path
                let archive_folder = self
                    .folders
                    .iter()
                    .find(|f| f.folder_type == FolderType::Archive)
                    .map(|f| f.path.clone());

                let Some(archive_path) = archive_folder else {
                    self.error_message = Some("No Archive folder found".to_string());
                    return Task::none();
                };

                if let Some(account) = self.current_account.clone()
                    && let Some(folder_id) = self.selected_folder
                    && let Some(folder_path) = self.folder_paths.get(&folder_id).cloned()
                {
                    // Remove from local list immediately for snappy UI
                    self.messages.retain(|m| m.id != message_id);
                    self.all_messages.retain(|m| m.id != message_id);
                    if self.selected_message == Some(message_id) {
                        self.selected_message = None;
                        self.message_content = None;
                    }

                    return Task::perform(
                        archive_message(account, folder_path, message_id.0, archive_path),
                        Message::MessageArchived,
                    );
                }
            }
            Message::ArchiveSelected => {
                if let Some(message_id) = self.selected_message {
                    return Task::done(Message::ArchiveMessage(message_id));
                }
            }
            Message::MessageArchived(result) => {
                if let Err(e) = result {
                    self.error_message = Some(format!("Failed to archive message: {e}"));
                    // Refresh to restore the message if archive failed
                    return Task::done(Message::RefreshMessages);
                }
            }
            Message::DownloadAttachment {
                message_id,
                part_number,
                filename,
                encoding,
            } => {
                if let Some(account) = self.current_account.clone()
                    && let Some(folder_id) = self.selected_folder
                    && let Some(folder_path) = self.folder_paths.get(&folder_id).cloned()
                {
                    return Task::perform(
                        download_attachment_task(
                            account,
                            folder_path,
                            message_id.0,
                            part_number,
                            filename,
                            encoding,
                        ),
                        Message::AttachmentDownloaded,
                    );
                }
            }
            Message::AttachmentDownloaded(result) => match result {
                Ok((filename, data)) => {
                    // Use native file dialog to save the attachment
                    if let Some(downloads_dir) = dirs::download_dir() {
                        // Sanitize filename to prevent path traversal attacks (CWE-22)
                        let safe_filename = sanitize_filename(&filename);

                        // Ensure sanitization didn't result in an empty filename
                        let final_filename = if safe_filename.is_empty() {
                            "attachment".to_string()
                        } else {
                            safe_filename
                        };

                        // Warn if filename was modified during sanitization
                        if final_filename != filename {
                            tracing::warn!(
                                "Sanitized potentially malicious filename: '{}' -> '{}'",
                                filename,
                                final_filename
                            );
                        }

                        let save_path = downloads_dir.join(&final_filename);
                        if let Err(e) = std::fs::write(&save_path, &data) {
                            self.error_message = Some(format!("Failed to save attachment: {e}"));
                        } else {
                            info!("Saved attachment to {:?}", save_path);
                            // Try to open the file manager at the download location
                            if let Err(e) = opener::open(downloads_dir) {
                                tracing::warn!("Failed to open downloads folder: {}", e);
                            }
                        }
                    } else {
                        self.error_message = Some("Could not find downloads directory".to_string());
                    }
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to download attachment: {e}"));
                }
            },
            Message::ComposeNew => {
                self.compose_state = ComposeState::new();
                self.compose_body = text_editor::Content::new();
                self.current_view = View::Compose;
            }
            Message::OpenHtml => {
                if let Some(content) = self.message_content.as_ref()
                    && let Some(html) = content.body_html.as_ref()
                {
                    return Task::perform(
                        open_html_message(html.clone(), content.id.0, content.subject.clone()),
                        Message::HtmlOpened,
                    );
                }
            }
            Message::HtmlOpened(result) => {
                if let Err(err) = result {
                    self.error_message = Some(format!("Failed to open HTML: {err}"));
                }
            }
            Message::LinkClicked(url) => {
                // Open links in the default browser
                if let Err(err) = opener::open(url.as_str()) {
                    self.error_message = Some(format!("Failed to open link: {err}"));
                }
            }
            Message::Reply => {
                if let Some(ref content) = self.message_content {
                    let quoted = content.body_text.as_deref().unwrap_or("");
                    self.compose_state =
                        ComposeState::reply(&content.from_email, &content.subject, quoted);
                    // Pre-fill body editor with quoted text
                    let quoted_body = format!("\n\n> {}", quoted.replace('\n', "\n> "));
                    self.compose_body = text_editor::Content::with_text(&quoted_body);
                } else {
                    self.compose_state = ComposeState::new();
                    self.compose_body = text_editor::Content::new();
                }
                self.current_view = View::Compose;
            }
            Message::ReplyAll => {
                // For now, same as Reply
                if let Some(ref content) = self.message_content {
                    let quoted = content.body_text.as_deref().unwrap_or("");
                    self.compose_state =
                        ComposeState::reply(&content.from_email, &content.subject, quoted);
                    let quoted_body = format!("\n\n> {}", quoted.replace('\n', "\n> "));
                    self.compose_body = text_editor::Content::with_text(&quoted_body);
                } else {
                    self.compose_state = ComposeState::new();
                    self.compose_body = text_editor::Content::new();
                }
                self.current_view = View::Compose;
            }
            Message::Forward => {
                if let Some(ref content) = self.message_content {
                    let body = content.body_text.as_deref().unwrap_or("");
                    self.compose_state =
                        ComposeState::forward(&content.subject, body, &content.from_email);
                    let fwd_body = format!(
                        "\n\n---------- Forwarded message ----------\nFrom: {}\n\n{}",
                        content.from_email, body
                    );
                    self.compose_body = text_editor::Content::with_text(&fwd_body);
                } else {
                    self.compose_state = ComposeState::new();
                    self.compose_body = text_editor::Content::new();
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

                        // Record recipients as contacts for future autocomplete
                        let recipients: Vec<String> = self
                            .compose_state
                            .to
                            .split(',')
                            .chain(self.compose_state.cc.split(','))
                            .chain(self.compose_state.bcc.split(','))
                            .map(str::trim)
                            .filter(|s| !s.is_empty())
                            .map(String::from)
                            .collect();

                        if !recipients.is_empty() {
                            return Task::perform(record_contacts(recipients), |_| {
                                // No message needed - fire and forget
                                Message::RefreshMessages
                            });
                        }
                    }
                    Err(e) => {
                        self.compose_state.send_error = Some(e);
                    }
                }
            }
            Message::Settings(msg) => {
                return self.handle_settings(msg);
            }
            Message::Screener(msg) => {
                return self.handle_screener(msg);
            }
            Message::PendingSendersLoaded(result) => {
                self.is_loading_screener = false;
                match result {
                    Ok(senders) => {
                        self.pending_senders = senders.iter().map(PendingSender::from).collect();
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load pending senders: {}", e);
                    }
                }
            }
            Message::SenderDecisionSaved(result) => {
                if let Err(e) = result {
                    self.error_message = Some(format!("Failed to save sender decision: {e}"));
                } else if let Some(account) = self.current_account.as_ref()
                    && let Some(account_id) = account.id
                {
                    // Reload pending senders after a decision is made
                    return Task::perform(
                        load_pending_senders(account_id),
                        Message::PendingSendersLoaded,
                    );
                }
            }
            Message::RecordSender {
                email,
                display_name,
            } => {
                // Record a new sender when we receive an email
                if let Some(account) = self.current_account.as_ref()
                    && let Some(account_id) = account.id
                {
                    return Task::perform(
                        record_sender(account_id, email, display_name),
                        |_| Message::RefreshMessages, // Silently refresh
                    );
                }
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
            Message::LoadAccounts => {
                return Task::perform(load_all_accounts(), Message::AccountsLoaded);
            }
            Message::AccountsLoaded(result) => match result {
                Ok(accounts) => {
                    if accounts.is_empty() {
                        info!("No accounts configured, showing account setup");
                        self.current_view = View::AccountSetup;
                    } else {
                        info!("Loaded {} account(s)", accounts.len());
                        self.accounts = accounts;
                        // Select the first account by default
                        if let Some(account) = self.accounts.first() {
                            self.active_account_id = account.id;
                            self.current_account = Some(account.clone());
                            self.is_loading_folders = true;
                            self.error_message = None;
                            return Task::perform(
                                load_folders(account.clone()),
                                Message::FoldersLoaded,
                            );
                        }
                    }
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to load accounts: {e}"));
                }
            },
            Message::SwitchAccount(account_id) => {
                self.account_switcher_open = false;
                if let Some(account) = self.accounts.iter().find(|a| a.id == Some(account_id)) {
                    info!("Switching to account: {}", account.email);
                    self.active_account_id = Some(account_id);
                    self.current_account = Some(account.clone());
                    // Clear current state and reload
                    self.folders.clear();
                    self.folder_paths.clear();
                    self.messages.clear();
                    self.all_messages.clear();
                    self.selected_folder = None;
                    self.selected_message = None;
                    self.message_content = None;
                    self.markdown_items.clear();
                    self.inline_images.clear();
                    self.is_loading_folders = true;
                    return Task::perform(load_folders(account.clone()), Message::FoldersLoaded);
                }
            }
            Message::ToggleAccountSwitcher => {
                self.account_switcher_open = !self.account_switcher_open;
            }
            Message::AddAccount => {
                self.account_switcher_open = false;
                self.account_setup = AccountSetupState::new();
                self.current_view = View::AccountSetup;
            }
            Message::LoadAccount => {
                return Task::perform(load_account(), Message::AccountLoaded);
            }
            Message::AccountLoaded(result) => match result {
                Ok(Some(account)) => {
                    info!("Account loaded: {}", account.email);
                    self.accounts = vec![account.clone()];
                    self.active_account_id = account.id;
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
                    info!(
                        "Settings loaded: theme={:?}, font_size={:?}, density={:?}",
                        settings.theme_mode, settings.font_size, settings.list_density
                    );
                    self.theme_mode = settings.theme_mode;
                    self.font_size = settings.font_size;
                    self.list_density = settings.list_density;
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
                        tracing::error!("Failed to load folders: {}", e);
                        self.error_message = Some(format!("Failed to load folders: {e}"));
                    }
                }
            }
            Message::MessagesLoaded(result) => {
                self.is_loading_messages = false;
                match result {
                    Ok(messages) => {
                        self.is_offline = false; // We're online if fetch succeeded
                        self.selected_message = None;
                        self.message_content = None;

                        // Cache messages for offline use
                        if let Some(account) = self.current_account.as_ref()
                            && let Some(account_id) = account.id
                            && let Some(folder_id) = self.selected_folder
                            && let Some(folder_path) = self.folder_paths.get(&folder_id).cloned()
                        {
                            // Clone for cache before moving to all_messages
                            let messages_for_cache = messages.clone();
                            self.all_messages = messages;
                            self.filter_messages();

                            // Fire and forget cache operation
                            let cache_task = Task::perform(
                                cache_messages(account_id, folder_path, messages_for_cache),
                                |_| Message::WindowResized(0, 0), // Ignore result
                            );
                            // Start IDLE after caching
                            return Task::batch([cache_task, Task::done(Message::StartIdle)]);
                        }
                        self.all_messages = messages;
                        self.filter_messages();
                        // Start IDLE monitoring after messages are loaded
                        return Task::done(Message::StartIdle);
                    }
                    Err(e) => {
                        // Connection failed - try loading from cache
                        if let Some(account) = self.current_account.as_ref()
                            && let Some(account_id) = account.id
                            && let Some(folder_id) = self.selected_folder
                            && let Some(folder_path) = self.folder_paths.get(&folder_id).cloned()
                        {
                            self.is_offline = true;
                            self.is_loading_messages = true;
                            info!("Connection failed, loading from cache: {}", e);
                            return Task::perform(
                                load_cached_messages(account_id, folder_path),
                                Message::CachedMessagesLoaded,
                            );
                        }
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
                            mailledger_core::IdleEvent::NewMail(count) => {
                                // Show desktop notification for new mail
                                show_new_mail_notification(count);

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
            Message::ConnectionStateChanged(is_online) => {
                self.is_offline = !is_online;
                if is_online {
                    info!("Connection restored - now online");
                } else {
                    info!("Connection lost - now offline");
                }
            }
            Message::CachedMessagesLoaded(result) => {
                self.is_loading_messages = false;
                match result {
                    Ok(cached) => {
                        // Get current folder_id for the cached messages
                        let folder_id = self.selected_folder.unwrap_or(FolderId(0));

                        // Convert cached summaries to MessageSummary
                        self.messages = cached
                            .into_iter()
                            .map(|c| MessageSummary {
                                id: MessageId(c.uid),
                                folder_id,
                                from_name: c.from_name,
                                from_email: c.from_email,
                                subject: c.subject,
                                snippet: c.snippet,
                                date: c.date,
                                is_read: c.is_read,
                                is_flagged: c.is_flagged,
                                has_attachments: c.has_attachments,
                                thread_id: None,
                                message_id: None,
                                in_reply_to: None,
                            })
                            .collect();
                        self.all_messages = self.messages.clone();
                        self.threads = group_into_threads(&self.messages);
                        info!("Loaded {} messages from cache", self.messages.len());
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to load cached messages: {e}"));
                    }
                }
            }
            Message::CachedContentLoaded(result) => {
                match result {
                    Ok(Some(cached)) => {
                        // Parse from string to get name and email
                        let (from_name, from_email) = parse_email_address(&cached.from);

                        // Convert cached content to MessageContent
                        self.message_content = Some(MessageContent {
                            id: MessageId(cached.uid),
                            from_name,
                            from_email,
                            to: cached.to.split(", ").map(String::from).collect(),
                            cc: cached.cc.split(", ").map(String::from).collect(),
                            subject: cached.subject,
                            date: cached.date,
                            body_text: cached.body_text,
                            body_html: cached.body_html,
                            attachments: Vec::new(), // TODO: deserialize from JSON
                        });
                        // Parse markdown for text body
                        if let Some(ref content) = self.message_content {
                            let text = content.body_text.as_deref().unwrap_or("");
                            self.markdown_items = markdown::parse(text).collect();
                        }
                    }
                    Ok(None) => {
                        self.error_message =
                            Some("Message content not available offline".to_string());
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to load cached content: {e}"));
                    }
                }
            }
            Message::KeyPressed(action) => {
                return self.handle_keyboard_action(action);
            }
            Message::SnoozeSelected(duration) => {
                if let Some(msg) = self.message_content.as_ref()
                    && let Some(account) = self.current_account.as_ref()
                    && let Some(account_id) = account.id
                {
                    let folder_path = self
                        .selected_folder
                        .and_then(|id| self.folder_paths.get(&id))
                        .cloned()
                        .unwrap_or_else(|| "INBOX".to_string());

                    let from_display = if msg.from_name.is_empty() {
                        msg.from_email.clone()
                    } else {
                        format!("{} <{}>", msg.from_name, msg.from_email)
                    };

                    return Task::perform(
                        snooze_message(
                            account_id,
                            msg.id.0,
                            folder_path,
                            duration.expiry_time(),
                            msg.subject.clone(),
                            from_display,
                        ),
                        Message::MessageSnoozed,
                    );
                }
                self.snooze_dropdown_open = false;
            }
            Message::MessageSnoozed(result) => {
                self.snooze_dropdown_open = false;
                match result {
                    Ok(()) => {
                        info!("Message snoozed successfully");
                        // Hide the snoozed message from the current list
                        if let Some(selected) = self.selected_message {
                            self.messages.retain(|m| m.id != selected);
                            self.all_messages.retain(|m| m.id != selected);
                            self.selected_message = None;
                            self.message_content = None;
                        }
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to snooze: {e}"));
                    }
                }
            }
            Message::CheckExpiredSnoozes => {
                return Task::perform(check_expired_snoozes(), Message::ExpiredSnoozesLoaded);
            }
            Message::ExpiredSnoozesLoaded(result) => {
                match result {
                    Ok(expired) => {
                        if !expired.is_empty() {
                            info!("{} snoozed messages have expired", expired.len());
                            // Refresh the message list to show un-snoozed messages
                            return Task::done(Message::RefreshMessages);
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to check expired snoozes: {}", e);
                    }
                }
            }
            Message::ToggleSnoozeDropdown => {
                self.snooze_dropdown_open = !self.snooze_dropdown_open;
            }
        }
        Task::none()
    }

    /// Handle keyboard shortcut actions.
    #[allow(clippy::too_many_lines)]
    fn handle_keyboard_action(&mut self, action: KeyboardAction) -> Task<Message> {
        match action {
            KeyboardAction::ComposeNew => {
                self.compose_state = ComposeState::new();
                self.compose_body = text_editor::Content::new();
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
            KeyboardAction::Archive => {
                if self.current_view == View::Inbox
                    && let Some(message_id) = self.selected_message
                {
                    return Task::done(Message::ArchiveMessage(message_id));
                }
            }
            KeyboardAction::ToggleStar => {
                if self.current_view == View::Inbox
                    && let Some(message_id) = self.selected_message
                {
                    return Task::done(Message::ToggleFlag(message_id));
                }
            }
            KeyboardAction::MarkUnread => {
                if self.current_view == View::Inbox
                    && let Some(message_id) = self.selected_message
                {
                    return Task::done(Message::ToggleRead(message_id));
                }
            }
            KeyboardAction::FocusSearch | KeyboardAction::ShowHelp => {
                // FocusSearch: will be handled when we add the search text input with an ID
                // ShowHelp: will be implemented with keyboard help view
                // For now, both are no-op placeholders
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
                View::Screener => {
                    // Go back to inbox
                    self.current_view = View::Inbox;
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
            KeyboardAction::SelectNextMessage => {
                if self.current_view == View::Inbox && !self.messages.is_empty() {
                    let next_id = match self.selected_message {
                        None => self.messages.first().map(|m| m.id),
                        Some(current_id) => {
                            let current_idx = self.messages.iter().position(|m| m.id == current_id);
                            match current_idx {
                                Some(idx) if idx + 1 < self.messages.len() => {
                                    Some(self.messages[idx + 1].id)
                                }
                                _ => None, // Already at last message
                            }
                        }
                    };
                    if let Some(id) = next_id {
                        return Task::done(Message::SelectMessage(id));
                    }
                }
            }
            KeyboardAction::SelectPreviousMessage => {
                if self.current_view == View::Inbox && !self.messages.is_empty() {
                    let prev_id = match self.selected_message {
                        None => self.messages.last().map(|m| m.id),
                        Some(current_id) => {
                            let current_idx = self.messages.iter().position(|m| m.id == current_id);
                            match current_idx {
                                Some(idx) if idx > 0 => Some(self.messages[idx - 1].id),
                                _ => None, // Already at first message
                            }
                        }
                    };
                    if let Some(id) = prev_id {
                        return Task::done(Message::SelectMessage(id));
                    }
                }
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
        use message::FormattingStyle;

        match msg {
            ComposeMessage::ToChanged(to) => {
                let task = self.trigger_autocomplete(&to, AutocompleteField::To);
                self.compose_state.to = to;
                return task;
            }
            ComposeMessage::CcChanged(cc) => {
                let task = self.trigger_autocomplete(&cc, AutocompleteField::Cc);
                self.compose_state.cc = cc;
                return task;
            }
            ComposeMessage::BccChanged(bcc) => {
                let task = self.trigger_autocomplete(&bcc, AutocompleteField::Bcc);
                self.compose_state.bcc = bcc;
                return task;
            }
            ComposeMessage::SubjectChanged(subject) => {
                self.compose_state.subject = subject;
            }
            ComposeMessage::BodyChanged(_) => {
                // Legacy handler - body now uses text_editor
            }
            ComposeMessage::BodyAction(action) => {
                self.compose_body.perform(action);
            }
            ComposeMessage::InsertFormatting(style) => {
                // Insert markdown formatting based on style
                let insert_text = match style {
                    FormattingStyle::Bold => "****",
                    FormattingStyle::Italic => "**",
                    FormattingStyle::Link => "[text](url)",
                };
                // Insert at cursor using Edit action
                self.compose_body
                    .perform(text_editor::Action::Edit(text_editor::Edit::Paste(
                        std::sync::Arc::new(insert_text.to_string()),
                    )));
            }
            ComposeMessage::Send => {
                if let Some(error) = self.compose_state.validate() {
                    self.compose_state.send_error = Some(error);
                } else if let Some(account) = self.current_account.clone() {
                    self.compose_state.is_sending = true;
                    self.compose_state.send_error = None;
                    self.compose_state.send_success = false;
                    // Get body from text_editor Content
                    let body_text = self.compose_body.text();
                    let message = self
                        .compose_state
                        .to_outgoing_with_body(&account.email, &body_text);
                    return Task::perform(send_email(account, message), Message::EmailSent);
                } else {
                    self.compose_state.send_error =
                        Some("No account configured. Please set up an account first.".to_string());
                }
            }
            ComposeMessage::Cancel => {
                self.compose_state = ComposeState::new();
                self.compose_body = text_editor::Content::new();
                self.current_view = View::Inbox;
            }
            ComposeMessage::SuggestionsLoaded(suggestions) => {
                // Store suggestions in compose state
                if let Some(field) = self.compose_state.active_autocomplete {
                    self.compose_state.show_suggestions(suggestions, field);
                }
            }
            ComposeMessage::SelectSuggestion(index) => {
                self.compose_state.apply_suggestion(index);
            }
            ComposeMessage::DismissSuggestions => {
                self.compose_state.clear_suggestions();
            }
        }
        Task::none()
    }

    /// Handle screener messages.
    fn handle_screener(&mut self, msg: ScreenerMessage) -> Task<Message> {
        let Some(account_id) = self.current_account.as_ref().and_then(|a| a.id) else {
            return Task::none();
        };

        match msg {
            ScreenerMessage::ApproveToImbox(email) => {
                // Remove from local list immediately for snappy UI
                self.pending_senders.retain(|s| s.email != email);
                Task::perform(
                    approve_sender(account_id, email, mailledger_core::InboxCategory::Imbox),
                    Message::SenderDecisionSaved,
                )
            }
            ScreenerMessage::ApproveToFeed(email) => {
                self.pending_senders.retain(|s| s.email != email);
                Task::perform(
                    approve_sender(account_id, email, mailledger_core::InboxCategory::Feed),
                    Message::SenderDecisionSaved,
                )
            }
            ScreenerMessage::ApproveToPaperTrail(email) => {
                self.pending_senders.retain(|s| s.email != email);
                Task::perform(
                    approve_sender(
                        account_id,
                        email,
                        mailledger_core::InboxCategory::PaperTrail,
                    ),
                    Message::SenderDecisionSaved,
                )
            }
            ScreenerMessage::Block(email) => {
                self.pending_senders.retain(|s| s.email != email);
                Task::perform(
                    block_sender(account_id, email),
                    Message::SenderDecisionSaved,
                )
            }
            ScreenerMessage::Reset(email) => Task::perform(
                reset_sender(account_id, email),
                Message::SenderDecisionSaved,
            ),
        }
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
                return Task::perform(
                    save_settings(self.current_settings()),
                    Message::SettingsSaved,
                );
            }
            SettingsMessage::SetFontSize(size) => {
                self.font_size = size;
                info!("Font size changed to {:?}", self.font_size);
                return Task::perform(
                    save_settings(self.current_settings()),
                    Message::SettingsSaved,
                );
            }
            SettingsMessage::SetDensity(density) => {
                self.list_density = density;
                info!("List density changed to {:?}", self.list_density);
                return Task::perform(
                    save_settings(self.current_settings()),
                    Message::SettingsSaved,
                );
            }
        }
        Task::none()
    }

    /// Creates an `AppSettings` from current state.
    const fn current_settings(&self) -> AppSettings {
        AppSettings {
            theme_mode: self.theme_mode,
            font_size: self.font_size,
            list_density: self.list_density,
        }
    }

    /// Builds IMAP search criteria from current search state.
    fn build_search_criteria(&self) -> mailledger_core::SearchCriteria {
        let mut criteria = mailledger_core::SearchCriteria::default();

        // Add text search if present
        if !self.search_query.is_empty() {
            criteria.text = Some(self.search_query.clone());
        }

        // Add filter criteria
        for filter in &self.search_filters {
            match filter {
                SearchFilter::Unread => criteria.unread = true,
                SearchFilter::Flagged => criteria.flagged = true,
                SearchFilter::HasAttachments => {
                    // Note: IMAP doesn't have a direct "has attachments" search
                    // We'd need to use Content-Type: multipart/mixed or similar
                    // For now, skip this filter on server-side
                }
            }
        }

        criteria
    }

    /// Triggers contact autocomplete for the given field.
    fn trigger_autocomplete(&mut self, value: &str, field: AutocompleteField) -> Task<Message> {
        // Extract the last entry being typed (after the last comma)
        let query = value.rsplit(',').next().map_or("", str::trim).to_string();

        // Clear suggestions if query is too short
        if query.len() < 2 {
            self.compose_state.clear_suggestions();
            return Task::none();
        }

        // Set active field for when results come back
        self.compose_state.active_autocomplete = Some(field);

        // Search for matching contacts
        Task::perform(search_contacts(query), |result| {
            Message::Compose(ComposeMessage::SuggestionsLoaded(
                result.unwrap_or_default(),
            ))
        })
    }

    /// Render current state as UI.
    fn view(&self) -> Element<'_, Message> {
        match self.current_view {
            View::Inbox => self.view_inbox(),
            View::Screener => self.view_screener(),
            View::Compose => self.view_compose(),
            View::Settings => self.view_settings(),
            View::AccountSetup => self.view_account_setup(),
        }
    }

    /// Screener view.
    fn view_screener(&self) -> Element<'_, Message> {
        view::view_screener(&self.pending_senders)
    }

    /// Main inbox view with three-pane layout.
    fn view_inbox(&self) -> Element<'_, Message> {
        let header = view::view_header(&self.search_query, &self.search_filters, self.is_offline);
        let error_banner: Element<'_, Message> = self.error_message.as_ref().map_or_else(
            || Space::new().height(0).into(),
            |error| {
                let p = palette::current();
                container(text(error).size(14).color(p.text_on_primary))
                    .padding([6, 12])
                    .width(Length::Fill)
                    .style(move |_theme| container::Style {
                        background: Some(Background::Color(p.accent_red)),
                        border: Border {
                            color: p.accent_red,
                            width: 1.0,
                            radius: radius::SMALL.into(),
                        },
                        ..Default::default()
                    })
                    .into()
            },
        );

        let mut main_content = row![];

        // Sidebar (folder list)
        if self.sidebar_visible {
            main_content = main_content.push(view::view_sidebar(
                &self.folders,
                self.selected_folder,
                self.pending_senders.len(),
                &self.accounts,
                self.active_account_id,
                self.account_switcher_open,
                self.sidebar_width,
            ));

            // Divider between sidebar and message list
            main_content = main_content.push(view::view_pane_divider(
                message::PaneDivider::SidebarMessageList,
            ));
        }

        // Message list
        main_content = main_content.push(view::view_message_list(
            &self.messages,
            self.selected_message,
            self.is_loading_messages,
            self.view_mode,
            &self.threads,
            &self.expanded_threads,
            self.font_size,
            self.list_density,
            self.message_list_scroll_offset,
            self.message_list_viewport_height,
            self.message_list_width,
        ));

        // Divider between message list and message view
        main_content = main_content.push(view::view_pane_divider(
            message::PaneDivider::MessageListMessageView,
        ));

        // Message content - get read status from selected message
        let is_read = self
            .selected_message
            .and_then(|id| self.messages.iter().find(|m| m.id == id))
            .is_some_and(|m| m.is_read);

        main_content = main_content.push(view::view_message_content(
            self.message_content.as_ref(),
            &self.markdown_items,
            &self.inline_images,
            is_read,
            self.quoted_expanded,
            self.font_size,
            self.snooze_dropdown_open,
        ));

        column![header, error_banner, main_content.height(Length::Fill)]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// Compose view.
    fn view_compose(&self) -> Element<'_, Message> {
        view::view_compose(&self.compose_state, &self.compose_body)
    }

    /// Settings view.
    fn view_settings(&self) -> Element<'_, Message> {
        view::view_settings(
            &self.settings_state,
            self.current_account.as_ref(),
            self.theme_mode,
            self.font_size,
            self.list_density,
        )
    }

    /// Account setup view.
    fn view_account_setup(&self) -> Element<'_, Message> {
        view::view_account_setup(&self.account_setup)
    }

    /// Subscribe to keyboard and mouse events.
    fn subscription(&self) -> Subscription<Message> {
        let is_dragging = self.dragging_divider.is_some();

        // Listen to all events when dragging for smooth tracking
        if is_dragging {
            event::listen().map(|event| match event {
                Event::Mouse(mouse::Event::CursorMoved { position }) => {
                    Message::PaneDragMoved(position.x)
                }
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                    Message::StopPaneDrag
                }
                Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                    handle_key_press(key, modifiers).unwrap_or(Message::WindowResized(0, 0))
                }
                _ => Message::WindowResized(0, 0),
            })
        } else {
            keyboard::listen().map(|event| {
                if let keyboard::Event::KeyPressed { key, modifiers, .. } = event {
                    handle_key_press(key, modifiers).unwrap_or(Message::WindowResized(0, 0))
                } else {
                    Message::WindowResized(0, 0)
                }
            })
        }
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
        // #: Delete (Gmail style)
        Key::Character(c) if !ctrl && c.as_str() == "#" => {
            Some(Message::KeyPressed(KeyboardAction::Delete))
        }
        // e: Archive (Gmail style)
        Key::Character(c) if !ctrl && !shift && c.as_str() == "e" => {
            Some(Message::KeyPressed(KeyboardAction::Archive))
        }
        // s: Star/flag toggle (Gmail style)
        Key::Character(c) if !ctrl && !shift && c.as_str() == "s" => {
            Some(Message::KeyPressed(KeyboardAction::ToggleStar))
        }
        // u: Mark unread (Gmail style)
        Key::Character(c) if !ctrl && !shift && c.as_str() == "u" => {
            Some(Message::KeyPressed(KeyboardAction::MarkUnread))
        }
        // /: Focus search
        Key::Character(c) if !ctrl && !shift && c.as_str() == "/" => {
            Some(Message::KeyPressed(KeyboardAction::FocusSearch))
        }
        // ?: Show keyboard shortcuts help
        Key::Character(c) if !ctrl && c.as_str() == "?" => {
            Some(Message::KeyPressed(KeyboardAction::ShowHelp))
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
        // Down arrow or J: Select next message
        Key::Named(keyboard::key::Named::ArrowDown) if !ctrl && !shift => {
            Some(Message::KeyPressed(KeyboardAction::SelectNextMessage))
        }
        Key::Character(c) if !ctrl && !shift && c.as_str() == "j" => {
            Some(Message::KeyPressed(KeyboardAction::SelectNextMessage))
        }
        // Up arrow or K: Select previous message
        Key::Named(keyboard::key::Named::ArrowUp) if !ctrl && !shift => {
            Some(Message::KeyPressed(KeyboardAction::SelectPreviousMessage))
        }
        Key::Character(c) if !ctrl && !shift && c.as_str() == "k" => {
            Some(Message::KeyPressed(KeyboardAction::SelectPreviousMessage))
        }
        _ => None,
    }
}

fn prepare_inline_images(urls: Vec<String>) -> (Vec<InlineImage>, Task<Message>) {
    let mut seen = HashSet::new();
    let mut inline_images = Vec::new();
    let mut tasks = Vec::new();

    for url in urls {
        if !seen.insert(url.clone()) {
            continue;
        }

        if url.starts_with("http://") || url.starts_with("https://") {
            inline_images.push(InlineImage {
                url: url.clone(),
                state: InlineImageState::Loading,
            });

            tasks.push(Task::perform(
                download_inline_image(url.clone()),
                move |result| Message::InlineImageLoaded { url, result },
            ));
        }

        if inline_images.len() >= 10 {
            break;
        }
    }

    let task = if tasks.is_empty() {
        Task::none()
    } else {
        Task::batch(tasks)
    };

    (inline_images, task)
}

/// Convert message content to markdown text.
///
/// Priority: HTML (converted via htmd) > plain text > empty message.
#[allow(clippy::option_if_let_else)] // Chained if-else is clearer here
fn content_to_markdown(content: &MessageContent) -> String {
    if let Some(html) = content.body_html.as_ref() {
        html_to_markdown(html)
    } else if let Some(plain) = content.body_text.as_ref() {
        plain.clone()
    } else {
        String::from("*(No content)*")
    }
}

/// Convert HTML to Markdown using htmd library.
#[allow(clippy::option_if_let_else)] // This is a Result, not Option
fn html_to_markdown(html: &str) -> String {
    let converter = htmd::HtmlToMarkdown::new();
    match converter.convert(html) {
        Ok(md) => clean_markdown(&md),
        Err(_) => simple_html_strip(html),
    }
}

/// Clean up converted markdown (remove excessive blank lines).
fn clean_markdown(md: &str) -> String {
    let mut result = String::with_capacity(md.len());
    let mut consecutive_newlines = 0;

    for line in md.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            consecutive_newlines += 1;
            if consecutive_newlines <= 2 {
                result.push('\n');
            }
        } else {
            consecutive_newlines = 0;
            result.push_str(trimmed);
            result.push('\n');
        }
    }

    result.trim().to_string()
}

/// Simple HTML to text fallback.
fn simple_html_strip(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;

    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }

    result
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
}

fn extract_image_urls(html: &str) -> Vec<String> {
    let lower = html.to_lowercase();
    let mut urls = Vec::new();
    let mut idx = 0;

    while let Some(pos) = lower[idx..].find("<img") {
        let start = idx + pos;
        let end = lower[start..]
            .find('>')
            .map_or(html.len(), |offset| start + offset);

        let tag = &html[start..end];
        if let Some(src) = extract_html_attr(tag, "src")
            && !src.is_empty()
        {
            urls.push(src);
        }

        idx = end.saturating_add(1);
    }

    urls
}

fn extract_html_attr(tag: &str, attr: &str) -> Option<String> {
    let lower = tag.to_lowercase();
    let mut idx = 0;

    while let Some(pos) = lower[idx..].find(attr) {
        let mut cursor = idx + pos + attr.len();

        while cursor < tag.len() && tag.as_bytes()[cursor].is_ascii_whitespace() {
            cursor += 1;
        }

        if cursor >= tag.len() || tag.as_bytes()[cursor] != b'=' {
            idx = cursor;
            continue;
        }

        cursor += 1;
        while cursor < tag.len() && tag.as_bytes()[cursor].is_ascii_whitespace() {
            cursor += 1;
        }

        if cursor >= tag.len() {
            return None;
        }

        let quote = tag.as_bytes()[cursor];
        if quote == b'"' || quote == b'\'' {
            cursor += 1;
            let start = cursor;
            while cursor < tag.len() && tag.as_bytes()[cursor] != quote {
                cursor += 1;
            }
            return Some(tag[start..cursor].to_string());
        }

        let start = cursor;
        while cursor < tag.len()
            && !tag.as_bytes()[cursor].is_ascii_whitespace()
            && tag.as_bytes()[cursor] != b'>'
        {
            cursor += 1;
        }
        return Some(tag[start..cursor].to_string());
    }

    None
}

async fn download_inline_image(url: String) -> Result<Vec<u8>, String> {
    let response = reqwest::get(&url).await.map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }
    let bytes = response.bytes().await.map_err(|e| e.to_string())?;
    Ok(bytes.to_vec())
}

async fn open_html_message(
    html_body: String,
    message_id: u32,
    subject: String,
) -> Result<(), String> {
    let dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("mailledger")
        .join("html");
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| e.to_string())?;

    let file_path = dir.join(format!("message_{message_id}.html"));
    let document = format!(
        "<!doctype html><meta charset=\"utf-8\"><title>{}</title>{}",
        html_escape(&subject),
        html_body
    );

    tokio::fs::write(&file_path, document)
        .await
        .map_err(|e| e.to_string())?;

    tokio::task::spawn_blocking(move || opener::open(file_path))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())?;

    Ok(())
}

fn html_escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(c),
        }
    }
    escaped
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

/// Search contacts for autocomplete suggestions.
async fn search_contacts(query: String) -> Result<Vec<mailledger_core::Contact>, String> {
    use mailledger_core::ContactRepository;

    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("mailledger");

    std::fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;

    let db_path = data_dir.join("mailledger.db");
    let repo = ContactRepository::new(db_path.to_str().unwrap_or("mailledger.db"))
        .await
        .map_err(|e| e.to_string())?;

    repo.search(&query, 5).await.map_err(|e| e.to_string())
}

/// Record contacts from sent email recipients.
async fn record_contacts(recipients: Vec<String>) -> Result<(), String> {
    use mailledger_core::ContactRepository;

    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("mailledger");

    std::fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;

    let db_path = data_dir.join("mailledger.db");
    let repo = ContactRepository::new(db_path.to_str().unwrap_or("mailledger.db"))
        .await
        .map_err(|e| e.to_string())?;

    for recipient in recipients {
        // Parse "Name <email>" format or plain email
        let (name, email) = parse_email_address(&recipient);
        if let Err(e) = repo.record(&email, &name).await {
            tracing::warn!("Failed to record contact {}: {}", email, e);
        }
    }

    Ok(())
}

/// Parse an email address that may be in "Name <email>" format.
fn parse_email_address(addr: &str) -> (String, String) {
    let addr = addr.trim();
    if let Some(start) = addr.find('<')
        && let Some(end) = addr.find('>')
    {
        let name = addr[..start].trim().trim_matches('"').to_string();
        let email = addr[start + 1..end].trim().to_string();
        return (name, email);
    }
    // Plain email address
    (String::new(), addr.to_string())
}

/// Snooze a message until the specified time.
async fn snooze_message(
    account_id: mailledger_core::AccountId,
    message_uid: u32,
    folder_path: String,
    snooze_until: chrono::DateTime<chrono::Utc>,
    subject: String,
    from: String,
) -> Result<(), String> {
    use mailledger_core::{SnoozeRepository, SnoozedMessage};

    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("mailledger");

    std::fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;

    let db_path = data_dir.join("mailledger.db");
    let repo = SnoozeRepository::new(db_path.to_str().unwrap_or("mailledger.db"))
        .await
        .map_err(|e| e.to_string())?;

    let snoozed = SnoozedMessage::new(
        account_id,
        message_uid,
        folder_path,
        snooze_until,
        subject,
        from,
    );

    repo.snooze(&snoozed).await.map_err(|e| e.to_string())?;

    tracing::info!("Snoozed message {} until {}", message_uid, snooze_until);
    Ok(())
}

/// Check for and clear expired snoozes.
async fn check_expired_snoozes() -> Result<Vec<mailledger_core::SnoozedMessage>, String> {
    use mailledger_core::SnoozeRepository;

    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("mailledger");

    let db_path = data_dir.join("mailledger.db");

    // Check if database exists
    if !db_path.exists() {
        return Ok(Vec::new());
    }

    let repo = SnoozeRepository::new(db_path.to_str().unwrap_or("mailledger.db"))
        .await
        .map_err(|e| e.to_string())?;

    let expired = repo.get_expired().await.map_err(|e| e.to_string())?;

    // Remove expired snoozes from database
    for msg in &expired {
        if let Err(e) = repo
            .unsnooze(msg.account_id, msg.message_uid, &msg.folder_path)
            .await
        {
            tracing::warn!("Failed to clear snooze for {}: {}", msg.message_uid, e);
        }
    }

    Ok(expired)
}

/// Execute IMAP search on the server.
async fn execute_search(
    account: mailledger_core::Account,
    folder_path: String,
    criteria: mailledger_core::SearchCriteria,
) -> Result<Vec<mailledger_imap::Uid>, String> {
    use mailledger_core::{connect_and_login, search_messages, select_folder};

    // Connect and login
    let client = connect_and_login(&account)
        .await
        .map_err(|e| e.to_string())?;

    // Select the folder (takes ownership of client)
    let (mut selected, _status) = select_folder(client, &folder_path)
        .await
        .map_err(|e| e.to_string())?;

    // Execute search
    let uids = search_messages(&mut selected, &criteria)
        .await
        .map_err(|e| e.to_string())?;

    tracing::debug!("IMAP search returned {} UIDs", uids.len());
    Ok(uids)
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

/// Load all accounts from database.
async fn load_all_accounts() -> Result<Vec<mailledger_core::Account>, String> {
    use mailledger_core::AccountRepository;

    // Get data directory
    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("mailledger");

    let db_path = data_dir.join("mailledger.db");

    // Check if database exists
    if !db_path.exists() {
        return Ok(Vec::new());
    }

    let repo = AccountRepository::new(db_path.to_str().unwrap_or("mailledger.db"))
        .await
        .map_err(|e| e.to_string())?;

    let accounts = repo.list().await.map_err(|e| e.to_string())?;
    tracing::info!("Loaded {} account(s) from database", accounts.len());
    Ok(accounts)
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

    // Get the number of messages and highest possible UID
    let total = status.exists;
    if total == 0 {
        return Ok(Vec::new());
    }

    // Use UIDNEXT - 1 as the max UID (UIDs are not sequential with message count)
    // If UIDNEXT is not available, use 1:* by setting a high upper bound
    let max_uid = status
        .uid_next
        .map_or(u32::MAX, |u| u.get().saturating_sub(1));

    // Fetch the most recent messages (up to 50)
    // We fetch from (max_uid - 49) to max_uid to get the latest messages
    let fetch_count = total.min(50);
    let start_uid = if max_uid > fetch_count {
        max_uid - fetch_count + 1
    } else {
        1
    };

    let uid_set = UidSet::range(
        mailledger_imap::types::Uid::new(start_uid).ok_or("Invalid UID")?,
        mailledger_imap::types::Uid::new(max_uid).ok_or("Invalid UID")?,
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

/// Load pending senders from the triage database.
async fn load_pending_senders(
    account_id: mailledger_core::AccountId,
) -> Result<Vec<mailledger_core::ScreenedSender>, String> {
    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("mailledger");

    let db_path = data_dir.join("triage.db");
    let repo = mailledger_core::TriageRepository::new(db_path.to_str().unwrap_or("triage.db"))
        .await
        .map_err(|e| e.to_string())?;

    repo.get_pending_senders(account_id)
        .await
        .map_err(|e| e.to_string())
}

/// Record a new sender in the triage database.
async fn record_sender(
    account_id: mailledger_core::AccountId,
    email: String,
    display_name: Option<String>,
) -> Result<(), String> {
    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("mailledger");

    std::fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;

    let db_path = data_dir.join("triage.db");
    let repo = mailledger_core::TriageRepository::new(db_path.to_str().unwrap_or("triage.db"))
        .await
        .map_err(|e| e.to_string())?;

    repo.record_sender(account_id, &email, display_name.as_deref())
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Approve a sender with a category.
async fn approve_sender(
    account_id: mailledger_core::AccountId,
    email: String,
    category: mailledger_core::InboxCategory,
) -> Result<(), String> {
    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("mailledger");

    let db_path = data_dir.join("triage.db");
    let repo = mailledger_core::TriageRepository::new(db_path.to_str().unwrap_or("triage.db"))
        .await
        .map_err(|e| e.to_string())?;

    repo.approve_sender(account_id, &email, category)
        .await
        .map_err(|e| e.to_string())
}

/// Block a sender.
async fn block_sender(account_id: mailledger_core::AccountId, email: String) -> Result<(), String> {
    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("mailledger");

    let db_path = data_dir.join("triage.db");
    let repo = mailledger_core::TriageRepository::new(db_path.to_str().unwrap_or("triage.db"))
        .await
        .map_err(|e| e.to_string())?;

    repo.block_sender(account_id, &email)
        .await
        .map_err(|e| e.to_string())
}

/// Reset a sender back to pending.
async fn reset_sender(account_id: mailledger_core::AccountId, email: String) -> Result<(), String> {
    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("mailledger");

    let db_path = data_dir.join("triage.db");
    let repo = mailledger_core::TriageRepository::new(db_path.to_str().unwrap_or("triage.db"))
        .await
        .map_err(|e| e.to_string())?;

    repo.reset_sender(account_id, &email)
        .await
        .map_err(|e| e.to_string())
}

/// Download an attachment from a message.
async fn download_attachment_task(
    account: mailledger_core::Account,
    folder_path: String,
    uid: u32,
    part_number: String,
    filename: String,
    encoding: String,
) -> Result<(String, Vec<u8>), String> {
    use mailledger_core::{connect_and_login, download_attachment, select_folder};
    use mailledger_imap::types::Uid;

    let client = connect_and_login(&account)
        .await
        .map_err(|e| e.to_string())?;

    let (mut selected_client, _status) = select_folder(client, &folder_path)
        .await
        .map_err(|e| e.to_string())?;

    let imap_uid = Uid::new(uid).ok_or("Invalid UID")?;

    let data = download_attachment(&mut selected_client, imap_uid, &part_number, &encoding)
        .await
        .map_err(|e| e.to_string())?;

    tracing::info!("Downloaded attachment: {} ({} bytes)", filename, data.len());
    Ok((filename, data))
}

/// Archive a message (move to Archive folder).
async fn archive_message(
    account: mailledger_core::Account,
    folder_path: String,
    uid: u32,
    archive_folder: String,
) -> Result<(), String> {
    use mailledger_core::{archive_message as core_archive, connect_and_login, select_folder};
    use mailledger_imap::types::Uid;

    let client = connect_and_login(&account)
        .await
        .map_err(|e| e.to_string())?;

    let (mut selected_client, _status) = select_folder(client, &folder_path)
        .await
        .map_err(|e| e.to_string())?;

    let imap_uid = Uid::new(uid).ok_or("Invalid UID")?;

    core_archive(&mut selected_client, imap_uid, &archive_folder)
        .await
        .map_err(|e| e.to_string())?;

    tracing::info!("Archived message UID {} to {}", uid, archive_folder);
    Ok(())
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

/// Show a desktop notification for new mail.
///
/// Uses notify-rust for cross-platform notifications.
/// Notifications are fire-and-forget - we don't wait for user interaction.
fn show_new_mail_notification(count: u32) {
    let body = if count == 1 {
        "You have new mail".to_string()
    } else {
        format!("You have {count} new messages")
    };

    // Spawn notification in background to avoid blocking
    std::thread::spawn(move || {
        if let Err(e) = notify_rust::Notification::new()
            .summary("New Email")
            .body(&body)
            .icon("mail-unread")
            .appname("MailLedger")
            .timeout(notify_rust::Timeout::Milliseconds(5000))
            .show()
        {
            tracing::warn!("Failed to show notification: {}", e);
        }
    });
}

/// Cache messages after successful fetch.
async fn cache_messages(
    account_id: mailledger_core::AccountId,
    folder_path: String,
    messages: Vec<MessageSummary>,
) -> Result<(), String> {
    use chrono::Utc;
    use mailledger_core::{CacheRepository, CachedMessageSummary};

    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("mailledger");
    std::fs::create_dir_all(&data_dir).ok();

    let db_path = data_dir.join("cache.db");
    let repo = CacheRepository::new(db_path.to_str().unwrap_or("cache.db"))
        .await
        .map_err(|e| e.to_string())?;

    let cached: Vec<CachedMessageSummary> = messages
        .iter()
        .map(|m| CachedMessageSummary {
            account_id,
            folder_path: folder_path.clone(),
            uid: m.id.0, // MessageId inner value
            from_name: m.from_name.clone(),
            from_email: m.from_email.clone(),
            subject: m.subject.clone(),
            snippet: m.snippet.clone(),
            date: m.date.clone(),
            is_read: m.is_read,
            is_flagged: m.is_flagged,
            has_attachments: m.has_attachments,
            cached_at: Utc::now(),
        })
        .collect();

    repo.cache_summaries(&cached)
        .await
        .map_err(|e| e.to_string())?;
    tracing::info!("Cached {} messages for offline use", cached.len());
    Ok(())
}

/// Load messages from cache when offline.
async fn load_cached_messages(
    account_id: mailledger_core::AccountId,
    folder_path: String,
) -> Result<Vec<mailledger_core::CachedMessageSummary>, String> {
    use mailledger_core::CacheRepository;

    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("mailledger");

    let db_path = data_dir.join("cache.db");
    let repo = CacheRepository::new(db_path.to_str().unwrap_or("cache.db"))
        .await
        .map_err(|e| e.to_string())?;

    repo.get_summaries(account_id, &folder_path)
        .await
        .map_err(|e| e.to_string())
}

/// Cache message content after fetching.
async fn cache_message_content(
    account_id: mailledger_core::AccountId,
    folder_path: String,
    content: MessageContent,
) -> Result<(), String> {
    use chrono::Utc;
    use mailledger_core::{CacheRepository, CachedMessageContent};

    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("mailledger");
    std::fs::create_dir_all(&data_dir).ok();

    let db_path = data_dir.join("cache.db");
    let repo = CacheRepository::new(db_path.to_str().unwrap_or("cache.db"))
        .await
        .map_err(|e| e.to_string())?;

    // Construct full from address
    let from = if content.from_name.is_empty() {
        content.from_email.clone()
    } else {
        format!("{} <{}>", content.from_name, content.from_email)
    };

    let cached = CachedMessageContent {
        account_id,
        folder_path,
        uid: content.id.0, // MessageId inner value
        from,
        to: content.to.join(", "),
        cc: content.cc.join(", "),
        subject: content.subject.clone(),
        date: content.date.clone(),
        body_text: content.body_text.clone(),
        body_html: content.body_html.clone(),
        attachments_json: None, // TODO: serialize attachments
        cached_at: Utc::now(),
    };

    repo.cache_content(&cached)
        .await
        .map_err(|e| e.to_string())?;
    tracing::info!("Cached message content for offline use");
    Ok(())
}

/// Load message content from cache when offline.
async fn load_cached_content(
    account_id: mailledger_core::AccountId,
    folder_path: String,
    uid: u32,
) -> Result<Option<mailledger_core::CachedMessageContent>, String> {
    use mailledger_core::CacheRepository;

    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("mailledger");

    let db_path = data_dir.join("cache.db");
    let repo = CacheRepository::new(db_path.to_str().unwrap_or("cache.db"))
        .await
        .map_err(|e| e.to_string())?;

    repo.get_content(account_id, &folder_path, uid)
        .await
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename_path_traversal_unix() {
        assert_eq!(sanitize_filename("../../etc/passwd"), "etcpasswd");
        assert_eq!(
            sanitize_filename("../../../root/.ssh/id_rsa"),
            "root.sshid_rsa"
        );
        assert_eq!(sanitize_filename("./../../sensitive.txt"), "sensitive.txt");
    }

    #[test]
    fn test_sanitize_filename_path_traversal_windows() {
        assert_eq!(
            sanitize_filename("..\\..\\windows\\system32"),
            "windowssystem32"
        );
        assert_eq!(
            sanitize_filename("C:\\Windows\\System32\\config\\sam"),
            "C:WindowsSystem32configsam"
        );
        assert_eq!(sanitize_filename("..\\..\\..\\boot.ini"), "boot.ini");
    }

    #[test]
    fn test_sanitize_filename_mixed_separators() {
        assert_eq!(sanitize_filename("../../../etc\\passwd"), "etcpasswd");
        assert_eq!(sanitize_filename("..\\../mixed/path"), "mixedpath");
    }

    #[test]
    fn test_sanitize_filename_hidden_files() {
        assert_eq!(sanitize_filename(".hidden"), "hidden");
        assert_eq!(sanitize_filename("..hidden"), "hidden");
        assert_eq!(sanitize_filename("...hidden"), "hidden");
        assert_eq!(sanitize_filename(".bashrc"), "bashrc");
    }

    #[test]
    fn test_sanitize_filename_null_bytes() {
        assert_eq!(sanitize_filename("file\0name.txt"), "filename.txt");
        assert_eq!(sanitize_filename("\0hidden"), "hidden");
        assert_eq!(sanitize_filename("file\0\0\0.txt"), "file.txt");
    }

    #[test]
    fn test_sanitize_filename_control_characters() {
        assert_eq!(sanitize_filename("file\x01name.txt"), "filename.txt");
        assert_eq!(sanitize_filename("file\x1Fname.txt"), "filename.txt");
        assert_eq!(sanitize_filename("test\r\nfile.txt"), "testfile.txt");
        assert_eq!(sanitize_filename("tab\tfile.txt"), "tabfile.txt");
    }

    #[test]
    fn test_sanitize_filename_length_limit() {
        let long_name = "a".repeat(300);
        let sanitized = sanitize_filename(&long_name);
        assert_eq!(sanitized.len(), 255);
        assert_eq!(sanitized, "a".repeat(255));
    }

    #[test]
    fn test_sanitize_filename_unicode_preservation() {
        assert_eq!(sanitize_filename("文档.txt"), "文档.txt");
        assert_eq!(sanitize_filename("Документ.pdf"), "Документ.pdf");
        assert_eq!(sanitize_filename("🔒secure.doc"), "🔒secure.doc");
        assert_eq!(
            sanitize_filename("日本語ファイル.jpg"),
            "日本語ファイル.jpg"
        );
    }

    #[test]
    fn test_sanitize_filename_normal_files() {
        assert_eq!(sanitize_filename("document.pdf"), "document.pdf");
        assert_eq!(sanitize_filename("report_2024.xlsx"), "report_2024.xlsx");
        assert_eq!(sanitize_filename("photo-001.jpg"), "photo-001.jpg");
        assert_eq!(
            sanitize_filename("My Document (1).txt"),
            "My Document (1).txt"
        );
    }

    #[test]
    fn test_sanitize_filename_whitespace() {
        assert_eq!(sanitize_filename("  file.txt  "), "file.txt");
        assert_eq!(sanitize_filename("\t\nfile.txt\r\n"), "file.txt");
        assert_eq!(sanitize_filename("   "), "");
    }

    #[test]
    fn test_sanitize_filename_empty_result() {
        assert_eq!(sanitize_filename(""), "");
        assert_eq!(sanitize_filename("."), "");
        assert_eq!(sanitize_filename(".."), "");
        assert_eq!(sanitize_filename("..."), "");
        assert_eq!(sanitize_filename("/"), "");
        assert_eq!(sanitize_filename("\\"), "");
    }

    #[test]
    fn test_sanitize_filename_special_characters_allowed() {
        assert_eq!(sanitize_filename("file-name.txt"), "file-name.txt");
        assert_eq!(sanitize_filename("file_name.txt"), "file_name.txt");
        assert_eq!(sanitize_filename("file (1).txt"), "file (1).txt");
        assert_eq!(sanitize_filename("file@host.txt"), "file@host.txt");
        assert_eq!(sanitize_filename("file#123.txt"), "file#123.txt");
    }

    #[test]
    fn test_sanitize_filename_complex_attack() {
        // Real-world attack pattern: mix of techniques
        assert_eq!(
            sanitize_filename("..\\../..\0/etc/passwd\x00.txt"),
            "etcpasswd.txt"
        );
        assert_eq!(sanitize_filename("....//\\\\.ssh/id_rsa"), "sshid_rsa");
    }

    #[test]
    fn test_sanitize_filename_dots_in_middle() {
        // Dots in the middle of filenames should be preserved (for extensions)
        assert_eq!(sanitize_filename("archive.tar.gz"), "archive.tar.gz");
        assert_eq!(sanitize_filename("file.backup.old"), "file.backup.old");
    }
}
