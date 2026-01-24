//! Compose message view.

use iced::widget::{
    Column, Space, button, column, container, row, scrollable, text, text_editor, text_input,
};
use iced::{Background, Border, Element, Length};

use crate::message::{ComposeMessage, FormattingStyle, Message};
use crate::model::{AutocompleteField, ComposeState};
use crate::style::widgets::{self, palette};

/// Renders the compose message view.
pub fn view_compose<'a>(
    state: &ComposeState,
    body_content: &'a text_editor::Content,
) -> Element<'a, Message> {
    let p = palette::current();
    let title = text("Compose Message").size(28).color(p.text_primary);

    // Address fields with autocomplete
    let to_row = view_address_field(state, AutocompleteField::To);
    let cc_row = view_address_field(state, AutocompleteField::Cc);
    let bcc_row = view_address_field(state, AutocompleteField::Bcc);

    // Subject field
    let subject_row = create_field_row("Subject:", &state.subject, "Enter subject", |s| {
        Message::Compose(ComposeMessage::SubjectChanged(s))
    });

    // Body section
    let body_row = view_body_section(body_content);

    // Status and buttons
    let status = view_status(state);
    let buttons = view_buttons(state);

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

/// Creates an address field (To, Cc, Bcc) with autocomplete support.
fn view_address_field(state: &ComposeState, field: AutocompleteField) -> Element<'static, Message> {
    let (label, value, placeholder, on_change): (_, _, _, fn(String) -> Message) = match field {
        AutocompleteField::To => ("To:", &state.to, "recipient@example.com", |s| {
            Message::Compose(ComposeMessage::ToChanged(s))
        }),
        AutocompleteField::Cc => ("Cc:", &state.cc, "cc@example.com", |s| {
            Message::Compose(ComposeMessage::CcChanged(s))
        }),
        AutocompleteField::Bcc => ("Bcc:", &state.bcc, "bcc@example.com", |s| {
            Message::Compose(ComposeMessage::BccChanged(s))
        }),
    };

    let suggestions = if state.active_autocomplete == Some(field) && !state.suggestions.is_empty() {
        Some(view_suggestions(
            &state.suggestions,
            state.selected_suggestion,
        ))
    } else {
        None
    };

    create_field_row_with_suggestions(label, value, placeholder, on_change, suggestions)
}

/// Creates the body editor section with toolbar.
fn view_body_section(body_content: &text_editor::Content) -> Element<'_, Message> {
    let p = palette::current();

    let body_label = text("Message:")
        .size(14)
        .color(p.text_secondary)
        .width(Length::Fixed(80.0));

    let toolbar = view_formatting_toolbar();

    let body_editor = text_editor(body_content)
        .placeholder("Write your message here...")
        .on_action(|action| Message::Compose(ComposeMessage::BodyAction(action)))
        .padding(12)
        .height(Length::Fixed(250.0));

    row![body_label, column![toolbar, body_editor].spacing(8)]
        .spacing(12)
        .align_y(iced::Alignment::Start)
        .into()
}

/// Creates the status/error message display.
fn view_status(state: &ComposeState) -> Element<'static, Message> {
    let p = palette::current();

    state.send_error.as_ref().map_or_else(
        || {
            if state.send_success {
                text("Message sent successfully!")
                    .size(14)
                    .color(p.accent_green)
                    .into()
            } else {
                Space::new().height(Length::Fixed(20.0)).into()
            }
        },
        |error| text(error.clone()).size(14).color(p.accent_red).into(),
    )
}

/// Creates the send/cancel buttons.
fn view_buttons(state: &ComposeState) -> Element<'static, Message> {
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

    row![send_btn, cancel_btn].spacing(12).into()
}

/// Creates a labeled input field row (without suggestions).
fn create_field_row(
    label: &str,
    value: &str,
    placeholder: &str,
    on_change: impl Fn(String) -> Message + 'static,
) -> Element<'static, Message> {
    create_field_row_with_suggestions(label, value, placeholder, on_change, None)
}

/// Creates a labeled input field row with optional suggestions dropdown.
fn create_field_row_with_suggestions(
    label: &str,
    value: &str,
    placeholder: &str,
    on_change: impl Fn(String) -> Message + 'static,
    suggestions: Option<Element<'static, Message>>,
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

    let input_with_suggestions: Element<'static, Message> = if let Some(dropdown) = suggestions {
        column![input, dropdown].spacing(0).into()
    } else {
        input.into()
    };

    row![label_text, input_with_suggestions]
        .spacing(12)
        .align_y(iced::Alignment::Start)
        .into()
}

/// Creates the autocomplete suggestions dropdown.
fn view_suggestions(
    suggestions: &[mailledger_core::Contact],
    selected: usize,
) -> Element<'static, Message> {
    let p = palette::current();

    let suggestion_buttons: Vec<Element<'static, Message>> = suggestions
        .iter()
        .enumerate()
        .take(5)
        .map(|(i, contact)| view_suggestion_button(i, contact, i == selected, p.text_muted))
        .collect();

    container(Column::with_children(suggestion_buttons))
        .width(Length::Fill)
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
        .into()
}

/// Creates a single suggestion button.
fn view_suggestion_button(
    index: usize,
    contact: &mailledger_core::Contact,
    is_selected: bool,
    muted_color: iced::Color,
) -> Element<'static, Message> {
    let primary_text = if contact.name.is_empty() {
        contact.email.clone()
    } else {
        contact.name.clone()
    };

    let secondary_text = if contact.name.is_empty() {
        String::new()
    } else {
        contact.email.clone()
    };

    button(
        column![
            text(primary_text).size(13),
            text(secondary_text).size(11).color(muted_color),
        ]
        .spacing(2),
    )
    .width(Length::Fill)
    .padding([8, 12])
    .style(move |_theme, status| {
        let p = palette::current();
        let bg = if is_selected {
            p.selected
        } else {
            match status {
                button::Status::Hovered | button::Status::Pressed => p.hover,
                _ => p.surface_elevated,
            }
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: p.text_primary,
            border: Border::default(),
            ..Default::default()
        }
    })
    .on_press(Message::Compose(ComposeMessage::SelectSuggestion(index)))
    .into()
}

/// Creates the markdown formatting toolbar.
fn view_formatting_toolbar() -> Element<'static, Message> {
    let p = palette::current();

    let bold_btn = button(text("B").size(14).font(iced::Font {
        weight: iced::font::Weight::Bold,
        ..Default::default()
    }))
    .padding([6, 12])
    .style(widgets::toolbar_button_style)
    .on_press(Message::Compose(ComposeMessage::InsertFormatting(
        FormattingStyle::Bold,
    )));

    let italic_btn = button(text("I").size(14).font(iced::Font {
        style: iced::font::Style::Italic,
        ..Default::default()
    }))
    .padding([6, 12])
    .style(widgets::toolbar_button_style)
    .on_press(Message::Compose(ComposeMessage::InsertFormatting(
        FormattingStyle::Italic,
    )));

    let link_btn = button(
        row![text("\u{1F517}").size(12), text("Link").size(12)]
            .spacing(4)
            .align_y(iced::Alignment::Center),
    )
    .padding([6, 12])
    .style(widgets::toolbar_button_style)
    .on_press(Message::Compose(ComposeMessage::InsertFormatting(
        FormattingStyle::Link,
    )));

    let hint = text("Markdown supported: **bold**, *italic*, [link](url)")
        .size(11)
        .color(p.text_muted);

    row![
        bold_btn,
        italic_btn,
        link_btn,
        Space::new().width(Length::Fill),
        hint
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center)
    .into()
}
