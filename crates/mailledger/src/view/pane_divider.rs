//! Pane divider component for resizable panes.

use iced::widget::{container, mouse_area};
use iced::{Background, Element, Length};

use crate::message::{Message, PaneDivider};
use crate::style::widgets::palette;

/// Renders a draggable pane divider.
pub fn view_pane_divider(divider: PaneDivider) -> Element<'static, Message> {
    let p = palette::current();

    // Invisible hit area for easier grabbing
    let hit_area = container(
        container(iced::widget::Space::new().height(Length::Fill))
            .width(Length::Fixed(2.0))
            .height(Length::Fill)
            .style(move |_theme| container::Style {
                background: Some(Background::Color(p.border_subtle)),
                ..Default::default()
            }),
    )
    .width(Length::Fixed(8.0))
    .height(Length::Fill)
    .align_x(iced::alignment::Horizontal::Center)
    .style(move |_theme| container::Style {
        background: Some(Background::Color(iced::Color::TRANSPARENT)),
        ..Default::default()
    });

    // Wrap in mouse area for drag detection
    mouse_area(hit_area)
        .on_press(Message::StartPaneDrag(divider))
        .on_release(Message::StopPaneDrag)
        .into()
}
