//! Sidebar view component (folder list) with polished styling.

use iced::widget::{Column, button, column, container, row, scrollable, text};
use iced::{Element, Length};

use crate::message::Message;
use crate::model::{Folder, FolderId, FolderType};
use crate::style::widgets::{
    folder_button_selected_style, folder_button_style, palette, scrollable_style, sidebar_style,
};

/// Renders the sidebar with folder list and polished styling.
pub fn view_sidebar(
    folders: &[Folder],
    selected_folder: Option<FolderId>,
) -> Element<'static, Message> {
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
    .padding([12, 16]);

    let folder_buttons: Vec<Element<'static, Message>> = folders
        .iter()
        .map(|folder| view_folder_item(folder, selected_folder))
        .collect();

    let folder_list = Column::with_children(folder_buttons)
        .spacing(2)
        .padding([0, 8]);

    let content = column![
        header,
        scrollable(folder_list)
            .height(Length::Fill)
            .style(scrollable_style),
    ]
    .spacing(0);

    container(content)
        .width(Length::Fixed(220.0))
        .height(Length::Fill)
        .style(sidebar_style)
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
        .padding([2, 6])
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
        .padding([10, 12])
        .style(btn_style)
        .on_press(Message::SelectFolder(folder.id))
        .into()
}
