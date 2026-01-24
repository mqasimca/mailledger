//! Message content view component with styled text rendering.
//!
//! Uses HTML → text conversion for email display with proper styling.

use iced::widget::{Column, button, column, container, image, markdown, row, scrollable, text};
use iced::{Background, Border, ContentFit, Element, Length};

use crate::message::{Message, SnoozeDuration};
use crate::model::{FontSize, InlineImage, InlineImageState, MessageContent, MessageId};
use crate::style::widgets::{
    message_content_style, message_header_style, palette, scrollable_style, toolbar_button_style,
    toolbar_style,
};

/// Renders the message content panel (right pane) with styled text.
#[allow(clippy::option_if_let_else)] // match is clearer here with lifetimes
pub fn view_message_content<'a>(
    content: Option<&MessageContent>,
    markdown_items: &'a [markdown::Item],
    inline_images: &[InlineImage],
    is_read: bool,
    quoted_expanded: bool,
    font_size: FontSize,
    snooze_dropdown_open: bool,
) -> Element<'a, Message> {
    match content {
        Some(msg) => view_message(
            msg,
            markdown_items,
            inline_images,
            is_read,
            quoted_expanded,
            font_size,
            snooze_dropdown_open,
        ),
        None => view_empty(),
    }
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

/// Renders message content with styled text.
fn view_message<'a>(
    msg: &MessageContent,
    markdown_items: &'a [markdown::Item],
    inline_images: &[InlineImage],
    is_read: bool,
    quoted_expanded: bool,
    font_size: FontSize,
    snooze_dropdown_open: bool,
) -> Element<'a, Message> {
    // Action toolbar
    let toolbar = view_toolbar(
        msg.body_html.is_some(),
        msg.id,
        is_read,
        snooze_dropdown_open,
    );

    // Header section
    let header = view_header(msg, font_size);

    // Attachments section (if any)
    let attachments = view_attachments(msg, font_size);

    // Body content with styled text
    let body = view_body(markdown_items, inline_images, font_size);

    // Quote toggle (only show if there's quoted text)
    let has_quotes = msg
        .body_text
        .as_ref()
        .is_some_and(|text| has_quoted_content(text));

    let quote_toggle = if has_quotes {
        view_quote_toggle(quoted_expanded)
    } else {
        column![].into()
    };

    let content = column![
        toolbar,
        header,
        attachments,
        scrollable(column![body, quote_toggle].spacing(0))
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
#[allow(clippy::too_many_lines)]
fn view_toolbar(
    has_html: bool,
    message_id: MessageId,
    is_read: bool,
    snooze_dropdown_open: bool,
) -> Element<'static, Message> {
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

    // Archive button
    let archive_btn = button(
        row![
            text("\u{1F4E5}").size(14), // inbox tray / archive icon
            text("Archive").font(iced::Font {
                weight: iced::font::Weight::Medium,
                ..Default::default()
            })
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center),
    )
    .padding([8, 14])
    .style(toolbar_button_style)
    .on_press(Message::ArchiveMessage(message_id));

    // Snooze button with dropdown
    let snooze_widget = view_snooze_button(snooze_dropdown_open);

    // Mark as read/unread toggle button
    let (read_icon, read_label) = if is_read {
        ("\u{2709}", "Mark Unread") // envelope icon
    } else {
        ("\u{2709}\u{FE0F}", "Mark Read") // envelope with variant
    };

    let read_toggle_btn = button(
        row![
            text(read_icon).size(14),
            text(read_label).font(iced::Font {
                weight: iced::font::Weight::Medium,
                ..Default::default()
            })
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center),
    )
    .padding([8, 14])
    .style(toolbar_button_style)
    .on_press(Message::ToggleRead(message_id));

    let delete_btn = button(text("\u{1F5D1}").size(16).style(|_theme| {
        let p = palette::current();
        text::Style {
            color: Some(p.accent_red),
        }
    }))
    .padding([8, 12])
    .style(toolbar_button_style)
    .on_press(Message::DeleteSelected);

    let view_html_btn = button(text("View HTML").size(14))
        .padding([8, 14])
        .style(toolbar_button_style)
        .on_press_maybe(if has_html {
            Some(Message::OpenHtml)
        } else {
            None
        });

    let spacer = iced::widget::Space::new().width(Length::Fill);

    let toolbar = row![
        reply_btn,
        reply_all_btn,
        forward_btn,
        archive_btn,
        snooze_widget,
        read_toggle_btn,
        view_html_btn,
        spacer,
        delete_btn
    ]
    .spacing(8)
    .padding([12, 20])
    .align_y(iced::Alignment::Center);

    container(toolbar)
        .width(Length::Fill)
        .style(toolbar_style)
        .into()
}

/// Renders the snooze button with dropdown.
fn view_snooze_button(dropdown_open: bool) -> Element<'static, Message> {
    // Main snooze button
    let snooze_btn = button(
        row![
            text("\u{1F4A4}").size(14), // zzz / sleep icon
            text("Snooze").font(iced::Font {
                weight: iced::font::Weight::Medium,
                ..Default::default()
            }),
            text(if dropdown_open {
                "\u{25B2}"
            } else {
                "\u{25BC}"
            })
            .size(10) // up/down arrow
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center),
    )
    .padding([8, 14])
    .style(toolbar_button_style)
    .on_press(Message::ToggleSnoozeDropdown);

    if dropdown_open {
        // Show dropdown with snooze options
        let options = column![
            view_snooze_option("Later today (3 hours)", SnoozeDuration::LaterToday),
            view_snooze_option("Tomorrow morning", SnoozeDuration::Tomorrow),
            view_snooze_option("Next Monday", SnoozeDuration::NextWeek),
        ]
        .spacing(0);

        let dropdown = container(options)
            .style(move |_theme| {
                let p = palette::current();
                container::Style {
                    background: Some(Background::Color(p.surface_elevated)),
                    border: Border {
                        color: p.border_medium,
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                }
            })
            .width(Length::Fixed(180.0));

        column![snooze_btn, dropdown].spacing(4).into()
    } else {
        snooze_btn.into()
    }
}

/// Renders a single snooze option in the dropdown.
fn view_snooze_option(label: &str, duration: SnoozeDuration) -> Element<'static, Message> {
    button(text(label.to_string()).size(13))
        .width(Length::Fill)
        .padding([10, 12])
        .style(move |_theme, status| {
            let p = palette::current();
            let bg = match status {
                button::Status::Hovered | button::Status::Pressed => p.hover,
                _ => p.surface_elevated,
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color: p.text_primary,
                border: Border::default(),
                ..Default::default()
            }
        })
        .on_press(Message::SnoozeSelected(duration))
        .into()
}

/// Renders the message header (from, to, subject, date) with polished styling.
fn view_header(msg: &MessageContent, font_size: FontSize) -> Element<'static, Message> {
    let heading = font_size.heading_size();
    let base = font_size.base_size();

    // Subject - large and bold
    // Scale heading: Small=18, Medium=22, Large=26
    let subject_size = heading + 4;
    let subject = text(msg.subject.clone())
        .size(subject_size)
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
    let from_row = view_field_row(
        "From",
        &format!("{} <{}>", msg.from_name, msg.from_email),
        base,
    );

    // To field
    let to_row = view_field_row("To", &msg.to.join(", "), base);

    // Build header fields
    let mut header_fields: Vec<Element<'static, Message>> = vec![subject.into(), from_row, to_row];

    // CC field (if present)
    if !msg.cc.is_empty() {
        header_fields.push(view_field_row("Cc", &msg.cc.join(", "), base));
    }

    // Date field
    header_fields.push(view_field_row("Date", &msg.date, base));

    let header_col = Column::with_children(header_fields)
        .spacing(8)
        .padding([20, 24]);

    container(header_col)
        .width(Length::Fill)
        .style(message_header_style)
        .into()
}

/// Helper to create a field row (label: value).
fn view_field_row(label: &str, value: &str, base_size: u32) -> Element<'static, Message> {
    let label_owned = format!("{label}:");
    let value_owned = value.to_string();
    let field_size = base_size - 1;

    let label_text = text(label_owned)
        .size(field_size)
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

    let value_text = text(value_owned).size(field_size).style(|_theme| {
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

/// Renders the attachments section if the message has attachments.
fn view_attachments(msg: &MessageContent, font_size: FontSize) -> Element<'static, Message> {
    if msg.attachments.is_empty() {
        return iced::widget::Space::new().width(Length::Shrink).into();
    }

    let p = palette::current();
    let message_id = msg.id;
    let base = font_size.base_size();
    let snippet = font_size.snippet_size();

    let attachment_chips: Vec<Element<'static, Message>> = msg
        .attachments
        .iter()
        .map(|att| {
            let filename = att.filename.clone();
            let size_display = att.size_display();
            let part_number = att.part_number.clone();
            let encoding = att.encoding.clone();

            let chip = button(
                row![
                    text("\u{1F4CE}").size(base), // paperclip icon
                    column![
                        text(filename.clone()).size(base - 1).font(iced::Font {
                            weight: iced::font::Weight::Medium,
                            ..Default::default()
                        }),
                        text(size_display)
                            .size(snippet)
                            .style(move |_theme: &iced::Theme| text::Style {
                                color: Some(p.text_muted),
                            }),
                    ]
                    .spacing(2),
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
            )
            .padding([8, 12])
            .style(|theme, status| {
                let p = palette::current();
                let background = match status {
                    button::Status::Hovered | button::Status::Pressed => p.surface_elevated,
                    _ => p.surface,
                };
                button::Style {
                    background: Some(iced::Background::Color(background)),
                    border: iced::Border {
                        color: p.border_medium,
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    text_color: theme.palette().text,
                    ..Default::default()
                }
            })
            .on_press(Message::DownloadAttachment {
                message_id,
                part_number,
                filename,
                encoding,
            });

            chip.into()
        })
        .collect();

    let attachments_row = iced::widget::Row::with_children(attachment_chips)
        .spacing(8)
        .wrap();

    let section = column![
        text("Attachments")
            .size(snippet)
            .font(iced::Font {
                weight: iced::font::Weight::Medium,
                ..Default::default()
            })
            .style(move |_theme| text::Style {
                color: Some(p.text_muted),
            }),
        attachments_row,
    ]
    .spacing(8)
    .padding([12, 24]);

    container(section)
        .width(Length::Fill)
        .style(|_theme| {
            let p = palette::current();
            container::Style {
                background: Some(iced::Background::Color(p.surface_elevated)),
                border: iced::Border {
                    color: p.border_subtle,
                    width: 0.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            }
        })
        .into()
}

/// Renders the message body with markdown support.
///
/// Uses pre-parsed markdown items from the model for efficient rendering.
fn view_body<'a>(
    markdown_items: &'a [markdown::Item],
    inline_images: &[InlineImage],
    font_size: FontSize,
) -> Element<'a, Message> {
    // Create custom palette with maximum contrast for readability
    // Use surface color (matches container background) for proper contrast calculation
    let p = palette::current();
    let custom_palette = iced::theme::Palette {
        background: p.surface,  // Match container background for correct contrast
        text: p.text_primary,   // Use theme's primary text color (near-white in dark mode)
        primary: p.accent_blue, // Blue for links (more visible than indigo)
        success: p.accent_green,
        warning: p.accent_yellow,
        danger: p.accent_red,
    };

    let style = markdown::Style::from_palette(custom_palette);
    // Scale body text with font size preference
    let body_text_size = font_size.base_size() + 1; // Slightly larger for readability
    let settings = markdown::Settings::with_text_size(body_text_size, style);

    // Create markdown view with link click handler
    let md_view: Element<'a, String> = markdown::view(markdown_items, settings);
    let md_view = md_view.map(Message::LinkClicked);

    let mut content: Column<'a, Message> = column![md_view].spacing(16);

    // Add inline images if present
    if !inline_images.is_empty() {
        let images = Column::with_children(
            inline_images
                .iter()
                .map(view_inline_image)
                .collect::<Vec<_>>(),
        )
        .spacing(12);

        content = content.push(images);
    }

    container(content)
        .width(Length::Fill)
        .padding([20, 24])
        .style(message_content_style)
        .into()
}

/// Render an inline image.
fn view_inline_image(image_entry: &InlineImage) -> Element<'static, Message> {
    let p = palette::current();
    match &image_entry.state {
        InlineImageState::Loading => text("Loading image...")
            .size(13)
            .style(move |_theme| text::Style {
                color: Some(p.text_muted),
            })
            .into(),
        InlineImageState::Failed(err) => text(format!("Image failed to load: {err}"))
            .size(13)
            .style(move |_theme| text::Style {
                color: Some(p.accent_red),
            })
            .into(),
        InlineImageState::Ready(handle) => container(
            image(handle.clone())
                .width(Length::Fixed(200.0))
                .content_fit(ContentFit::ScaleDown),
        )
        .into(),
    }
}

/// Checks if text contains quoted content (lines starting with > or "On ... wrote:").
fn has_quoted_content(text: &str) -> bool {
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('>') {
            return true;
        }
        // Check for common reply patterns
        if trimmed.starts_with("On ") && trimmed.contains(" wrote:") {
            return true;
        }
        // Check for forwarded message marker
        if trimmed.contains("---------- Forwarded message ----------") {
            return true;
        }
    }
    false
}

/// Renders a toggle button for showing/hiding quoted text.
fn view_quote_toggle(expanded: bool) -> Element<'static, Message> {
    let p = palette::current();

    let (icon, label) = if expanded {
        ("\u{25B2}", "Hide quoted text") // up arrow
    } else {
        ("\u{25BC}", "Show quoted text") // down arrow
    };

    let toggle_btn = button(
        row![
            text(icon).size(10),
            text(label).size(12).style(move |_theme| text::Style {
                color: Some(p.text_secondary),
            })
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center),
    )
    .padding([8, 16])
    .style(move |_theme, status| {
        let p = palette::current();
        button::Style {
            background: Some(iced::Background::Color(match status {
                button::Status::Hovered | button::Status::Pressed => p.hover,
                _ => p.surface,
            })),
            text_color: p.text_secondary,
            border: iced::Border {
                color: p.border_subtle,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        }
    })
    .on_press(Message::ToggleQuotedText);

    container(
        row![
            container(text(""))
                .width(Length::Fill)
                .height(1)
                .style(move |_theme| container::Style {
                    background: Some(iced::Background::Color(p.border_subtle)),
                    ..Default::default()
                }),
            toggle_btn,
            container(text(""))
                .width(Length::Fill)
                .height(1)
                .style(move |_theme| container::Style {
                    background: Some(iced::Background::Color(p.border_subtle)),
                    ..Default::default()
                }),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center),
    )
    .padding([16, 24])
    .into()
}
