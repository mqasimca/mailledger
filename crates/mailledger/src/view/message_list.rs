//! Message list view component with Air-inspired styling.

use std::collections::HashSet;

use iced::widget::{Column, button, column, container, row, scrollable, text};
use iced::{Background, Border, Element, Length};

use crate::message::Message;
use crate::model::{FontSize, ListDensity, MessageId, MessageSummary, Thread, ViewMode};
use crate::style::widgets::{
    message_button_style, message_list_style, message_row_border_style, message_row_selected_style,
    message_row_style, palette, primary_button_style, scrollable_style, secondary_button_style,
};

/// Renders the message list panel with polished styling and virtual scrolling.
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
pub fn view_message_list(
    messages: &[MessageSummary],
    selected_message: Option<MessageId>,
    is_loading: bool,
    view_mode: ViewMode,
    threads: &[Thread],
    expanded_threads: &HashSet<String>,
    font_size: FontSize,
    list_density: ListDensity,
    scroll_offset: f32,
    viewport_height: f32,
    width: f32,
) -> Element<'static, Message> {
    // Show loading spinner when loading
    if is_loading {
        return container(
            column![
                text("\u{23F3}").size(48), // hourglass spinner
                text("Loading messages...").size(16).style(|_theme| {
                    let p = palette::current();
                    text::Style {
                        color: Some(p.text_secondary),
                    }
                }),
            ]
            .spacing(12)
            .align_x(iced::Alignment::Center),
        )
        .width(Length::Fixed(width))
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(message_list_style)
        .into();
    }

    // Show empty state with compose button
    if messages.is_empty() {
        let compose_btn = button(
            row![text("\u{270F}").size(14), text("Compose").size(14)]
                .spacing(8)
                .align_y(iced::Alignment::Center),
        )
        .padding([10, 20])
        .style(primary_button_style)
        .on_press(Message::ComposeNew);

        return container(
            column![
                text("\u{1F4ED}").size(48), // empty mailbox
                text("No messages").size(16).style(|_theme| {
                    let p = palette::current();
                    text::Style {
                        color: Some(p.text_secondary),
                    }
                }),
                compose_btn,
            ]
            .spacing(12)
            .align_x(iced::Alignment::Center),
        )
        .width(Length::Fixed(width))
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(message_list_style)
        .into();
    }

    // View mode toggle button
    let toggle_icon = match view_mode {
        ViewMode::Flat => "\u{1F4DD}",     // memo for flat
        ViewMode::Threaded => "\u{1F5C2}", // folder for threaded
    };
    let toggle_label = match view_mode {
        ViewMode::Flat => "Threaded",
        ViewMode::Threaded => "Flat",
    };
    let view_mode_toggle = button(
        row![text(toggle_icon).size(12), text(toggle_label).size(12)]
            .spacing(4)
            .align_y(iced::Alignment::Center),
    )
    .padding([6, 12])
    .style(secondary_button_style)
    .on_press(Message::ToggleViewMode);

    let header = container(
        row![
            iced::widget::Space::new().width(Length::Fill),
            view_mode_toggle,
        ]
        .padding([8, 12]),
    )
    .style(|_theme| {
        let p = palette::current();
        container::Style {
            border: Border {
                width: 0.0,
                color: p.border_subtle,
                ..Default::default()
            },
            ..Default::default()
        }
    });

    // Virtual scrolling constants
    let row_height = estimate_row_height(list_density);
    let buffer_rows = 5; // Render extra rows above/below for smooth scrolling

    // Collect all row items first to get total count
    let row_items: Vec<RowItem> = match view_mode {
        ViewMode::Flat => messages
            .iter()
            .map(|msg| RowItem::Message(msg, false))
            .collect(),
        ViewMode::Threaded => {
            let mut items = Vec::new();
            for thread in threads {
                let is_expanded = expanded_threads.contains(&thread.id);
                if thread.message_count() == 1 {
                    if let Some(msg) = messages.iter().find(|m| m.id == thread.messages[0]) {
                        items.push(RowItem::Message(msg, false));
                    }
                } else if is_expanded {
                    items.push(RowItem::ThreadHeader(thread, true));
                    for msg_id in &thread.messages {
                        if let Some(msg) = messages.iter().find(|m| m.id == *msg_id) {
                            items.push(RowItem::Message(msg, true));
                        }
                    }
                } else {
                    items.push(RowItem::ThreadHeader(thread, false));
                }
            }
            items
        }
    };

    let total_rows = row_items.len();

    // Calculate visible range with buffer
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let first_visible = (scroll_offset / row_height).floor() as usize;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let visible_count = (viewport_height / row_height).ceil() as usize + 1;

    let start_idx = first_visible.saturating_sub(buffer_rows);
    let end_idx = (first_visible + visible_count + buffer_rows).min(total_rows);

    // Build visible rows with spacers
    let mut elements: Vec<Element<'static, Message>> = Vec::new();

    // Top spacer
    #[allow(clippy::cast_precision_loss)] // Precision loss insignificant for message counts
    let top_height = (start_idx as f32) * row_height;
    if top_height > 0.0 {
        elements.push(
            iced::widget::Space::new()
                .width(Length::Fill)
                .height(Length::Fixed(top_height))
                .into(),
        );
    }

    // Visible rows
    for item in row_items
        .into_iter()
        .skip(start_idx)
        .take(end_idx - start_idx)
    {
        let element = match item {
            RowItem::Message(msg, indented) => {
                view_message_row(msg, selected_message, indented, font_size, list_density)
            }
            RowItem::ThreadHeader(thread, expanded) => {
                view_thread_header(thread, expanded, font_size, list_density)
            }
        };
        elements.push(element);
    }

    // Bottom spacer
    let bottom_rows = total_rows.saturating_sub(end_idx);
    #[allow(clippy::cast_precision_loss)] // Precision loss insignificant for message counts
    let bottom_height = (bottom_rows as f32) * row_height;
    if bottom_height > 0.0 {
        elements.push(
            iced::widget::Space::new()
                .width(Length::Fill)
                .height(Length::Fixed(bottom_height))
                .into(),
        );
    }

    let list = Column::with_children(elements);

    container(column![
        header,
        scrollable(list)
            .height(Length::Fill)
            .style(scrollable_style)
            .on_scroll(Message::MessageListScrolled),
    ])
    .width(Length::Fixed(width))
    .height(Length::Fill)
    .style(message_list_style)
    .into()
}

/// Row item for virtual scrolling (either a message or thread header).
enum RowItem<'a> {
    Message(&'a MessageSummary, bool), // (message, indented)
    ThreadHeader(&'a Thread, bool),    // (thread, expanded)
}

/// Estimates row height based on list density.
const fn estimate_row_height(density: ListDensity) -> f32 {
    // Row height = 2*padding + content height (~3 lines of text)
    // Base content height ~48px, padding varies by density
    match density {
        ListDensity::Compact => 64.0,
        ListDensity::Comfortable => 80.0,
        ListDensity::Spacious => 96.0,
    }
}

/// Renders a thread header row (collapsed or expanded).
fn view_thread_header(
    thread: &Thread,
    is_expanded: bool,
    font_size: FontSize,
    list_density: ListDensity,
) -> Element<'static, Message> {
    let p = palette::current();
    let base = font_size.base_size();
    let snippet = font_size.snippet_size();
    let row_padding = list_density.row_padding();
    let spacing = list_density.spacing();

    // Expand/collapse icon
    let expand_icon = if is_expanded {
        "\u{25BC}" // down arrow
    } else {
        "\u{25B6}" // right arrow
    };

    let icon = text(expand_icon).size(10).style(move |_theme| text::Style {
        color: Some(p.text_muted),
    });

    // Thread participants
    let participants = text(thread.participants_display())
        .size(base)
        .font(iced::Font {
            weight: if thread.unread_count > 0 {
                iced::font::Weight::Semibold
            } else {
                iced::font::Weight::Normal
            },
            ..Default::default()
        })
        .style(move |_theme| text::Style {
            color: Some(p.text_primary),
        });

    // Message count badge
    let count_badge = container(
        text(format!("{}", thread.message_count()))
            .size(snippet)
            .style(move |_theme| text::Style {
                color: Some(p.text_on_primary),
            }),
    )
    .padding([2, 6])
    .style(move |_theme| container::Style {
        background: Some(Background::Color(p.accent_blue)),
        border: Border {
            radius: 10.0.into(),
            ..Default::default()
        },
        ..Default::default()
    });

    // Subject
    let subject_size = base - 1;
    let subject = text(truncate(&thread.subject, 40))
        .size(subject_size)
        .style(move |_theme| text::Style {
            color: Some(p.text_primary),
        });

    // Date
    let date = text(thread.latest_date.clone())
        .size(snippet)
        .style(move |_theme| text::Style {
            color: Some(p.text_muted),
        });

    // Unread indicator
    let mut indicators = row![].spacing(4);
    if thread.unread_count > 0 {
        indicators = indicators.push(container(text("")).width(8).height(8).style(move |_theme| {
            container::Style {
                background: Some(Background::Color(p.unread)),
                border: Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        }));
    }

    let spacer = iced::widget::Space::new().width(Length::Fill);

    let header_row = row![icon, participants, count_badge, indicators, spacer, date]
        .spacing(spacing)
        .align_y(iced::Alignment::Center);

    let content = column![header_row, subject].spacing(spacing / 2);

    let main_row = row![content]
        .padding([row_padding, 14])
        .align_y(iced::Alignment::Start);

    let btn = button(main_row)
        .width(Length::Fill)
        .padding(0)
        .style(message_button_style)
        .on_press(Message::ToggleThread(thread.id.clone()));

    container(container(btn).style(message_row_style))
        .style(message_row_border_style)
        .into()
}

/// Renders a single message row in the list with Air-inspired styling.
#[allow(clippy::too_many_lines)]
fn view_message_row(
    msg: &MessageSummary,
    selected: Option<MessageId>,
    indented: bool,
    font_size: FontSize,
    list_density: ListDensity,
) -> Element<'static, Message> {
    let is_selected = selected == Some(msg.id);
    let base = font_size.base_size();
    let snippet_size = font_size.snippet_size();
    let row_padding = list_density.row_padding();
    let spacing = list_density.spacing();
    let avatar_size = list_density.avatar_size();

    // Avatar (Air-style colored circle with initials)
    let avatar = view_avatar(&msg.from_name, avatar_size);

    // Sender name - bold if unread
    let from_weight = if msg.is_read {
        iced::font::Weight::Normal
    } else {
        iced::font::Weight::Semibold
    };

    let from = text(msg.from_name.clone())
        .size(base)
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
    let date = text(msg.date.clone()).size(snippet_size).style(|_theme| {
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

    let subject_text_size = base - 1;
    let subject = text(msg.subject.clone())
        .size(subject_text_size)
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

    // Snippet - Air uses secondary text color
    let snippet = text(truncate(&msg.snippet, 60))
        .size(snippet_size)
        .style(|_theme| {
            let p = palette::current();
            text::Style {
                color: Some(p.text_secondary),
            }
        });

    // Indicators row
    let mut indicators = row![].spacing(4);

    // Flag indicator (star)
    if msg.is_flagged {
        indicators = indicators.push(text("\u{2B50}").size(11).style(|_theme| {
            let p = palette::current();
            text::Style {
                color: Some(p.accent_yellow),
            }
        }));
    }

    // Attachment indicator
    if msg.has_attachments {
        indicators = indicators.push(text("\u{1F4CE}").size(11).style(|_theme| {
            let p = palette::current();
            text::Style {
                color: Some(p.text_muted),
            }
        }));
    }

    // Unread indicator (indigo dot) - Air style
    if !msg.is_read {
        indicators = indicators.push(container(text("")).width(8).height(8).style(|_theme| {
            let p = palette::current();
            container::Style {
                background: Some(Background::Color(p.unread)),
                border: Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        }));
    }

    // Spacer for date alignment
    let spacer = iced::widget::Space::new().width(Length::Fill);

    // Header: sender name, indicators, date
    let header_row = row![from, indicators, spacer, date]
        .spacing(spacing / 2)
        .align_y(iced::Alignment::Center);

    // Text content (right of avatar)
    let text_content = column![header_row, subject, snippet].spacing(spacing / 4);

    // Main row: avatar + text content
    // Add indentation for threaded messages
    let main_row = if indented {
        row![
            iced::widget::Space::new().width(Length::Fixed(20.0)),
            avatar,
            text_content
        ]
        .spacing(spacing)
        .padding([row_padding, 14])
        .align_y(iced::Alignment::Start)
    } else {
        row![avatar, text_content]
            .spacing(spacing)
            .padding([row_padding, 14])
            .align_y(iced::Alignment::Start)
    };

    // Row styling based on selection
    let row_style = if is_selected {
        message_row_selected_style
    } else {
        message_row_style
    };

    let btn = button(main_row)
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

/// Gets initials from a name (Air-style avatar).
fn get_initials(name: &str) -> String {
    let parts: Vec<&str> = name.split_whitespace().collect();
    match parts.len() {
        0 => "?".to_string(),
        1 => parts[0]
            .chars()
            .next()
            .unwrap_or('?')
            .to_uppercase()
            .to_string(),
        _ => {
            let first = parts[0].chars().next().unwrap_or('?');
            let last = parts[parts.len() - 1].chars().next().unwrap_or('?');
            format!("{}{}", first.to_uppercase(), last.to_uppercase())
        }
    }
}

/// Gets avatar color based on name hash (consistent color per sender).
fn get_avatar_color(name: &str) -> iced::Color {
    let p = palette::current();
    let colors = [
        p.avatar_purple,
        p.avatar_pink,
        p.avatar_cyan,
        p.avatar_green,
        p.avatar_orange,
    ];

    // Simple hash of name to pick color
    let hash: usize = name.bytes().map(|b| b as usize).sum();
    colors[hash % colors.len()]
}

/// Renders an Air-style avatar circle with initials.
fn view_avatar(name: &str, avatar_size: f32) -> Element<'static, Message> {
    let p = palette::current();
    let initials = get_initials(name);
    let color = get_avatar_color(name);

    // Scale text size proportionally to avatar
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let avatar_text_size = (avatar_size * 0.36) as u32; // ~13px for 36px avatar

    container(
        text(initials)
            .size(avatar_text_size)
            .font(iced::Font {
                weight: iced::font::Weight::Bold,
                ..Default::default()
            })
            .color(p.text_on_primary),
    )
    .width(Length::Fixed(avatar_size))
    .height(Length::Fixed(avatar_size))
    .align_x(iced::alignment::Horizontal::Center)
    .align_y(iced::alignment::Vertical::Center)
    .style(move |_theme| container::Style {
        background: Some(Background::Color(color)),
        border: Border {
            radius: (avatar_size / 2.0).into(), // Perfect circle
            ..Default::default()
        },
        ..Default::default()
    })
    .into()
}
