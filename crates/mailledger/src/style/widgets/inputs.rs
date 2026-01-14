//! Text input and scrollable style functions.

use iced::widget::{container, scrollable, text_input};
use iced::{Background, Border, Color};

use super::palette;
use super::shadows;
use super::shadows::radius;

/// Search input style - terminal style with minimal rounding.
pub fn search_input_style(_theme: &iced::Theme, status: text_input::Status) -> text_input::Style {
    let p = palette::current();

    let base = text_input::Style {
        background: Background::Color(p.surface_elevated),
        border: Border {
            color: p.border_subtle,
            width: 1.0,
            radius: radius::MEDIUM.into(), // Minimal rounding
        },
        icon: p.text_muted,
        placeholder: p.text_muted,
        value: p.text_primary,
        selection: p.selected,
    };

    match status {
        text_input::Status::Active => base,
        text_input::Status::Hovered => text_input::Style {
            background: Background::Color(p.surface),
            border: Border {
                color: p.border_medium,
                width: 1.0,
                ..base.border
            },
            ..base
        },
        text_input::Status::Focused { .. } => text_input::Style {
            background: Background::Color(p.surface),
            border: Border {
                color: p.primary,
                width: 1.0, // Consistent border width
                ..base.border
            },
            ..base
        },
        text_input::Status::Disabled => text_input::Style {
            background: Background::Color(p.background_secondary),
            value: p.text_muted,
            ..base
        },
    }
}

/// Scrollable style.
pub fn scrollable_style(_theme: &iced::Theme, status: scrollable::Status) -> scrollable::Style {
    let p = palette::current();

    let scroller_border = Border {
        color: Color::TRANSPARENT,
        width: 0.0,
        radius: radius::SMALL.into(), // Minimal rounding for terminal style
    };

    let base = scrollable::Style {
        container: container::Style::default(),
        vertical_rail: scrollable::Rail {
            background: Some(Background::Color(Color::TRANSPARENT)),
            border: Border::default(),
            scroller: scrollable::Scroller {
                background: Background::Color(p.border_medium),
                border: scroller_border,
            },
        },
        horizontal_rail: scrollable::Rail {
            background: Some(Background::Color(Color::TRANSPARENT)),
            border: Border::default(),
            scroller: scrollable::Scroller {
                background: Background::Color(p.border_medium),
                border: scroller_border,
            },
        },
        gap: None,
        auto_scroll: scrollable::AutoScroll {
            background: Background::Color(p.surface),
            border: Border::default(),
            shadow: shadows::none(),
            icon: p.text_muted,
        },
    };

    match status {
        scrollable::Status::Active { .. } => base,
        scrollable::Status::Hovered {
            is_horizontal_scrollbar_hovered,
            is_vertical_scrollbar_hovered,
            ..
        } => {
            let mut style = base;
            if is_vertical_scrollbar_hovered {
                style.vertical_rail.scroller.background = Background::Color(p.primary_light);
            }
            if is_horizontal_scrollbar_hovered {
                style.horizontal_rail.scroller.background = Background::Color(p.primary_light);
            }
            style
        }
        scrollable::Status::Dragged {
            is_horizontal_scrollbar_dragged,
            is_vertical_scrollbar_dragged,
            ..
        } => {
            let mut style = base;
            if is_vertical_scrollbar_dragged {
                style.vertical_rail.scroller.background = Background::Color(p.primary);
            }
            if is_horizontal_scrollbar_dragged {
                style.horizontal_rail.scroller.background = Background::Color(p.primary);
            }
            style
        }
    }
}
