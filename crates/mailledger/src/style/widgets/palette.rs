//! Color palette with light and dark theme support.
//!
//! Provides a modern, polished color system inspired by best-in-class email clients
//! like Spark, Superhuman, and modern Material Design.

use iced::Color;

/// Application theme mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeMode {
    /// Light theme (default).
    #[default]
    Light,
    /// Dark theme.
    Dark,
}

/// Complete color palette for the application.
#[derive(Debug, Clone, Copy)]
pub struct Palette {
    // Primary brand colors
    pub primary: Color,
    pub primary_light: Color,
    pub primary_dark: Color,

    // Surface colors
    pub surface: Color,
    pub surface_elevated: Color,
    pub surface_sunken: Color,
    pub background: Color,
    pub background_secondary: Color,

    // Text colors
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,
    pub text_on_primary: Color,

    // Accent colors
    pub accent_blue: Color,
    pub accent_green: Color,
    pub accent_yellow: Color,
    pub accent_red: Color,
    pub accent_purple: Color,

    // State colors
    pub selected: Color,
    pub selected_border: Color,
    pub hover: Color,
    pub unread: Color,

    // Border colors
    pub border_subtle: Color,
    pub border_medium: Color,
    pub border_strong: Color,

    // Shadow color
    pub shadow: Color,
    pub shadow_medium: Color,
}

impl Palette {
    /// Creates the light theme palette.
    ///
    /// Clean, modern appearance inspired by iOS, macOS, and Arc browser.
    /// Soft colors with excellent readability and subtle depth.
    #[must_use]
    pub const fn light() -> Self {
        Self {
            // Primary - Modern blue with warmth
            primary: Color::from_rgb(0.0, 0.48, 0.95), // Bright, friendly blue
            primary_light: Color::from_rgb(0.35, 0.65, 1.0),
            primary_dark: Color::from_rgb(0.0, 0.38, 0.80),

            // Surfaces - Soft, airy whites
            surface: Color::WHITE,
            surface_elevated: Color::from_rgb(1.0, 1.0, 1.0), // Pure white elevated
            surface_sunken: Color::from_rgb(0.97, 0.975, 0.99), // Soft recessed
            background: Color::from_rgb(0.98, 0.985, 0.99),   // Almost white with warmth
            background_secondary: Color::from_rgb(0.96, 0.965, 0.98),

            // Text - Clear hierarchy
            text_primary: Color::from_rgb(0.08, 0.10, 0.14), // Almost black, warm
            text_secondary: Color::from_rgb(0.42, 0.46, 0.54), // Muted, readable
            text_muted: Color::from_rgb(0.60, 0.64, 0.70),   // Subtle
            text_on_primary: Color::WHITE,

            // Accents - Vibrant, energetic
            accent_blue: Color::from_rgb(0.0, 0.55, 1.0), // Bright blue
            accent_green: Color::from_rgb(0.2, 0.75, 0.45), // Fresh green
            accent_yellow: Color::from_rgb(1.0, 0.75, 0.0), // Warm yellow
            accent_red: Color::from_rgb(0.98, 0.28, 0.35), // Vibrant red
            accent_purple: Color::from_rgb(0.65, 0.35, 0.98), // Electric purple

            // States - Subtle depth
            selected: Color::from_rgb(0.94, 0.97, 1.0), // Light blue tint
            selected_border: Color::from_rgb(0.0, 0.55, 1.0), // Blue highlight
            hover: Color::from_rgb(0.97, 0.98, 0.99),   // Very subtle
            unread: Color::from_rgb(0.0, 0.55, 1.0),    // Blue indicator

            // Borders - Soft, natural
            border_subtle: Color::from_rgb(0.92, 0.93, 0.95), // Very light
            border_medium: Color::from_rgb(0.86, 0.88, 0.91), // Medium
            border_strong: Color::from_rgb(0.78, 0.81, 0.85), // Stronger

            // Shadows - Soft, natural depth
            shadow: Color::from_rgba(0.0, 0.0, 0.0, 0.04), // Very soft
            shadow_medium: Color::from_rgba(0.0, 0.0, 0.0, 0.08), // Medium soft
        }
    }

    /// Creates the dark theme palette.
    ///
    /// Futuristic cyber design with neon accents and glassmorphism.
    /// Inspired by modern UI/UX trends: Arc browser, Linear, Raycast.
    #[must_use]
    pub const fn dark() -> Self {
        Self {
            // Primary - Bright teal like terminal/CLI interfaces
            primary: Color::from_rgb(0.0, 1.0, 0.8), // #00FFCC - bright teal
            primary_light: Color::from_rgb(0.2, 1.0, 0.85),
            primary_dark: Color::from_rgb(0.0, 0.8, 0.65),

            // Surfaces - Lighter dark theme, more visible
            surface: Color::from_rgb(0.12, 0.13, 0.15), // Lighter card
            surface_elevated: Color::from_rgb(0.15, 0.16, 0.18), // Elevated
            surface_sunken: Color::from_rgb(0.10, 0.11, 0.13), // Recessed
            background: Color::from_rgb(0.08, 0.09, 0.11), // Lighter background
            background_secondary: Color::from_rgb(0.10, 0.11, 0.13),

            // Text - High contrast for readability
            text_primary: Color::from_rgb(0.92, 0.93, 0.95), // Near white
            text_secondary: Color::from_rgb(0.65, 0.68, 0.72), // Lighter muted gray
            text_muted: Color::from_rgb(0.50, 0.53, 0.58),   // More visible
            text_on_primary: Color::from_rgb(0.08, 0.09, 0.11), // Dark on teal

            // Accents - Terminal-inspired colors
            accent_blue: Color::from_rgb(0.3, 0.7, 1.0), // Bright blue
            accent_green: Color::from_rgb(0.2, 0.9, 0.5), // Terminal green
            accent_yellow: Color::from_rgb(1.0, 0.85, 0.2), // Terminal yellow
            accent_red: Color::from_rgb(1.0, 0.35, 0.4), // Bright red
            accent_purple: Color::from_rgb(0.7, 0.4, 1.0), // Bright purple

            // States - Visible highlighting, terminal style
            selected: Color::from_rgb(0.10, 0.18, 0.20), // Teal-tinted selection
            selected_border: Color::from_rgb(0.0, 1.0, 0.8), // Bright teal border
            hover: Color::from_rgb(0.14, 0.15, 0.17),    // Visible hover
            unread: Color::from_rgb(0.0, 1.0, 0.8),      // Bright teal indicator

            // Borders - More visible
            border_subtle: Color::from_rgb(0.20, 0.21, 0.24), // Subtle but visible
            border_medium: Color::from_rgb(0.28, 0.29, 0.32), // Medium
            border_strong: Color::from_rgb(0.40, 0.42, 0.45), // Stronger

            // Shadows - Minimal (terminals don't have heavy shadows)
            shadow: Color::from_rgba(0.0, 0.0, 0.0, 0.20), // Dark shadow
            shadow_medium: Color::from_rgba(0.0, 0.0, 0.0, 0.30), // Medium shadow
        }
    }

    /// Gets the palette for a given theme mode.
    #[must_use]
    pub const fn for_mode(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Light => Self::light(),
            ThemeMode::Dark => Self::dark(),
        }
    }
}

// Default palette (light mode) for backwards compatibility
// These are computed at compile time and used when theme is not passed

/// Current active palette - defaults to light mode.
/// Note: For theme-aware code, use `Palette::for_mode()` instead.
pub static CURRENT: std::sync::LazyLock<std::sync::RwLock<Palette>> =
    std::sync::LazyLock::new(|| std::sync::RwLock::new(Palette::light()));

/// Sets the current global palette.
pub fn set_theme(mode: ThemeMode) {
    if let Ok(mut palette) = CURRENT.write() {
        *palette = Palette::for_mode(mode);
    }
}

/// Gets a copy of the current palette.
#[must_use]
pub fn current() -> Palette {
    CURRENT.read().map_or_else(|_| Palette::light(), |p| *p)
}

// Legacy constants for backwards compatibility
// These read from the current theme

pub const PRIMARY: Color = Color::from_rgb(0.18, 0.45, 0.92);
pub const PRIMARY_LIGHT: Color = Color::from_rgb(0.35, 0.58, 0.98);
pub const PRIMARY_DARK: Color = Color::from_rgb(0.12, 0.35, 0.78);
pub const SURFACE: Color = Color::WHITE;
pub const SURFACE_ELEVATED: Color = Color::from_rgb(0.995, 0.995, 1.0);
pub const BACKGROUND: Color = Color::from_rgb(0.945, 0.95, 0.96);
pub const BACKGROUND_DARK: Color = Color::from_rgb(0.92, 0.925, 0.94);
pub const TEXT_PRIMARY: Color = Color::from_rgb(0.10, 0.12, 0.16);
pub const TEXT_SECONDARY: Color = Color::from_rgb(0.40, 0.44, 0.52);
pub const TEXT_MUTED: Color = Color::from_rgb(0.58, 0.62, 0.68);
pub const TEXT_ON_PRIMARY: Color = Color::WHITE;
pub const ACCENT_BLUE: Color = Color::from_rgb(0.0, 0.48, 0.95);
pub const ACCENT_GREEN: Color = Color::from_rgb(0.15, 0.68, 0.38);
pub const ACCENT_YELLOW: Color = Color::from_rgb(0.92, 0.70, 0.0);
pub const ACCENT_RED: Color = Color::from_rgb(0.88, 0.22, 0.28);
pub const SELECTED: Color = Color::from_rgb(0.92, 0.95, 1.0);
pub const SELECTED_BORDER: Color = Color::from_rgb(0.68, 0.80, 0.98);
pub const HOVER: Color = Color::from_rgb(0.96, 0.97, 0.99);
pub const UNREAD: Color = Color::from_rgb(0.18, 0.52, 0.98);
pub const BORDER_LIGHT: Color = Color::from_rgb(0.90, 0.91, 0.93);
pub const BORDER_MEDIUM: Color = Color::from_rgb(0.84, 0.86, 0.89);
pub const SHADOW: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.06);
pub const SHADOW_MEDIUM: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.10);
