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

    // Avatar gradient colors (Air-style)
    pub avatar_purple: Color,
    pub avatar_pink: Color,
    pub avatar_cyan: Color,
    pub avatar_green: Color,
    pub avatar_orange: Color,
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

            // Avatar colors (Air-style gradients - use primary color)
            avatar_purple: Color::from_rgb(0.388, 0.400, 0.945), // #6366f1
            avatar_pink: Color::from_rgb(0.925, 0.286, 0.600),   // #ec4899
            avatar_cyan: Color::from_rgb(0.024, 0.714, 0.831),   // #06b6d4
            avatar_green: Color::from_rgb(0.063, 0.725, 0.506),  // #10b981
            avatar_orange: Color::from_rgb(0.961, 0.620, 0.043), // #f59e0b
        }
    }

    /// Creates the dark theme palette.
    ///
    /// "Obsidian Velocity" - Premium dark theme inspired by Nylas Air.
    /// Deep blacks with electric indigo accent for a refined, modern feel.
    #[must_use]
    pub const fn dark() -> Self {
        Self {
            // Primary - Electric indigo (Air's signature color)
            primary: Color::from_rgb(0.388, 0.400, 0.945), // #6366f1 - indigo
            primary_light: Color::from_rgb(0.506, 0.549, 0.972), // #818cf8 - lighter
            primary_dark: Color::from_rgb(0.310, 0.275, 0.898), // #4f46e5 - darker

            // Surfaces - Deep obsidian blacks
            surface: Color::from_rgb(0.067, 0.067, 0.078), // #111114 - cards
            surface_elevated: Color::from_rgb(0.094, 0.094, 0.110), // #18181c - elevated
            surface_sunken: Color::from_rgb(0.039, 0.039, 0.047), // #0a0a0c - deepest
            background: Color::from_rgb(0.039, 0.039, 0.047), // #0a0a0c - main bg
            background_secondary: Color::from_rgb(0.067, 0.067, 0.078), // #111114

            // Text - Clear hierarchy with warm whites
            text_primary: Color::from_rgb(0.957, 0.957, 0.961), // #f4f4f5
            text_secondary: Color::from_rgb(0.631, 0.631, 0.667), // #a1a1aa
            text_muted: Color::from_rgb(0.443, 0.443, 0.478),   // #71717a
            text_on_primary: Color::WHITE,                      // White on indigo

            // Accents - Air's color-coded action system
            accent_blue: Color::from_rgb(0.231, 0.510, 0.965), // #3b82f6 - info
            accent_green: Color::from_rgb(0.133, 0.773, 0.369), // #22c55e - archive/success
            accent_yellow: Color::from_rgb(0.961, 0.620, 0.043), // #f59e0b - snooze/warning
            accent_red: Color::from_rgb(0.937, 0.267, 0.267),  // #ef4444 - delete/error
            accent_purple: Color::from_rgb(0.545, 0.361, 0.965), // #8b5cf6 - violet accent

            // States - Indigo-tinted selections
            selected: Color::from_rgb(0.094, 0.094, 0.125), // Subtle indigo tint
            selected_border: Color::from_rgb(0.388, 0.400, 0.945), // Indigo border
            hover: Color::from_rgb(0.133, 0.133, 0.157),    // #222228
            unread: Color::from_rgb(0.388, 0.400, 0.945),   // Indigo indicator

            // Borders - Subtle but defined
            border_subtle: Color::from_rgb(0.133, 0.133, 0.157), // #222228
            border_medium: Color::from_rgb(0.165, 0.165, 0.196), // #2a2a32
            border_strong: Color::from_rgb(0.224, 0.224, 0.255), // #393941

            // Shadows - Soft depth
            shadow: Color::from_rgba(0.0, 0.0, 0.0, 0.25),
            shadow_medium: Color::from_rgba(0.0, 0.0, 0.0, 0.40),

            // Avatar colors (Air-style - vibrant on dark)
            avatar_purple: Color::from_rgb(0.388, 0.400, 0.945), // #6366f1
            avatar_pink: Color::from_rgb(0.925, 0.286, 0.600),   // #ec4899
            avatar_cyan: Color::from_rgb(0.024, 0.714, 0.831),   // #06b6d4
            avatar_green: Color::from_rgb(0.063, 0.725, 0.506),  // #10b981
            avatar_orange: Color::from_rgb(0.961, 0.620, 0.043), // #f59e0b
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
