//! Settings model.

use crate::style::widgets::palette::ThemeMode;

/// State for the settings screen.
#[derive(Debug, Clone, Default)]
pub struct SettingsState {
    /// Selected settings section.
    pub selected_section: SettingsSection,
}

/// Settings sections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsSection {
    /// Account settings.
    #[default]
    Account,
    /// Appearance settings.
    Appearance,
    /// About the application.
    About,
}

/// Font size preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FontSize {
    /// Small font (12px base).
    Small,
    /// Medium font (14px base, default).
    #[default]
    Medium,
    /// Large font (16px base).
    Large,
}

/// List density preference for message list rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ListDensity {
    /// Compact density (48px row height).
    Compact,
    /// Comfortable density (64px row height, default).
    #[default]
    Comfortable,
    /// Spacious density (80px row height).
    Spacious,
}

impl FontSize {
    /// Returns the base font size in pixels.
    #[must_use]
    pub const fn base_size(self) -> u32 {
        match self {
            Self::Small => 12,
            Self::Medium => 14,
            Self::Large => 16,
        }
    }

    /// Returns the heading font size in pixels.
    #[must_use]
    pub const fn heading_size(self) -> u32 {
        match self {
            Self::Small => 14,
            Self::Medium => 16,
            Self::Large => 18,
        }
    }

    /// Returns the snippet font size in pixels.
    #[must_use]
    pub const fn snippet_size(self) -> u32 {
        match self {
            Self::Small => 11,
            Self::Medium => 12,
            Self::Large => 14,
        }
    }
}

impl ListDensity {
    /// Returns the vertical padding for message rows in pixels.
    #[must_use]
    pub const fn row_padding(self) -> u16 {
        match self {
            Self::Compact => 8,
            Self::Comfortable => 12,
            Self::Spacious => 16,
        }
    }

    /// Returns the avatar size in pixels.
    #[must_use]
    pub const fn avatar_size(self) -> f32 {
        match self {
            Self::Compact => 28.0,
            Self::Comfortable => 36.0,
            Self::Spacious => 44.0,
        }
    }

    /// Returns the spacing between elements in pixels.
    #[must_use]
    pub const fn spacing(self) -> u32 {
        match self {
            Self::Compact => 8,
            Self::Comfortable => 12,
            Self::Spacious => 16,
        }
    }
}

/// Application settings that persist across sessions.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct AppSettings {
    /// Current theme mode (serialized as string).
    #[serde(with = "theme_mode_serde")]
    pub theme_mode: ThemeMode,
    /// Font size preference.
    #[serde(default, with = "font_size_serde")]
    pub font_size: FontSize,
    /// List density preference.
    #[serde(default, with = "list_density_serde")]
    pub list_density: ListDensity,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme_mode: ThemeMode::Dark, // Default to dark mode for modern look
            font_size: FontSize::Medium,
            list_density: ListDensity::Comfortable,
        }
    }
}

/// Serde helpers for `ThemeMode` (since it doesn't derive `Serialize`/`Deserialize`).
mod theme_mode_serde {
    use super::ThemeMode;
    use serde::{Deserialize, Deserializer, Serializer};

    #[allow(clippy::trivially_copy_pass_by_ref)] // Required by serde with= signature
    pub fn serialize<S>(mode: &ThemeMode, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match mode {
            ThemeMode::Light => "light",
            ThemeMode::Dark => "dark",
        };
        serializer.serialize_str(s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<ThemeMode, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "light" => Ok(ThemeMode::Light),
            _ => Ok(ThemeMode::Dark),
        }
    }
}

/// Serde helpers for `FontSize`.
mod font_size_serde {
    use super::FontSize;
    use serde::{Deserialize, Deserializer, Serializer};

    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn serialize<S>(size: &FontSize, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match size {
            FontSize::Small => "small",
            FontSize::Medium => "medium",
            FontSize::Large => "large",
        };
        serializer.serialize_str(s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<FontSize, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "small" => Ok(FontSize::Small),
            "large" => Ok(FontSize::Large),
            _ => Ok(FontSize::Medium),
        }
    }
}

/// Serde helpers for `ListDensity`.
mod list_density_serde {
    use super::ListDensity;
    use serde::{Deserialize, Deserializer, Serializer};

    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn serialize<S>(density: &ListDensity, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match density {
            ListDensity::Compact => "compact",
            ListDensity::Comfortable => "comfortable",
            ListDensity::Spacious => "spacious",
        };
        serializer.serialize_str(s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<ListDensity, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "compact" => Ok(ListDensity::Compact),
            "spacious" => Ok(ListDensity::Spacious),
            _ => Ok(ListDensity::Comfortable),
        }
    }
}

impl SettingsState {
    /// Creates a new settings state.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}
