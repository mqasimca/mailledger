//! Folder (mailbox) data model.

/// Unique identifier for a folder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FolderId(pub u32);

/// A mail folder (mailbox).
#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields will be used as features are implemented
#[allow(clippy::struct_field_names)] // folder_type is clearer than just `kind`
pub struct Folder {
    /// Unique identifier.
    pub id: FolderId,
    /// Display name.
    pub name: String,
    /// Full IMAP path for selection.
    pub path: String,
    /// Number of unread messages.
    pub unread_count: u32,
    /// Total number of messages.
    pub total_count: u32,
    /// Whether this is a special folder (Inbox, Sent, etc.).
    pub folder_type: FolderType,
}

/// Type of folder for special handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)] // All variants needed for complete folder type coverage
pub enum FolderType {
    /// Regular folder.
    #[default]
    Normal,
    /// Inbox.
    Inbox,
    /// Sent messages.
    Sent,
    /// Drafts.
    Drafts,
    /// Trash/Deleted.
    Trash,
    /// Archive.
    Archive,
    /// Spam/Junk.
    Spam,
}

impl FolderType {
    /// Converts from core folder type.
    #[must_use]
    pub const fn from_core(core_type: mailledger_core::FolderType) -> Self {
        match core_type {
            mailledger_core::FolderType::Inbox => Self::Inbox,
            mailledger_core::FolderType::Sent => Self::Sent,
            mailledger_core::FolderType::Drafts => Self::Drafts,
            mailledger_core::FolderType::Trash => Self::Trash,
            mailledger_core::FolderType::Archive => Self::Archive,
            mailledger_core::FolderType::Spam => Self::Spam,
            mailledger_core::FolderType::Regular => Self::Normal,
        }
    }
}

impl Folder {
    /// Creates a new folder.
    #[must_use]
    #[allow(dead_code)] // Will be used when creating folders from IMAP data
    pub fn new(id: FolderId, name: impl Into<String>, folder_type: FolderType) -> Self {
        let name = name.into();
        let path = name.clone();
        Self {
            id,
            name,
            path,
            unread_count: 0,
            total_count: 0,
            folder_type,
        }
    }

    /// Creates a folder from core service folder.
    #[must_use]
    pub fn from_core(id: u32, core_folder: &mailledger_core::Folder) -> Self {
        Self {
            id: FolderId(id),
            name: core_folder.name.clone(),
            path: core_folder.path.clone(),
            unread_count: core_folder.unread_count.unwrap_or(0),
            total_count: core_folder.total_count.unwrap_or(0),
            folder_type: FolderType::from_core(core_folder.folder_type),
        }
    }

    /// Creates mock folders for testing.
    #[must_use]
    #[allow(dead_code)] // Used for testing/demo
    pub fn mock_folders() -> Vec<Self> {
        vec![
            Self {
                id: FolderId(1),
                name: "Inbox".into(),
                path: "INBOX".into(),
                unread_count: 3,
                total_count: 42,
                folder_type: FolderType::Inbox,
            },
            Self {
                id: FolderId(2),
                name: "Sent".into(),
                path: "Sent".into(),
                unread_count: 0,
                total_count: 128,
                folder_type: FolderType::Sent,
            },
            Self {
                id: FolderId(3),
                name: "Drafts".into(),
                path: "Drafts".into(),
                unread_count: 0,
                total_count: 2,
                folder_type: FolderType::Drafts,
            },
            Self {
                id: FolderId(4),
                name: "Archive".into(),
                path: "Archive".into(),
                unread_count: 0,
                total_count: 1024,
                folder_type: FolderType::Archive,
            },
            Self {
                id: FolderId(5),
                name: "Trash".into(),
                path: "Trash".into(),
                unread_count: 0,
                total_count: 15,
                folder_type: FolderType::Trash,
            },
        ]
    }
}
