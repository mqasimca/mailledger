//! Message list view component with polished styling.

use iced::widget::{Column, button, column, container, row, scrollable, text};
use iced::{Element, Length};

use crate::message::Message;
use crate::model::{MessageId, MessageSummary};
use crate::style::widgets::{
    message_button_style, message_list_style, message_row_border_style, message_row_selected_style,
    message_row_style, palette, scrollable_style,
};

/// Renders the message list panel with polished styling.
pub fn view_message_list(
    messages: &[MessageSummary],
    selected_message: Option<MessageId>,
) -> Element<'static, Message> {
    if messages.is_empty() {
        return container(
            column![
                text("\u{1F4ED}").size(48), // empty mailbox
                text("No messages").size(16).style(|_theme| {
                    let p = palette::current();
                    text::Style {
                        color: Some(p.text_secondary),
                    }
                }),
            ]
            .spacing(12)
            .align_x(iced::Alignment::Center),
        )
        .width(Length::Fixed(380.0))
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(message_list_style)
        .into();
    }

    let message_rows: Vec<Element<'static, Message>> = messages
        .iter()
        .map(|msg| view_message_row(msg, selected_message))
        .collect();

    let list = Column::with_children(message_rows);

    container(
        scrollable(list)
            .height(Length::Fill)
            .style(scrollable_style),
    )
    .width(Length::Fixed(380.0))
    .height(Length::Fill)
    .style(message_list_style)
    .into()
}

/// Renders a single message row in the list with polished styling.
fn view_message_row(
    msg: &MessageSummary,
    selected: Option<MessageId>,
) -> Element<'static, Message> {
    let is_selected = selected == Some(msg.id);

    // Sender name - bold if unread
    let from_weight = if msg.is_read {
        iced::font::Weight::Normal
    } else {
        iced::font::Weight::Semibold
    };

    let from = text(msg.from_name.clone())
        .size(14)
        .font(iced::Font {
            weight: from_weight,
            ..Default::default()
        })
        .style(|_theme| {
            let p = palette::current();
            text::Style {
                color: Some(p.text_primary),
            }
        });

    // Date - muted color
    let date = text(msg.date.clone()).size(12).style(|_theme| {
        let p = palette::current();
        text::Style {
            color: Some(p.text_muted),
        }
    });

    // Subject - bold if unread
    let subject_weight = if msg.is_read {
        iced::font::Weight::Normal
    } else {
        iced::font::Weight::Semibold
    };

    let subject = text(msg.subject.clone())
        .size(13)
        .font(iced::Font {
            weight: subject_weight,
            ..Default::default()
        })
        .style(|_theme| {
            let p = palette::current();
            text::Style {
                color: Some(p.text_primary),
            }
        });

    // Snippet
    let snippet = text(truncate(&msg.snippet, 70)).size(12).style(|_theme| {
        let p = palette::current();
        text::Style {
            color: Some(p.text_secondary),
        }
    });

    // Indicators row
    let mut indicators = row![].spacing(6);

    // Flag indicator
    if msg.is_flagged {
        indicators = indicators.push(text("\u{2B50}").size(12).style(|_theme| {
            let p = palette::current();
            text::Style {
                color: Some(p.accent_yellow),
            }
        }));
    }

    // Attachment indicator
    if msg.has_attachments {
        indicators = indicators.push(text("\u{1F4CE}").size(12).style(|_theme| {
            let p = palette::current();
            text::Style {
                color: Some(p.text_muted),
            }
        }));
    }

    // Unread indicator (blue dot)
    if !msg.is_read {
        indicators = indicators.push(container(text("")).width(8).height(8).style(|_theme| {
            let p = palette::current();
            container::Style {
                background: Some(iced::Background::Color(p.unread)),
                border: iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        }));
    }

    // Spacer for date alignment
    let spacer = iced::widget::Space::new().width(Length::Fill);

    let header_row = row![from, indicators, spacer, date]
        .spacing(8)
        .align_y(iced::Alignment::Center);

    let content = column![header_row, subject, snippet]
        .spacing(4)
        .padding([14, 16]);

    // Row styling based on selection
    let row_style = if is_selected {
        message_row_selected_style
    } else {
        message_row_style
    };

    let btn = button(content)
        .width(Length::Fill)
        .padding(0)
        .style(message_button_style)
        .on_press(Message::SelectMessage(msg.id));

    container(container(btn).style(row_style))
        .style(message_row_border_style)
        .into()
}

/// Truncates a string to a maximum length with ellipsis.
fn truncate(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_len.saturating_sub(3)).collect();
        format!("{truncated}...")
    }
}
