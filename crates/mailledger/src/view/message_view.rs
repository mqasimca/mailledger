//! Message content view component with polished styling.

use iced::widget::{Column, button, column, container, row, scrollable, text};
use iced::{Element, Length};

use crate::message::Message;
use crate::model::MessageContent;
use crate::style::widgets::{
    message_content_style, message_header_style, palette, scrollable_style, toolbar_button_style,
    toolbar_style,
};

/// Renders the message content panel (right pane) with polished styling.
pub fn view_message_content(content: Option<&MessageContent>) -> Element<'static, Message> {
    content.map_or_else(view_empty, view_message)
}

/// Renders empty state when no message is selected.
fn view_empty() -> Element<'static, Message> {
    container(
        column![
            text("\u{1F4E7}").size(64), // envelope icon
            text("Select a message to read").size(16).style(|_theme| {
                let p = palette::current();
                text::Style {
                    color: Some(p.text_secondary),
                }
            }),
        ]
        .spacing(16)
        .align_x(iced::Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .style(message_content_style)
    .into()
}

/// Renders message content with polished styling.
fn view_message(msg: &MessageContent) -> Element<'static, Message> {
    // Action toolbar
    let toolbar = view_toolbar();

    // Header section
    let header = view_header(msg);

    // Body content
    let body = view_body(msg);

    let content = column![
        toolbar,
        header,
        scrollable(body)
            .height(Length::Fill)
            .style(scrollable_style)
    ]
    .spacing(0)
    .width(Length::Fill);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(message_content_style)
        .into()
}

/// Renders the message action toolbar with polished buttons.
fn view_toolbar() -> Element<'static, Message> {
    let reply_btn = button(
        row![
            text("\u{21A9}").size(14),
            text("Reply").font(iced::Font {
                weight: iced::font::Weight::Medium,
                ..Default::default()
            })
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center),
    )
    .padding([8, 14])
    .style(toolbar_button_style)
    .on_press(Message::Reply);

    let reply_all_btn = button(
        row![
            text("\u{21AA}").size(14),
            text("Reply All").font(iced::Font {
                weight: iced::font::Weight::Medium,
                ..Default::default()
            })
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center),
    )
    .padding([8, 14])
    .style(toolbar_button_style)
    .on_press(Message::ReplyAll);

    let forward_btn = button(
        row![
            text("\u{2192}").size(14),
            text("Forward").font(iced::Font {
                weight: iced::font::Weight::Medium,
                ..Default::default()
            })
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center),
    )
    .padding([8, 14])
    .style(toolbar_button_style)
    .on_press(Message::Forward);

    let delete_btn = button(text("\u{1F5D1}").size(16).style(|_theme| {
        let p = palette::current();
        text::Style {
            color: Some(p.accent_red),
        }
    }))
    .padding([8, 12])
    .style(toolbar_button_style)
    .on_press(Message::DeleteSelected);

    let spacer = iced::widget::Space::new().width(Length::Fill);

    let toolbar = row![reply_btn, reply_all_btn, forward_btn, spacer, delete_btn]
        .spacing(8)
        .padding([12, 20])
        .align_y(iced::Alignment::Center);

    container(toolbar)
        .width(Length::Fill)
        .style(toolbar_style)
        .into()
}

/// Renders the message header (from, to, subject, date) with polished styling.
fn view_header(msg: &MessageContent) -> Element<'static, Message> {
    // Subject - large and bold
    let subject = text(msg.subject.clone())
        .size(22)
        .font(iced::Font {
            weight: iced::font::Weight::Bold,
            ..Default::default()
        })
        .style(|_theme| {
            let p = palette::current();
            text::Style {
                color: Some(p.text_primary),
            }
        });

    // From field
    let from_row = view_field_row("From", &format!("{} <{}>", msg.from_name, msg.from_email));

    // To field
    let to_row = view_field_row("To", &msg.to.join(", "));

    // Build header fields
    let mut header_fields: Vec<Element<'static, Message>> = vec![subject.into(), from_row, to_row];

    // CC field (if present)
    if !msg.cc.is_empty() {
        header_fields.push(view_field_row("Cc", &msg.cc.join(", ")));
    }

    // Date field
    header_fields.push(view_field_row("Date", &msg.date));

    let header_col = Column::with_children(header_fields)
        .spacing(8)
        .padding([20, 24]);

    container(header_col)
        .width(Length::Fill)
        .style(message_header_style)
        .into()
}

/// Helper to create a field row (label: value).
fn view_field_row(label: &str, value: &str) -> Element<'static, Message> {
    let label_owned = format!("{label}:");
    let value_owned = value.to_string();

    let label_text = text(label_owned)
        .size(13)
        .font(iced::Font {
            weight: iced::font::Weight::Medium,
            ..Default::default()
        })
        .style(|_theme| {
            let p = palette::current();
            text::Style {
                color: Some(p.text_muted),
            }
        });

    let value_text = text(value_owned).size(13).style(|_theme| {
        let p = palette::current();
        text::Style {
            color: Some(p.text_primary),
        }
    });

    row![container(label_text).width(Length::Fixed(50.0)), value_text]
        .spacing(8)
        .align_y(iced::Alignment::Start)
        .into()
}

/// Renders the message body with polished styling.
fn view_body(msg: &MessageContent) -> Element<'static, Message> {
    // Prefer plain text, fall back to HTML converted to text
    let body_text = if let Some(plain) = msg.body_text.as_ref()
        && !plain.trim().is_empty()
    {
        plain.clone()
    } else if let Some(html) = msg.body_html.as_ref() {
        html_to_text(html)
    } else {
        String::from("(No content)")
    };

    let body = text(body_text).size(14).style(|_theme| {
        let p = palette::current();
        text::Style {
            color: Some(p.text_primary),
        }
    });

    container(body)
        .width(Length::Fill)
        .padding([20, 24])
        .style(message_content_style)
        .into()
}

/// Convert HTML to plain text for display.
///
/// This is a simple conversion that:
/// - Strips HTML tags
/// - Converts block elements to line breaks
/// - Decodes common HTML entities
#[allow(clippy::too_many_lines)]
fn html_to_text(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut last_was_block = false;
    let mut chars = html.chars();

    while let Some(c) = chars.next() {
        match c {
            '<' => {
                in_tag = true;
                // Check for block-level tags that should insert line breaks
                let tag_start: String = chars
                    .clone()
                    .take_while(|&c| c.is_ascii_alphabetic() || c == '/')
                    .collect();
                let tag_lower = tag_start.to_lowercase();

                if matches!(
                    tag_lower.as_str(),
                    "br" | "/p"
                        | "/div"
                        | "/tr"
                        | "/li"
                        | "/h1"
                        | "/h2"
                        | "/h3"
                        | "/h4"
                        | "/h5"
                        | "/h6"
                        | "/blockquote"
                ) {
                    if !last_was_block {
                        result.push('\n');
                        last_was_block = true;
                    }
                } else if matches!(tag_lower.as_str(), "p" | "div" | "tr" | "li") {
                    // Opening block tags also add line break if needed
                    if !result.is_empty() && !last_was_block {
                        result.push('\n');
                        last_was_block = true;
                    }
                }
            }
            '>' => {
                in_tag = false;
            }
            '&' if !in_tag => {
                // Handle HTML entities
                let entity: String = chars.clone().take_while(|&c| c != ';').collect();
                if chars.clone().any(|c| c == ';') {
                    // Skip entity characters
                    for _ in 0..=entity.len() {
                        chars.next();
                    }

                    let decoded = match entity.as_str() {
                        "amp" => "&",
                        "lt" => "<",
                        "gt" => ">",
                        "quot" => "\"",
                        "apos" => "'",
                        "nbsp" => " ",
                        "mdash" | "#8212" => "\u{2014}",  // —
                        "ndash" | "#8211" => "\u{2013}",  // –
                        "hellip" | "#8230" => "\u{2026}", // …
                        "ldquo" | "#8220" => "\u{201C}",  // "
                        "rdquo" | "#8221" => "\u{201D}",  // "
                        "lsquo" | "#8216" => "\u{2018}",  // '
                        "rsquo" | "#8217" => "\u{2019}",  // '
                        "copy" | "#169" => "\u{00A9}",    // ©
                        "reg" | "#174" => "\u{00AE}",     // ®
                        "trade" | "#8482" => "\u{2122}",  // ™
                        _ => {
                            // Try numeric entities
                            if let Some(code) = entity
                                .strip_prefix("#x")
                                .and_then(|hex| u32::from_str_radix(hex, 16).ok())
                                .or_else(|| entity.strip_prefix('#')?.parse::<u32>().ok())
                                && let Some(ch) = char::from_u32(code)
                            {
                                result.push(ch);
                                last_was_block = false;
                                continue;
                            }
                            // Unknown entity, keep original
                            result.push('&');
                            result.push_str(&entity);
                            result.push(';');
                            last_was_block = false;
                            continue;
                        }
                    };
                    result.push_str(decoded);
                    last_was_block = false;
                } else {
                    // Not a valid entity
                    result.push('&');
                }
            }
            '\n' | '\r' if !in_tag => {
                // Normalize whitespace but don't double up
                if !result.is_empty() && !result.ends_with(' ') && !result.ends_with('\n') {
                    result.push(' ');
                }
            }
            _ if !in_tag => {
                // Normal character
                if c.is_whitespace() {
                    if !result.is_empty() && !result.ends_with(' ') && !result.ends_with('\n') {
                        result.push(' ');
                    }
                } else {
                    result.push(c);
                    last_was_block = false;
                }
            }
            _ => {}
        }
    }

    // Clean up: normalize multiple newlines
    let mut cleaned = String::with_capacity(result.len());
    let mut prev_newlines = 0;

    for c in result.chars() {
        if c == '\n' {
            prev_newlines += 1;
            if prev_newlines <= 2 {
                cleaned.push(c);
            }
        } else {
            prev_newlines = 0;
            cleaned.push(c);
        }
    }

    cleaned.trim().to_string()
}
