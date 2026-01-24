//! Button style functions with theme support.

use iced::widget::button;
use iced::{Background, Border, Color};

use super::palette;
use super::shadows;
use super::shadows::radius;

/// Primary button style - Air-style with glow effect.
pub fn primary_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let p = palette::current();

    let base = button::Style {
        background: Some(Background::Color(p.primary)),
        text_color: p.text_on_primary,
        border: Border {
            color: p.primary_light,
            width: 1.0,
            radius: radius::MEDIUM.into(),
        },
        shadow: shadows::glow(p.primary), // Glow effect
        snap: false,
    };

    match status {
        button::Status::Active => base,
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(p.primary_light)),
            border: Border {
                color: p.primary_light,
                width: 1.0,
                radius: radius::MEDIUM.into(),
            },
            shadow: shadows::glow_strong(p.primary), // Stronger glow on hover
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(p.primary_dark)),
            shadow: shadows::subtle(), // Pressed down feel
            ..base
        },
        button::Status::Disabled => button::Style {
            background: Some(Background::Color(p.text_muted)),
            text_color: p.surface,
            shadow: shadows::none(),
            ..base
        },
    }
}

/// Secondary/ghost button style - rounded with subtle hover.
pub fn secondary_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let p = palette::current();

    let base = button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: p.text_primary,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: radius::LARGE.into(), // More rounded
        },
        shadow: shadows::none(),
        snap: false,
    };

    match status {
        button::Status::Active | button::Status::Disabled => base,
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(p.hover)),
            border: Border {
                color: p.border_subtle,
                width: 1.0,
                radius: radius::LARGE.into(),
            },
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(p.selected)),
            ..base
        },
    }
}

/// Folder item button style - modern rounded.
pub fn folder_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let p = palette::current();

    let base = button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: p.text_primary,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: radius::MEDIUM.into(), // More rounded
        },
        shadow: shadows::none(),
        snap: false,
    };

    match status {
        button::Status::Active | button::Status::Disabled => base,
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(p.hover)),
            border: Border {
                color: p.border_subtle,
                width: 1.0,
                radius: radius::MEDIUM.into(),
            },
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(p.selected)),
            ..base
        },
    }
}

/// Selected folder button style - Air-style left accent border.
pub fn folder_button_selected_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let p = palette::current();

    // Air uses a subtle background tint with left border accent
    let base = button::Style {
        background: Some(Background::Color(p.selected)),
        text_color: p.primary,
        border: Border {
            color: p.primary, // Indigo accent on left (simulated with full border)
            width: 2.0,       // Visible accent
            radius: radius::SMALL.into(),
        },
        shadow: shadows::none(),
        snap: false,
    };

    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(p.hover)),
            ..base
        },
        _ => base,
    }
}

/// Message row button style - normal.
pub fn message_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let p = palette::current();

    let base = button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: p.text_primary,
        border: Border::default(),
        shadow: shadows::none(),
        snap: false,
    };

    match status {
        button::Status::Active | button::Status::Disabled => base,
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(p.hover)),
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(p.selected)),
            ..base
        },
    }
}

/// Ghost button style - transparent with subtle border on hover.
pub fn ghost_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let p = palette::current();

    let base = button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: p.text_primary,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: radius::MEDIUM.into(),
        },
        shadow: shadows::none(),
        snap: false,
    };

    match status {
        button::Status::Active | button::Status::Disabled => base,
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(p.hover)),
            border: Border {
                color: p.border_subtle,
                width: 1.0,
                radius: radius::MEDIUM.into(),
            },
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(p.selected)),
            ..base
        },
    }
}

/// Toolbar button style.
pub fn toolbar_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let p = palette::current();

    let base = button::Style {
        background: Some(Background::Color(p.surface)),
        text_color: p.text_primary,
        border: Border {
            color: p.border_subtle,
            width: 1.0,
            radius: radius::MEDIUM.into(),
        },
        shadow: shadows::none(), // No shadows in terminal UI
        snap: false,
    };

    match status {
        button::Status::Active => base,
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(p.hover)),
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(p.selected)),
            ..base
        },
        button::Status::Disabled => button::Style {
            text_color: p.text_muted,
            ..base
        },
    }
}
