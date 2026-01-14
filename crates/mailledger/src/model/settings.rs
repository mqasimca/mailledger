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

/// Application settings that persist across sessions.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct AppSettings {
    /// Current theme mode (serialized as string).
    #[serde(with = "theme_mode_serde")]
    pub theme_mode: ThemeMode,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme_mode: ThemeMode::Dark, // Default to dark mode for modern look
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

impl SettingsState {
    /// Creates a new settings state.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}
