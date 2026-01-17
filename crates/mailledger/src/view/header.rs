//! Header/toolbar view component with polished styling.

use iced::widget::{Row, button, container, row, text, text_input};
use iced::{Element, Length};

use crate::message::Message;
use crate::style::widgets::{
    header_style, palette, primary_button_style, search_input_style, secondary_button_style,
};

/// Renders the application header/toolbar with glossy styling.
pub fn view_header(search_query: &str) -> Element<'static, Message> {
    // App title with branding
    let title = text("MailLedger")
        .size(22)
        .font(iced::Font {
            weight: iced::font::Weight::Bold,
            ..Default::default()
        })
        .style(|_theme| {
            let p = palette::current();
            text::Style {
                color: Some(p.primary),
            }
        });

    // Hamburger menu button
    let hamburger = button(text("\u{2630}").size(20).style(|_theme| {
        let p = palette::current();
        text::Style {
            color: Some(p.text_secondary),
        }
    }))
    .padding([8, 12])
    .style(secondary_button_style)
    .on_press(Message::ToggleSidebar);

    // Search input with rounded style
    let search = text_input("Search messages...", search_query)
        .width(Length::Fixed(320.0))
        .padding([10, 16])
        .style(search_input_style)
        .on_input(Message::SearchQueryChanged)
        .on_submit(Message::SearchExecute);

    // Compose button with glossy primary style
    let compose_btn = button(
        row![
            text("\u{270F}").size(14),
            text(" Compose").font(iced::Font {
                weight: iced::font::Weight::Semibold,
                ..Default::default()
            })
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center),
    )
    .padding([10, 20])
    .style(primary_button_style)
    .on_press(Message::ComposeNew);

    // Refresh button
    let refresh_btn = button(text("\u{21BB}").size(18).style(|_theme| {
        let p = palette::current();
        text::Style {
            color: Some(p.text_secondary),
        }
    }))
    .padding([8, 12])
    .style(secondary_button_style)
    .on_press(Message::RefreshMessages);

    // Settings button
    let settings_btn = button(text("\u{2699}").size(20).style(|_theme| {
        let p = palette::current();
        text::Style {
            color: Some(p.text_secondary),
        }
    }))
    .padding([8, 12])
    .style(secondary_button_style)
    .on_press(Message::NavigateTo(crate::message::View::Settings));

    // Spacer for layout
    let spacer = iced::widget::Space::new().width(Length::Fill);

    let header_content: Row<'_, Message> = row![
        hamburger,
        title,
        spacer,
        search,
        iced::widget::Space::new().width(16),
        compose_btn,
        iced::widget::Space::new().width(8),
        refresh_btn,
        iced::widget::Space::new().width(8),
        settings_btn,
    ]
    .spacing(12)
    .padding([12, 20])
    .align_y(iced::Alignment::Center);

    container(header_content)
        .width(Length::Fill)
        .style(header_style)
        .into()
}
