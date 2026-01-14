//! Theme definitions for the application.

use iced::Color;

/// Application theme colors.
#[derive(Debug, Clone)]
#[allow(dead_code)] // Will be used for themeable UI components
pub struct Theme {
    /// Main background color.
    pub background: Color,
    /// Surface color (cards, panels).
    pub surface: Color,
    /// Primary accent color.
    pub primary: Color,
    /// Primary text color.
    pub text: Color,
    /// Secondary/muted text color.
    pub text_secondary: Color,
    /// Border color.
    pub border: Color,
    /// Selected item background.
    pub selected: Color,
    /// Hover background.
    pub hover: Color,
    /// Unread indicator color.
    pub unread: Color,
    /// Flagged/starred color.
    pub flagged: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self::light()
    }
}

impl Theme {
    /// Light theme (default).
    #[must_use]
    #[allow(dead_code)] // Will be used for themeable UI components
    pub const fn light() -> Self {
        Self {
            background: Color::from_rgb(0.96, 0.96, 0.96),
            surface: Color::WHITE,
            primary: Color::from_rgb(0.0, 0.47, 0.84),
            text: Color::from_rgb(0.1, 0.1, 0.1),
            text_secondary: Color::from_rgb(0.5, 0.5, 0.5),
            border: Color::from_rgb(0.88, 0.88, 0.88),
            selected: Color::from_rgb(0.9, 0.95, 1.0),
            hover: Color::from_rgb(0.95, 0.95, 0.95),
            unread: Color::from_rgb(0.0, 0.47, 0.84),
            flagged: Color::from_rgb(0.95, 0.77, 0.06),
        }
    }

    /// Dark theme.
    #[must_use]
    #[allow(dead_code)] // Will be used for themeable UI components
    pub const fn dark() -> Self {
        Self {
            background: Color::from_rgb(0.1, 0.1, 0.1),
            surface: Color::from_rgb(0.15, 0.15, 0.15),
            primary: Color::from_rgb(0.4, 0.7, 1.0),
            text: Color::from_rgb(0.9, 0.9, 0.9),
            text_secondary: Color::from_rgb(0.6, 0.6, 0.6),
            border: Color::from_rgb(0.25, 0.25, 0.25),
            selected: Color::from_rgb(0.2, 0.25, 0.3),
            hover: Color::from_rgb(0.2, 0.2, 0.2),
            unread: Color::from_rgb(0.4, 0.7, 1.0),
            flagged: Color::from_rgb(1.0, 0.85, 0.3),
        }
    }
}
