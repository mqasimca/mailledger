//! Account setup view.
//!
//! Provides a form for configuring email accounts.

use iced::widget::{
    Space, button, column, container, pick_list, row, scrollable, text, text_input,
};
use iced::{Alignment, Element, Length};

use crate::message::{AccountSetupMessage, Message};
use crate::model::AccountSetupState;
use crate::style::widgets;
use crate::style::widgets::palette;

/// Render the account setup view.
pub fn view_account_setup(state: &AccountSetupState) -> Element<'_, Message> {
    let p = palette::current();

    let title = text("Account Setup").size(28).color(p.text_primary);

    let subtitle = text("Configure your email account settings")
        .size(14)
        .color(p.text_secondary);

    let basic_section = create_basic_section(state);
    let imap_section = create_imap_section(state);
    let smtp_section = create_smtp_section(state);
    let error_display = create_error_display(state);
    let buttons = create_action_buttons(state);

    // Main content
    let content = column![
        title,
        subtitle,
        Space::new().height(20),
        basic_section,
        imap_section,
        smtp_section,
        error_display,
        Space::new().height(20),
        buttons,
    ]
    .spacing(16)
    .padding(32)
    .max_width(600);

    container(scrollable(container(content).center_x(Length::Fill)))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| {
            let p = palette::current();
            container::Style {
                background: Some(iced::Background::Color(p.background)),
                ..Default::default()
            }
        })
        .into()
}

/// Create the basic information section.
fn create_basic_section(state: &AccountSetupState) -> Element<'_, Message> {
    create_section(
        "Basic Information",
        column![
            labeled_input(
                "Account Name",
                "My Email",
                &state.name,
                AccountSetupMessage::NameChanged,
                state.errors.get("name"),
            ),
            labeled_input(
                "Email Address",
                "user@example.com",
                &state.email,
                AccountSetupMessage::EmailChanged,
                state.errors.get("email"),
            ),
        ]
        .spacing(12),
    )
}

/// Create the IMAP server configuration section.
fn create_imap_section(state: &AccountSetupState) -> Element<'_, Message> {
    create_section(
        "Incoming Mail (IMAP)",
        column![
            create_server_port_row(
                &state.imap_host,
                &state.imap_port,
                "imap.example.com",
                "993",
                AccountSetupMessage::ImapHostChanged,
                AccountSetupMessage::ImapPortChanged,
            ),
            create_security_row(&state.imap_security, |s| {
                Message::AccountSetup(AccountSetupMessage::ImapSecurityChanged(parse_security(s)))
            }),
            labeled_input(
                "Username",
                "user@example.com",
                &state.imap_username,
                AccountSetupMessage::ImapUsernameChanged,
                state.errors.get("imap_username"),
            ),
            labeled_password(
                "Password",
                &state.imap_password,
                AccountSetupMessage::ImapPasswordChanged,
                state.errors.get("imap_password"),
            ),
        ]
        .spacing(12),
    )
}

/// Create the SMTP server configuration section.
fn create_smtp_section(state: &AccountSetupState) -> Element<'_, Message> {
    create_section(
        "Outgoing Mail (SMTP)",
        column![
            create_server_port_row(
                &state.smtp_host,
                &state.smtp_port,
                "smtp.example.com",
                "465",
                AccountSetupMessage::SmtpHostChanged,
                AccountSetupMessage::SmtpPortChanged,
            ),
            create_security_row(&state.smtp_security, |s| {
                Message::AccountSetup(AccountSetupMessage::SmtpSecurityChanged(parse_security(s)))
            }),
            labeled_input(
                "Username",
                "user@example.com",
                &state.smtp_username,
                AccountSetupMessage::SmtpUsernameChanged,
                state.errors.get("smtp_username"),
            ),
            labeled_password(
                "Password",
                &state.smtp_password,
                AccountSetupMessage::SmtpPasswordChanged,
                state.errors.get("smtp_password"),
            ),
        ]
        .spacing(12),
    )
}

/// Create the server and port input row.
fn create_server_port_row<'a>(
    host: &'a str,
    port: &'a str,
    host_placeholder: &'a str,
    port_placeholder: &'a str,
    on_host_change: impl Fn(String) -> AccountSetupMessage + 'a,
    on_port_change: impl Fn(String) -> AccountSetupMessage + 'a,
) -> Element<'a, Message> {
    let p = palette::current();
    row![
        column![
            text("Server").size(12).color(p.text_secondary),
            text_input(host_placeholder, host)
                .on_input(move |s| Message::AccountSetup(on_host_change(s)))
                .padding(10)
                .style(widgets::search_input_style),
        ]
        .spacing(4)
        .width(Length::FillPortion(3)),
        column![
            text("Port").size(12).color(p.text_secondary),
            text_input(port_placeholder, port)
                .on_input(move |s| Message::AccountSetup(on_port_change(s)))
                .padding(10)
                .style(widgets::search_input_style),
        ]
        .spacing(4)
        .width(Length::FillPortion(1)),
    ]
    .spacing(12)
    .into()
}

/// Create the security selection row.
fn create_security_row<'a>(
    security: &'a str,
    on_change: impl Fn(&str) -> Message + 'a,
) -> Element<'a, Message> {
    let p = palette::current();
    row![
        column![
            text("Security").size(12).color(p.text_secondary),
            pick_list(
                vec!["SSL/TLS", "STARTTLS", "None (insecure)"],
                Some(security_display(security)),
                on_change
            )
            .padding(10)
            .width(Length::Fill),
        ]
        .spacing(4)
        .width(Length::Fill),
    ]
    .into()
}

/// Create the error display element.
fn create_error_display(state: &AccountSetupState) -> Element<'_, Message> {
    let p = palette::current();
    let error_color = p.accent_red;
    state.save_error.as_ref().map_or_else(
        || Space::new().height(0).into(),
        move |error| {
            container(text(error).size(14).color(error_color))
                .padding(10)
                .into()
        },
    )
}

/// Create the action buttons row.
fn create_action_buttons(state: &AccountSetupState) -> Element<'_, Message> {
    row![
        button(text("Cancel").size(14))
            .on_press(Message::AccountSetup(AccountSetupMessage::Cancel))
            .padding([10, 20])
            .style(widgets::secondary_button_style),
        Space::new().width(Length::Fill),
        button(
            text(if state.is_testing {
                "Testing..."
            } else {
                "Test Connection"
            })
            .size(14)
        )
        .on_press_maybe(if state.is_testing {
            None
        } else {
            Some(Message::AccountSetup(AccountSetupMessage::TestConnection))
        })
        .padding([10, 20])
        .style(widgets::toolbar_button_style),
        button(text(if state.is_saving { "Saving..." } else { "Save" }).size(14))
            .on_press_maybe(if state.is_saving {
                None
            } else {
                Some(Message::AccountSetup(AccountSetupMessage::Save))
            })
            .padding([10, 20])
            .style(widgets::primary_button_style),
    ]
    .spacing(12)
    .align_y(Alignment::Center)
    .into()
}

/// Create a section with title and content.
fn create_section<'a>(
    title: &'a str,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    let p = palette::current();
    container(
        column![
            text(title).size(16).color(p.text_primary),
            Space::new().height(12),
            content.into(),
        ]
        .spacing(8),
    )
    .padding(20)
    .style(widgets::card_style)
    .into()
}

/// Create a labeled text input.
fn labeled_input<'a>(
    label: &'a str,
    placeholder: &'a str,
    value: &'a str,
    on_input: impl Fn(String) -> AccountSetupMessage + 'a,
    error: Option<&'a String>,
) -> Element<'a, Message> {
    let p = palette::current();
    let mut col = column![
        text(label).size(12).color(p.text_secondary),
        text_input(placeholder, value)
            .on_input(move |s| Message::AccountSetup(on_input(s)))
            .padding(10)
            .style(widgets::search_input_style),
    ]
    .spacing(4);

    if let Some(err) = error {
        col = col.push(text(err).size(11).color(p.accent_red));
    }

    col.into()
}

/// Create a labeled password input.
fn labeled_password<'a>(
    label: &'a str,
    value: &'a str,
    on_input: impl Fn(String) -> AccountSetupMessage + 'a,
    error: Option<&'a String>,
) -> Element<'a, Message> {
    let p = palette::current();
    let mut col = column![
        text(label).size(12).color(p.text_secondary),
        text_input("", value)
            .on_input(move |s| Message::AccountSetup(on_input(s)))
            .padding(10)
            .secure(true)
            .style(widgets::search_input_style),
    ]
    .spacing(4);

    if let Some(err) = error {
        col = col.push(text(err).size(11).color(p.accent_red));
    }

    col.into()
}

fn security_display(security: &str) -> &'static str {
    match security {
        "starttls" => "STARTTLS",
        "none" => "None (insecure)",
        _ => "SSL/TLS",
    }
}

fn parse_security(display: &str) -> String {
    match display {
        "STARTTLS" => "starttls".to_string(),
        "None (insecure)" => "none".to_string(),
        _ => "tls".to_string(),
    }
}
