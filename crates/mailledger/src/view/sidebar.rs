//! Sidebar view component (folder list) with Air-inspired styling.

use iced::widget::{Column, Space, button, column, container, row, scrollable, text};
use iced::{Background, Border, Element, Length};

use crate::message::{Message, View};
use crate::model::{Folder, FolderId, FolderType};
use crate::style::widgets::{
    folder_button_selected_style, folder_button_style, palette, primary_button_style,
    scrollable_style, sidebar_style,
};

/// Renders the sidebar with folder list and polished styling.
pub fn view_sidebar(
    folders: &[Folder],
    selected_folder: Option<FolderId>,
    pending_count: usize,
    accounts: &[mailledger_core::Account],
    active_account_id: Option<mailledger_core::AccountId>,
    account_switcher_open: bool,
    width: f32,
) -> Element<'static, Message> {
    let p = palette::current();

    // Account switcher at the top
    let account_switcher =
        view_account_switcher(accounts, active_account_id, account_switcher_open);

    // Screener button - prominent at the top
    let screener_btn = view_screener_button(pending_count);

    // Divider using a container with border
    let divider = container(Space::new().height(1))
        .width(Length::Fill)
        .padding([8, 16])
        .style(move |_theme| container::Style {
            border: Border {
                color: p.border_subtle,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        });

    // Section header
    let header = container(
        text("FOLDERS")
            .size(11)
            .font(iced::Font {
                weight: iced::font::Weight::Bold,
                ..Default::default()
            })
            .style(|_theme| {
                let p = palette::current();
                text::Style {
                    color: Some(p.text_muted),
                }
            }),
    )
    .padding(12);

    let folder_buttons: Vec<Element<'static, Message>> = folders
        .iter()
        .map(|folder| view_folder_item(folder, selected_folder))
        .collect();

    let folder_list = Column::with_children(folder_buttons).spacing(2).padding(8);

    let content = column![
        account_switcher,
        screener_btn,
        divider,
        header,
        scrollable(folder_list)
            .height(Length::Fill)
            .style(scrollable_style),
    ]
    .spacing(0);

    container(content)
        .width(Length::Fixed(width))
        .height(Length::Fill)
        .style(sidebar_style)
        .into()
}

/// Renders the account switcher dropdown.
#[allow(clippy::too_many_lines)]
fn view_account_switcher(
    accounts: &[mailledger_core::Account],
    active_account_id: Option<mailledger_core::AccountId>,
    is_open: bool,
) -> Element<'static, Message> {
    let p = palette::current();

    // Find the active account
    let active_account = accounts
        .iter()
        .find(|a| a.id == active_account_id)
        .or_else(|| accounts.first());

    let account_name = active_account.map_or_else(|| "No Account".to_string(), |a| a.name.clone());
    let account_email = active_account.map_or_else(String::new, |a| a.email.clone());

    // Get first letter for avatar
    let avatar_letter = account_name
        .chars()
        .next()
        .unwrap_or('?')
        .to_ascii_uppercase();

    let avatar = container(
        text(avatar_letter.to_string())
            .size(14)
            .font(iced::Font {
                weight: iced::font::Weight::Bold,
                ..Default::default()
            })
            .color(p.text_on_primary),
    )
    .width(Length::Fixed(32.0))
    .height(Length::Fixed(32.0))
    .center_x(Length::Fixed(32.0))
    .center_y(Length::Fixed(32.0))
    .style(move |_theme| container::Style {
        background: Some(Background::Color(p.primary)),
        border: Border {
            radius: 16.0.into(),
            ..Default::default()
        },
        ..Default::default()
    });

    let account_info = column![
        text(account_name)
            .size(13)
            .font(iced::Font {
                weight: iced::font::Weight::Medium,
                ..Default::default()
            })
            .style(move |_theme| text::Style {
                color: Some(p.text_primary),
            }),
        text(account_email)
            .size(11)
            .style(move |_theme| text::Style {
                color: Some(p.text_muted),
            }),
    ]
    .spacing(2);

    let dropdown_icon = text(if is_open { "\u{25B2}" } else { "\u{25BC}" })
        .size(10)
        .style(move |_theme| text::Style {
            color: Some(p.text_muted),
        });

    let header_content = row![avatar, account_info.width(Length::Fill), dropdown_icon,]
        .spacing(10)
        .align_y(iced::Alignment::Center);

    let header_btn = button(header_content)
        .width(Length::Fill)
        .padding(10)
        .style(folder_button_style)
        .on_press(Message::ToggleAccountSwitcher);

    let mut content: Column<'static, Message> = column![container(header_btn).padding(8)];

    // Show dropdown if open
    if is_open && accounts.len() > 1 {
        let account_buttons: Vec<Element<'static, Message>> = accounts
            .iter()
            .filter(|a| a.id != active_account_id) // Don't show currently active
            .filter_map(|account| {
                account.id.map(|account_id| {
                    let name = account.name.clone();
                    let email = account.email.clone();
                    let letter = name.chars().next().unwrap_or('?').to_ascii_uppercase();

                    let small_avatar = container(
                        text(letter.to_string())
                            .size(12)
                            .font(iced::Font {
                                weight: iced::font::Weight::Bold,
                                ..Default::default()
                            })
                            .color(p.text_on_primary),
                    )
                    .width(Length::Fixed(24.0))
                    .height(Length::Fixed(24.0))
                    .center_x(Length::Fixed(24.0))
                    .center_y(Length::Fixed(24.0))
                    .style(move |_theme| container::Style {
                        background: Some(Background::Color(p.accent_purple)),
                        border: Border {
                            radius: 12.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    });

                    let info = column![
                        text(name).size(12).style(move |_theme| text::Style {
                            color: Some(p.text_primary),
                        }),
                        text(email).size(10).style(move |_theme| text::Style {
                            color: Some(p.text_muted),
                        }),
                    ]
                    .spacing(1);

                    let row_content = row![small_avatar, info]
                        .spacing(8)
                        .align_y(iced::Alignment::Center);

                    button(row_content)
                        .width(Length::Fill)
                        .padding(8)
                        .style(folder_button_style)
                        .on_press(Message::SwitchAccount(account_id))
                        .into()
                })
            })
            .collect();

        if !account_buttons.is_empty() {
            let dropdown = Column::with_children(account_buttons).spacing(2);
            content = content.push(container(dropdown).padding([0, 8]));
        }

        // Add account button
        let add_btn = button(
            row![
                text("+").size(14).style(move |_theme| text::Style {
                    color: Some(p.primary),
                }),
                text("Add Account")
                    .size(12)
                    .style(move |_theme| text::Style {
                        color: Some(p.primary),
                    }),
            ]
            .spacing(6)
            .align_y(iced::Alignment::Center),
        )
        .width(Length::Fill)
        .padding(8)
        .style(folder_button_style)
        .on_press(Message::AddAccount);

        content = content.push(container(add_btn).padding(8));
    }

    content.into()
}

/// Renders the Screener button with Air-style compose button design.
/// Uses primary color with glow effect for prominence.
fn view_screener_button(pending_count: usize) -> Element<'static, Message> {
    let p = palette::current();

    let icon = text("\u{1F6E1}") // shield emoji
        .size(18);

    let label = text("The Screener")
        .size(14)
        .font(iced::Font {
            weight: iced::font::Weight::Semibold,
            ..Default::default()
        })
        .color(p.text_on_primary); // White text on primary button

    let mut content = row![icon, label]
        .spacing(10)
        .align_y(iced::Alignment::Center);

    // Badge for pending count - Air uses amber for pending/snooze
    if pending_count > 0 {
        let badge = container(
            text(pending_count.to_string())
                .size(11)
                .font(iced::Font {
                    weight: iced::font::Weight::Bold,
                    ..Default::default()
                })
                .color(p.background), // Dark text on bright badge
        )
        .padding([2, 6])
        .style(move |_theme| container::Style {
            background: Some(Background::Color(p.accent_yellow)),
            border: Border {
                radius: 10.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });

        content = content.push(Space::new().width(Length::Fill));
        content = content.push(badge);
    }

    // Use Air-style primary button with glow
    container(
        button(content.width(Length::Fill))
            .width(Length::Fill)
            .padding(12)
            .style(primary_button_style)
            .on_press(Message::NavigateTo(View::Screener)),
    )
    .padding(12)
    .into()
}

/// Renders a single folder item with icon and polished styling.
fn view_folder_item(folder: &Folder, selected: Option<FolderId>) -> Element<'static, Message> {
    let is_selected = selected == Some(folder.id);

    let icon = match folder.folder_type {
        FolderType::Inbox => "\u{1F4E5}",   // inbox tray
        FolderType::Sent => "\u{1F4E4}",    // outbox tray
        FolderType::Drafts => "\u{1F4DD}",  // memo
        FolderType::Trash => "\u{1F5D1}",   // wastebasket
        FolderType::Archive => "\u{1F4C1}", // folder
        FolderType::Spam => "\u{26A0}",     // warning
        FolderType::Normal => "\u{1F4C2}",  // open folder
    };

    // Text weight based on unread count
    let name_weight = if folder.unread_count > 0 {
        iced::font::Weight::Semibold
    } else {
        iced::font::Weight::Normal
    };

    let icon_text = text(icon).size(16);

    let folder_name = text(folder.name.clone())
        .size(14)
        .font(iced::Font {
            weight: name_weight,
            ..Default::default()
        })
        .style(move |_theme| {
            let p = palette::current();
            let text_color = if is_selected {
                p.primary
            } else {
                p.text_primary
            };
            text::Style {
                color: Some(text_color),
            }
        });

    let mut content = row![icon_text, folder_name]
        .spacing(10)
        .align_y(iced::Alignment::Center);

    // Unread badge
    if folder.unread_count > 0 {
        let badge = container(
            text(folder.unread_count.to_string())
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
        .padding(2)
        .style(|_theme| {
            let p = palette::current();
            container::Style {
                background: Some(iced::Background::Color(p.primary)),
                border: iced::Border {
                    radius: 10.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        });

        content = content.push(iced::widget::Space::new().width(Length::Fill));
        content = content.push(badge);
    }

    let btn_style = if is_selected {
        folder_button_selected_style
    } else {
        folder_button_style
    };

    button(content.width(Length::Fill))
        .width(Length::Fill)
        .padding(10)
        .style(btn_style)
        .on_press(Message::SelectFolder(folder.id))
        .into()
}
