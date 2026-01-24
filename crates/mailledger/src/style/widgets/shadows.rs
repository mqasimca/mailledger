//! Shadow presets and rounded corner radii.
//!
//! Includes Air-style glow effects for primary buttons.

use iced::{Color, Shadow, Vector};

use super::palette;

/// Rounded corner radii - terminal style, subtle edges.
pub mod radius {
    pub const NONE: f32 = 0.0;
    pub const SMALL: f32 = 4.0; // Minimal rounding
    pub const MEDIUM: f32 = 6.0; // Subtle rounding
    pub const LARGE: f32 = 8.0; // Moderate rounding
    pub const XLARGE: f32 = 10.0; // Still clean, not too round
    pub const PILL: f32 = 9999.0; // Fully rounded (rarely used in terminal UIs)
}

pub fn none() -> Shadow {
    Shadow::default()
}

pub const fn subtle() -> Shadow {
    Shadow {
        color: palette::SHADOW,
        offset: Vector::new(0.0, 1.0),
        blur_radius: 3.0,
    }
}

pub const fn small() -> Shadow {
    Shadow {
        color: palette::SHADOW,
        offset: Vector::new(0.0, 2.0),
        blur_radius: 6.0,
    }
}

pub const fn medium() -> Shadow {
    Shadow {
        color: palette::SHADOW_MEDIUM,
        offset: Vector::new(0.0, 4.0),
        blur_radius: 12.0,
    }
}

pub const fn large() -> Shadow {
    Shadow {
        color: palette::SHADOW_MEDIUM,
        offset: Vector::new(0.0, 8.0),
        blur_radius: 24.0,
    }
}

/// Glow effect - colored shadow for Air-style buttons.
/// Creates a subtle colored aura around elements.
pub const fn glow(color: Color) -> Shadow {
    Shadow {
        color: Color::from_rgba(color.r, color.g, color.b, 0.3),
        offset: Vector::new(0.0, 2.0),
        blur_radius: 12.0,
    }
}

/// Strong glow effect - for hover states.
pub const fn glow_strong(color: Color) -> Shadow {
    Shadow {
        color: Color::from_rgba(color.r, color.g, color.b, 0.5),
        offset: Vector::new(0.0, 4.0),
        blur_radius: 20.0,
    }
}
