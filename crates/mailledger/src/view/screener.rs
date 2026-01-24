//! The Screener view - Gmail-inspired sender approval interface.
//!
//! Shows first-time senders in a clean list format with circular avatars
//! and intuitive action buttons for quick triage decisions.

use iced::widget::{Space, button, column, container, row, scrollable, text};
use iced::{Background, Border, Color, Element, Fill, Length};

use crate::message::{Message, ScreenerMessage, View};
use crate::style::widgets::{palette, radius};

/// A pending sender awaiting the user's decision.
#[derive(Debug, Clone)]
pub struct PendingSender {
    /// Email address (normalized to lowercase).
    pub email: String,
    /// Display name (if known).
    pub display_name: Option<String>,
    /// Number of emails received from this sender.
    pub email_count: u32,
    /// When first seen (for future display).
    #[allow(dead_code)]
    pub first_seen: Option<String>,
}

impl From<&mailledger_core::ScreenedSender> for PendingSender {
    fn from(sender: &mailledger_core::ScreenedSender) -> Self {
        Self {
            email: sender.email.clone(),
            display_name: sender.display_name.clone(),
            email_count: sender.email_count,
            first_seen: sender.first_seen.clone(),
        }
    }
}

/// Get initials from a name or email for the avatar.
fn get_initials(name: Option<&str>, email: &str) -> String {
    name.map_or_else(
        || {
            // Use first letter of email local part
            email
                .split('@')
                .next()
                .and_then(|local| local.chars().next())
                .unwrap_or('?')
                .to_uppercase()
                .to_string()
        },
        |name| {
            let parts: Vec<&str> = name.split_whitespace().collect();
            match parts.len() {
                0 => email
                    .chars()
                    .next()
                    .unwrap_or('?')
                    .to_uppercase()
                    .to_string(),
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
        },
    )
}

/// Generate a consistent color from a string (for avatar backgrounds).
#[allow(clippy::cast_precision_loss)]
fn string_to_color(s: &str) -> Color {
    let hash: u32 = s.bytes().fold(0u32, |acc, b| {
        acc.wrapping_add(u32::from(b)).wrapping_mul(31)
    });
    let hue = (hash % 360) as f32;
    // Convert HSL to RGB (saturation=65%, lightness=45% for vibrant but not harsh colors)
    hsl_to_rgb(hue, 0.65, 0.45)
}

/// Convert HSL to RGB color.
#[allow(
    clippy::many_single_char_names,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::suboptimal_flops
)]
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> Color {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r, g, b) = match (h as u32) / 60 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    Color::from_rgb(r + m, g + m, b + m)
}

/// Render the Screener view.
#[allow(clippy::too_many_lines)]
pub fn view_screener(pending_senders: &[PendingSender]) -> Element<'_, Message> {
    let p = palette::current();

    // Header bar
    let header = render_header(pending_senders.len());

    // Content area
    let content: Element<'_, Message> = if pending_senders.is_empty() {
        render_empty_state()
    } else {
        let sender_rows: Vec<Element<'_, Message>> =
            pending_senders.iter().map(render_sender_row).collect();

        container(scrollable(column(sender_rows).spacing(0).width(Fill)).height(Fill))
            .width(Fill)
            .height(Fill)
            .style(move |_| container::Style {
                background: Some(Background::Color(p.background)),
                ..Default::default()
            })
            .into()
    };

    column![header, content].width(Fill).height(Fill).into()
}

/// Render the header bar.
fn render_header(count: usize) -> Element<'static, Message> {
    let p = palette::current();

    let back_btn = button(
        row![text("\u{2190}").size(16), text("Back").size(14),]
            .spacing(6)
            .align_y(iced::Alignment::Center),
    )
    .style(move |_theme, status| {
        let bg = match status {
            iced::widget::button::Status::Hovered => p.hover,
            iced::widget::button::Status::Pressed => p.selected,
            _ => Color::TRANSPARENT,
        };
        iced::widget::button::Style {
            background: Some(Background::Color(bg)),
            text_color: p.text_primary,
            border: Border {
                radius: radius::MEDIUM.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    })
    .padding([8, 12])
    .on_press(Message::NavigateTo(View::Inbox));

    let title_text = match count {
        0 => "Screener".to_string(),
        1 => "Screener (1)".to_string(),
        n => format!("Screener ({n})"),
    };

    container(
        row![
            back_btn,
            Space::new().width(16),
            text(title_text)
                .size(20)
                .font(iced::Font {
                    weight: iced::font::Weight::Medium,
                    ..Default::default()
                })
                .color(p.text_primary),
        ]
        .align_y(iced::Alignment::Center),
    )
    .padding([12, 16])
    .width(Fill)
    .style(move |_| container::Style {
        background: Some(Background::Color(p.surface)),
        border: Border {
            color: p.border_subtle,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    })
    .into()
}

/// Render the empty state when no senders are pending.
fn render_empty_state() -> Element<'static, Message> {
    let p = palette::current();

    container(
        column![
            container(text("\u{1F4EC}").size(48)).padding(20),
            text("You're all caught up!")
                .size(22)
                .font(iced::Font {
                    weight: iced::font::Weight::Medium,
                    ..Default::default()
                })
                .color(p.text_primary),
            Space::new().height(8),
            text("No new senders waiting for review")
                .size(14)
                .color(p.text_secondary),
            Space::new().height(24),
            button(text("Go to Inbox").size(14).color(p.text_on_primary))
                .style(move |_theme, status| {
                    let bg = match status {
                        iced::widget::button::Status::Hovered => p.primary_light,
                        iced::widget::button::Status::Pressed => p.primary_dark,
                        _ => p.primary,
                    };
                    iced::widget::button::Style {
                        background: Some(Background::Color(bg)),
                        text_color: p.text_on_primary,
                        border: Border {
                            radius: radius::MEDIUM.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                })
                .padding([10, 24])
                .on_press(Message::NavigateTo(View::Inbox)),
        ]
        .align_x(iced::Alignment::Center)
        .spacing(4),
    )
    .width(Fill)
    .height(Fill)
    .align_x(iced::alignment::Horizontal::Center)
    .align_y(iced::alignment::Vertical::Center)
    .style(move |_| container::Style {
        background: Some(Background::Color(p.background)),
        ..Default::default()
    })
    .into()
}

/// Render the avatar with initials.
fn render_avatar(sender: &PendingSender) -> Element<'_, Message> {
    let initials = get_initials(sender.display_name.as_deref(), &sender.email);
    let avatar_color = string_to_color(&sender.email);

    container(
        text(initials)
            .size(14)
            .font(iced::Font {
                weight: iced::font::Weight::Medium,
                ..Default::default()
            })
            .color(Color::WHITE),
    )
    .width(Length::Fixed(40.0))
    .height(Length::Fixed(40.0))
    .align_x(iced::alignment::Horizontal::Center)
    .align_y(iced::alignment::Vertical::Center)
    .style(move |_| container::Style {
        background: Some(Background::Color(avatar_color)),
        border: Border {
            radius: 20.0.into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .into()
}

/// Render action buttons for a sender row.
#[allow(clippy::too_many_lines)]
fn render_action_buttons(email: String) -> Element<'static, Message> {
    let p = palette::current();

    let approve_btn = button(
        row![
            text("\u{2713}").size(14).color(p.accent_green),
            text("Allow").size(12).color(p.text_primary),
        ]
        .spacing(4)
        .align_y(iced::Alignment::Center),
    )
    .style(move |_theme, status| {
        let bg = match status {
            iced::widget::button::Status::Hovered => Color::from_rgba(0.2, 0.8, 0.4, 0.15),
            iced::widget::button::Status::Pressed => Color::from_rgba(0.2, 0.8, 0.4, 0.25),
            _ => Color::TRANSPARENT,
        };
        iced::widget::button::Style {
            background: Some(Background::Color(bg)),
            text_color: p.text_primary,
            border: Border {
                color: p.border_subtle,
                width: 1.0,
                radius: radius::SMALL.into(),
            },
            ..Default::default()
        }
    })
    .padding([6, 12])
    .on_press(Message::Screener(ScreenerMessage::ApproveToImbox(
        email.clone(),
    )));

    let feed_btn = button(text("Feed").size(12).color(p.text_secondary))
        .style(move |_theme, status| {
            let bg = match status {
                iced::widget::button::Status::Hovered => p.hover,
                iced::widget::button::Status::Pressed => p.selected,
                _ => Color::TRANSPARENT,
            };
            iced::widget::button::Style {
                background: Some(Background::Color(bg)),
                text_color: p.text_secondary,
                border: Border {
                    color: p.border_subtle,
                    width: 1.0,
                    radius: radius::SMALL.into(),
                },
                ..Default::default()
            }
        })
        .padding([6, 10])
        .on_press(Message::Screener(ScreenerMessage::ApproveToFeed(
            email.clone(),
        )));

    let paper_btn = button(text("Receipts").size(12).color(p.text_secondary))
        .style(move |_theme, status| {
            let bg = match status {
                iced::widget::button::Status::Hovered => p.hover,
                iced::widget::button::Status::Pressed => p.selected,
                _ => Color::TRANSPARENT,
            };
            iced::widget::button::Style {
                background: Some(Background::Color(bg)),
                text_color: p.text_secondary,
                border: Border {
                    color: p.border_subtle,
                    width: 1.0,
                    radius: radius::SMALL.into(),
                },
                ..Default::default()
            }
        })
        .padding([6, 10])
        .on_press(Message::Screener(ScreenerMessage::ApproveToPaperTrail(
            email.clone(),
        )));

    let block_btn = button(
        row![
            text("\u{2715}").size(12).color(p.accent_red),
            text("Block").size(12).color(p.accent_red),
        ]
        .spacing(4)
        .align_y(iced::Alignment::Center),
    )
    .style(move |_theme, status| {
        let bg = match status {
            iced::widget::button::Status::Hovered => Color::from_rgba(1.0, 0.3, 0.3, 0.15),
            iced::widget::button::Status::Pressed => Color::from_rgba(1.0, 0.3, 0.3, 0.25),
            _ => Color::TRANSPARENT,
        };
        iced::widget::button::Style {
            background: Some(Background::Color(bg)),
            text_color: p.accent_red,
            border: Border {
                color: p.accent_red.scale_alpha(0.5),
                width: 1.0,
                radius: radius::SMALL.into(),
            },
            ..Default::default()
        }
    })
    .padding([6, 12])
    .on_press(Message::Screener(ScreenerMessage::Block(email)));

    row![
        approve_btn,
        feed_btn,
        paper_btn,
        Space::new().width(8),
        block_btn,
    ]
    .spacing(6)
    .align_y(iced::Alignment::Center)
    .into()
}

/// Render a single sender row - Gmail-inspired list item.
fn render_sender_row(sender: &PendingSender) -> Element<'_, Message> {
    let p = palette::current();
    let email = sender.email.clone();

    // Avatar
    let avatar = render_avatar(sender);

    // Sender info
    let display_name = sender.display_name.as_deref().unwrap_or(&sender.email);

    let name_text = text(display_name)
        .size(14)
        .font(iced::Font {
            weight: iced::font::Weight::Medium,
            ..Default::default()
        })
        .color(p.text_primary);

    let email_line = if sender.display_name.is_some() {
        text(&sender.email).size(12).color(p.text_secondary)
    } else {
        text("").size(0)
    };

    let count_text = if sender.email_count == 1 {
        "1 message".to_string()
    } else {
        format!("{} messages", sender.email_count)
    };
    let meta = text(count_text).size(12).color(p.text_muted);

    let sender_info = column![name_text, email_line, meta].spacing(2).width(Fill);

    // Action buttons
    let actions = render_action_buttons(email);

    // Row container
    container(
        row![avatar, Space::new().width(12), sender_info, actions,]
            .align_y(iced::Alignment::Center)
            .padding([12, 16]),
    )
    .width(Fill)
    .style(move |_| container::Style {
        background: Some(Background::Color(p.surface)),
        border: Border {
            color: p.border_subtle,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    })
    .into()
}
