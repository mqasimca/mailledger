//! Container style functions with theme support.

use iced::widget::container;
use iced::{Background, Border, Color};

use super::palette;
use super::shadows;
use super::shadows::radius;

/// Header bar style - Air's navigation style with bottom border.
pub fn header_style(_theme: &iced::Theme) -> container::Style {
    let p = palette::current();

    container::Style {
        background: Some(Background::Color(p.surface)),
        border: Border {
            color: p.border_subtle, // Subtle bottom border
            width: 1.0,
            radius: radius::NONE.into(),
        },
        shadow: shadows::none(),
        ..Default::default()
    }
}

/// Sidebar style - Air's elevated surface with right border.
pub fn sidebar_style(_theme: &iced::Theme) -> container::Style {
    let p = palette::current();

    container::Style {
        background: Some(Background::Color(p.surface)), // Elevated surface
        border: Border {
            color: p.border_subtle, // Subtle right border
            width: 1.0,
            radius: radius::NONE.into(),
        },
        ..Default::default()
    }
}

/// Message list panel style.
pub fn message_list_style(_theme: &iced::Theme) -> container::Style {
    let p = palette::current();

    container::Style {
        background: Some(Background::Color(p.surface)),
        border: Border {
            color: p.border_subtle,
            width: 1.0,
            radius: radius::NONE.into(),
        },
        shadow: shadows::none(), // No shadows in terminal UI
        ..Default::default()
    }
}

/// Message content panel style.
pub fn message_content_style(_theme: &iced::Theme) -> container::Style {
    let p = palette::current();

    container::Style {
        background: Some(Background::Color(p.surface)),
        ..Default::default()
    }
}

/// Card style - flat terminal style with minimal shadow.
pub fn card_style(_theme: &iced::Theme) -> container::Style {
    let p = palette::current();

    container::Style {
        background: Some(Background::Color(p.surface_elevated)),
        border: Border {
            color: p.border_subtle,
            width: 1.0,
            radius: radius::MEDIUM.into(), // Minimal rounding
        },
        shadow: shadows::none(), // No shadows in terminal UI
        ..Default::default()
    }
}

/// Selected item style.
pub fn selected_style(_theme: &iced::Theme) -> container::Style {
    let p = palette::current();

    container::Style {
        background: Some(Background::Color(p.selected)),
        border: Border {
            color: p.selected_border,
            width: 1.0,
            radius: radius::SMALL.into(),
        },
        ..Default::default()
    }
}

/// Message row - normal state.
pub fn message_row_style(_theme: &iced::Theme) -> container::Style {
    let p = palette::current();

    container::Style {
        background: Some(Background::Color(p.surface)),
        border: Border {
            color: p.border_subtle,
            width: 0.0,
            radius: radius::NONE.into(),
        },
        ..Default::default()
    }
}

/// Message row - selected state.
pub fn message_row_selected_style(_theme: &iced::Theme) -> container::Style {
    let p = palette::current();

    container::Style {
        background: Some(Background::Color(p.selected)),
        border: Border {
            color: p.selected_border,
            width: 0.0,
            radius: radius::NONE.into(),
        },
        ..Default::default()
    }
}

/// Message row bottom border.
pub fn message_row_border_style(_theme: &iced::Theme) -> container::Style {
    let p = palette::current();

    container::Style {
        border: Border {
            color: p.border_subtle,
            width: 1.0,
            radius: radius::NONE.into(),
        },
        ..Default::default()
    }
}

/// Toolbar container style.
pub fn toolbar_style(_theme: &iced::Theme) -> container::Style {
    let p = palette::current();

    container::Style {
        background: Some(Background::Color(p.surface_elevated)),
        border: Border {
            color: p.border_subtle,
            width: 1.0,
            radius: radius::NONE.into(),
        },
        shadow: shadows::none(), // No shadows in terminal UI
        ..Default::default()
    }
}

/// Elevated card style - for sender cards, etc.
pub fn elevated_card_style(_theme: &iced::Theme) -> container::Style {
    let p = palette::current();

    container::Style {
        background: Some(Background::Color(p.surface_elevated)),
        border: Border {
            color: p.border_subtle,
            width: 1.0,
            radius: radius::MEDIUM.into(),
        },
        shadow: shadows::subtle(),
        ..Default::default()
    }
}

/// Message header style.
pub fn message_header_style(_theme: &iced::Theme) -> container::Style {
    let p = palette::current();

    container::Style {
        background: Some(Background::Color(p.surface)),
        border: Border {
            color: p.border_subtle,
            width: 1.0,
            radius: radius::NONE.into(),
        },
        ..Default::default()
    }
}
