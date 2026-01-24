//! Header/toolbar view component with polished styling.

use std::collections::HashSet;

use iced::widget::{Row, button, container, row, text, text_input};
use iced::{Background, Border, Element, Length};

use crate::message::{Message, SearchFilter};
use crate::style::widgets::{
    header_style, palette, primary_button_style, search_input_style, secondary_button_style,
};

/// Renders the application header/toolbar with glossy styling.
#[allow(clippy::too_many_lines)]
pub fn view_header(
    search_query: &str,
    search_filters: &HashSet<SearchFilter>,
    is_offline: bool,
) -> Element<'static, Message> {
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
        .width(Length::Fixed(240.0))
        .padding([10, 16])
        .style(search_input_style)
        .on_input(Message::SearchQueryChanged)
        .on_submit(Message::SearchExecute);

    // Filter chips
    let unread_chip = view_filter_chip(
        "Unread",
        SearchFilter::Unread,
        search_filters.contains(&SearchFilter::Unread),
    );
    let flagged_chip = view_filter_chip(
        "Starred",
        SearchFilter::Flagged,
        search_filters.contains(&SearchFilter::Flagged),
    );
    let attach_chip = view_filter_chip(
        "Attachments",
        SearchFilter::HasAttachments,
        search_filters.contains(&SearchFilter::HasAttachments),
    );

    let filter_chips = row![unread_chip, flagged_chip, attach_chip].spacing(6);

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

    // Offline indicator
    let offline_indicator: Element<'static, Message> = if is_offline {
        container(
            text("Offline")
                .size(11)
                .font(iced::Font {
                    weight: iced::font::Weight::Bold,
                    ..Default::default()
                })
                .style(|_theme| {
                    let p = palette::current();
                    text::Style {
                        color: Some(p.text_on_primary),
                    }
                }),
        )
        .padding([4, 8])
        .style(|_theme| {
            let p = palette::current();
            container::Style {
                background: Some(Background::Color(p.accent_yellow)),
                border: Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        })
        .into()
    } else {
        iced::widget::Space::new().width(0).into()
    };

    // Spacer for layout
    let spacer = iced::widget::Space::new().width(Length::Fill);

    let header_content: Row<'_, Message> = row![
        hamburger,
        title,
        offline_indicator,
        spacer,
        search,
        iced::widget::Space::new().width(8),
        filter_chips,
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

/// Creates a filter chip button.
fn view_filter_chip(
    label: &str,
    filter: SearchFilter,
    is_active: bool,
) -> Element<'static, Message> {
    let btn = button(text(label.to_string()).size(12))
        .padding([6, 12])
        .style(move |_theme, status| {
            let p = palette::current();
            let (bg, text_color, border_color) = if is_active {
                (p.primary, p.text_on_primary, p.primary)
            } else {
                match status {
                    button::Status::Hovered => (p.hover, p.text_primary, p.border_medium),
                    _ => (p.surface, p.text_secondary, p.border_subtle),
                }
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color,
                border: Border {
                    color: border_color,
                    width: 1.0,
                    radius: 16.0.into(),
                },
                ..Default::default()
            }
        })
        .on_press(Message::ToggleSearchFilter(filter));

    btn.into()
}
