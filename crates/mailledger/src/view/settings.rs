//! Settings view.

use iced::widget::{Space, button, column, container, row, scrollable, text, toggler};
use iced::{Element, Length};

use crate::message::{Message, SettingsMessage, View};
use crate::model::{SettingsSection, SettingsState};
use crate::style::widgets::palette::{self, ThemeMode};

/// Renders the settings view.
pub fn view_settings(
    state: &SettingsState,
    account: Option<&mailledger_core::Account>,
    theme_mode: ThemeMode,
) -> Element<'static, Message> {
    let p = palette::current();

    let title = text("Settings").size(28).color(p.text_primary);

    // Section tabs
    let tabs = row![
        section_tab("Account", SettingsSection::Account, state.selected_section),
        section_tab(
            "Appearance",
            SettingsSection::Appearance,
            state.selected_section
        ),
        section_tab("About", SettingsSection::About, state.selected_section),
    ]
    .spacing(4);

    // Content based on selected section
    let content: Element<'static, Message> = match state.selected_section {
        SettingsSection::Account => view_account_section(account),
        SettingsSection::Appearance => view_appearance_section(theme_mode),
        SettingsSection::About => view_about_section(),
    };

    // Back button
    let back_btn = button(text("Back to Inbox").size(14).color(p.text_primary))
        .padding([10, 20])
        .style(move |theme, status| {
            let p = palette::current();
            secondary_button_style_themed(&p, theme, status)
        })
        .on_press(Message::NavigateTo(View::Inbox));

    let layout = column![
        title,
        Space::new().height(Length::Fixed(16.0)),
        tabs,
        Space::new().height(Length::Fixed(20.0)),
        content,
        Space::new().height(Length::Fixed(20.0)),
        back_btn,
    ]
    .spacing(8)
    .padding(24)
    .width(Length::Fill);

    let scrollable_content = scrollable(layout).height(Length::Fill);

    container(scrollable_content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_theme| {
            let p = palette::current();
            container::Style {
                background: Some(iced::Background::Color(p.surface)),
                ..Default::default()
            }
        })
        .into()
}

/// Secondary button style that reads from current palette.
fn secondary_button_style_themed(
    p: &palette::Palette,
    _theme: &iced::Theme,
    status: iced::widget::button::Status,
) -> iced::widget::button::Style {
    use iced::widget::button;
    use iced::{Background, Border, Color};

    let base = button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: p.text_primary,
        border: Border {
            color: p.border_medium,
            width: 1.0,
            radius: 8.0.into(),
        },
        shadow: iced::Shadow::default(),
        snap: false,
    };

    match status {
        button::Status::Active | button::Status::Disabled => base,
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(p.hover)),
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(p.selected)),
            ..base
        },
    }
}

/// Primary button style that reads from current palette.
fn primary_button_style_themed(
    p: &palette::Palette,
    _theme: &iced::Theme,
    status: iced::widget::button::Status,
) -> iced::widget::button::Style {
    use iced::widget::button;
    use iced::{Background, Border};

    let base = button::Style {
        background: Some(Background::Color(p.primary)),
        text_color: p.text_on_primary,
        border: Border {
            color: p.primary_dark,
            width: 1.0,
            radius: 8.0.into(),
        },
        shadow: iced::Shadow::default(),
        snap: false,
    };

    match status {
        button::Status::Active => base,
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(p.primary_light)),
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(p.primary_dark)),
            ..base
        },
        button::Status::Disabled => button::Style {
            background: Some(Background::Color(p.text_muted)),
            text_color: p.surface,
            ..base
        },
    }
}

/// Creates a section tab button.
fn section_tab(
    label: &str,
    section: SettingsSection,
    current: SettingsSection,
) -> Element<'static, Message> {
    let is_active = section == current;
    let label_owned = label.to_string();

    button(text(label_owned).size(14))
        .padding([8, 16])
        .style(move |theme, status| {
            let p = palette::current();
            if is_active {
                primary_button_style_themed(&p, theme, status)
            } else {
                secondary_button_style_themed(&p, theme, status)
            }
        })
        .on_press(Message::Settings(SettingsMessage::SelectSection(section)))
        .into()
}

/// Account settings section.
fn view_account_section(account: Option<&mailledger_core::Account>) -> Element<'static, Message> {
    let p = palette::current();

    let account_info: Element<'static, Message> = account.map_or_else(
        || {
            let p = palette::current();
            column![
                text("No account configured")
                    .size(14)
                    .color(p.text_secondary),
                Space::new().height(Length::Fixed(12.0)),
                button(text("Add Account").size(14).color(p.text_on_primary))
                    .padding([10, 20])
                    .style(move |theme, status| {
                        let p = palette::current();
                        primary_button_style_themed(&p, theme, status)
                    })
                    .on_press(Message::NavigateTo(View::AccountSetup)),
            ]
            .spacing(8)
            .into()
        },
        |acc| {
            let p = palette::current();
            let email = acc.email.clone();
            let name = acc.name.clone();
            let imap = format!("{}:{}", acc.imap.host, acc.imap.port);
            let smtp = format!("{}:{}", acc.smtp.host, acc.smtp.port);

            column![
                settings_row("Email", &email),
                settings_row("Name", &name),
                settings_row("IMAP Server", &imap),
                settings_row("SMTP Server", &smtp),
                Space::new().height(Length::Fixed(16.0)),
                row![
                    button(text("Edit Account").size(14).color(p.text_on_primary))
                        .padding([10, 20])
                        .style(move |theme, status| {
                            let p = palette::current();
                            primary_button_style_themed(&p, theme, status)
                        })
                        .on_press(Message::NavigateTo(View::AccountSetup)),
                ]
                .spacing(12),
            ]
            .spacing(8)
            .into()
        },
    );

    column![
        text("Account").size(20).color(p.text_primary),
        Space::new().height(Length::Fixed(12.0)),
        account_info,
    ]
    .spacing(4)
    .into()
}

/// Appearance settings section with theme toggle.
fn view_appearance_section(theme_mode: ThemeMode) -> Element<'static, Message> {
    let p = palette::current();
    let is_dark = theme_mode == ThemeMode::Dark;

    let theme_label = if is_dark { "Dark Mode" } else { "Light Mode" };

    let theme_toggle = row![
        text("Theme")
            .size(14)
            .color(p.text_secondary)
            .width(Length::Fixed(120.0)),
        toggler(is_dark)
            .label(theme_label)
            .on_toggle(|_| Message::Settings(SettingsMessage::ToggleTheme))
            .text_size(14)
            .width(Length::Shrink),
    ]
    .spacing(16)
    .align_y(iced::Alignment::Center);

    let theme_description = text(if is_dark {
        "Using dark theme for reduced eye strain"
    } else {
        "Using light theme for bright environments"
    })
    .size(12)
    .color(p.text_muted);

    column![
        text("Appearance").size(20).color(p.text_primary),
        Space::new().height(Length::Fixed(16.0)),
        theme_toggle,
        Space::new().height(Length::Fixed(8.0)),
        theme_description,
        Space::new().height(Length::Fixed(24.0)),
        text("Display Options").size(16).color(p.text_primary),
        Space::new().height(Length::Fixed(12.0)),
        text("Font size: Medium (default)")
            .size(14)
            .color(p.text_secondary),
        Space::new().height(Length::Fixed(8.0)),
        text("Message density: Comfortable")
            .size(14)
            .color(p.text_secondary),
        Space::new().height(Length::Fixed(16.0)),
        text("More appearance options coming soon.")
            .size(12)
            .color(p.text_muted),
    ]
    .spacing(4)
    .into()
}

/// About section.
fn view_about_section() -> Element<'static, Message> {
    let p = palette::current();

    column![
        text("About MailLedger").size(20).color(p.text_primary),
        Space::new().height(Length::Fixed(12.0)),
        text("Version 0.1.0").size(14).color(p.text_secondary),
        Space::new().height(Length::Fixed(8.0)),
        text("A cross-platform desktop email client built with Rust.")
            .size(14)
            .color(p.text_secondary),
        Space::new().height(Length::Fixed(16.0)),
        text("Features:").size(14).color(p.text_primary),
        text("  - Custom IMAP implementation")
            .size(13)
            .color(p.text_secondary),
        text("  - Real-time push notifications (IDLE)")
            .size(13)
            .color(p.text_secondary),
        text("  - SMTP email sending")
            .size(13)
            .color(p.text_secondary),
        text("  - Secure connections via TLS")
            .size(13)
            .color(p.text_secondary),
        text("  - Light and Dark themes")
            .size(13)
            .color(p.text_secondary),
        Space::new().height(Length::Fixed(16.0)),
        text("Built with iced GUI framework")
            .size(12)
            .color(p.text_muted),
    ]
    .spacing(4)
    .into()
}

/// Creates a settings row with label and value.
fn settings_row(label: &str, value: &str) -> Element<'static, Message> {
    let p = palette::current();
    let label_owned = format!("{label}:");
    let value_owned = value.to_string();

    row![
        text(label_owned)
            .size(14)
            .color(p.text_secondary)
            .width(Length::Fixed(120.0)),
        text(value_owned).size(14).color(p.text_primary),
    ]
    .spacing(8)
    .into()
}
