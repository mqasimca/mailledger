//! Compose message view.

use iced::widget::{Space, button, column, container, row, scrollable, text, text_input};
use iced::{Element, Length};

use crate::message::{ComposeMessage, Message};
use crate::model::ComposeState;
use crate::style::widgets::{self, palette};

/// Renders the compose message view.
pub fn view_compose(state: &ComposeState) -> Element<'static, Message> {
    let p = palette::current();

    let title = text("Compose Message").size(28).color(p.text_primary);

    // To field
    let to_row = create_field_row("To:", &state.to, "recipient@example.com", |s| {
        Message::Compose(ComposeMessage::ToChanged(s))
    });

    // CC field
    let cc_row = create_field_row("Cc:", &state.cc, "cc@example.com", |s| {
        Message::Compose(ComposeMessage::CcChanged(s))
    });

    // BCC field
    let bcc_row = create_field_row("Bcc:", &state.bcc, "bcc@example.com", |s| {
        Message::Compose(ComposeMessage::BccChanged(s))
    });

    // Subject field
    let subject_row = create_field_row("Subject:", &state.subject, "Enter subject", |s| {
        Message::Compose(ComposeMessage::SubjectChanged(s))
    });

    // Body
    let body_label = text("Message:")
        .size(14)
        .color(p.text_secondary)
        .width(Length::Fixed(80.0));

    let body_input = text_input("Write your message here...", &state.body)
        .on_input(|s| Message::Compose(ComposeMessage::BodyChanged(s)))
        .padding(12)
        .size(14)
        .width(Length::Fill);

    let body_row = row![body_label, body_input]
        .spacing(12)
        .align_y(iced::Alignment::Start);

    // Status/error message
    let success_color = p.accent_green;
    let error_color = p.accent_red;
    let status: Element<'static, Message> = state.send_error.as_ref().map_or_else(
        move || {
            if state.send_success {
                text("Message sent successfully!")
                    .size(14)
                    .color(success_color)
                    .into()
            } else {
                Space::new().height(Length::Fixed(20.0)).into()
            }
        },
        move |error| text(error.clone()).size(14).color(error_color).into(),
    );

    // Buttons
    let send_btn = if state.is_sending {
        button(text("Sending...").size(14))
            .padding([10, 20])
            .style(widgets::primary_button_style)
    } else {
        button(text("Send").size(14))
            .padding([10, 20])
            .style(widgets::primary_button_style)
            .on_press(Message::Compose(ComposeMessage::Send))
    };

    let cancel_btn = button(text("Cancel").size(14))
        .padding([10, 20])
        .style(widgets::secondary_button_style)
        .on_press(Message::Compose(ComposeMessage::Cancel));

    let buttons = row![send_btn, cancel_btn].spacing(12);

    // Main content
    let content = column![
        title,
        Space::new().height(Length::Fixed(20.0)),
        to_row,
        cc_row,
        bcc_row,
        subject_row,
        Space::new().height(Length::Fixed(12.0)),
        body_row,
        Space::new().height(Length::Fixed(20.0)),
        status,
        Space::new().height(Length::Fixed(12.0)),
        buttons,
    ]
    .spacing(12)
    .padding(24)
    .width(Length::Fill);

    let scrollable_content = scrollable(content).height(Length::Fill);

    container(scrollable_content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme| {
            let p = palette::current();
            container::Style {
                background: Some(iced::Background::Color(p.surface)),
                ..Default::default()
            }
        })
        .into()
}

/// Creates a labeled input field row.
fn create_field_row(
    label: &str,
    value: &str,
    placeholder: &str,
    on_change: impl Fn(String) -> Message + 'static,
) -> Element<'static, Message> {
    let p = palette::current();
    let label_text = text(label.to_string())
        .size(14)
        .color(p.text_secondary)
        .width(Length::Fixed(80.0));

    let input = text_input(placeholder, value)
        .on_input(on_change)
        .padding(10)
        .size(14)
        .width(Length::Fill);

    row![label_text, input]
        .spacing(12)
        .align_y(iced::Alignment::Center)
        .into()
}
