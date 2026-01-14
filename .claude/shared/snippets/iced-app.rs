//! iced application template.

use iced::{Element, Task};

fn main() -> iced::Result {
    iced::application("App Name", App::update, App::view)
        .run_with(App::new)
}

#[derive(Debug, Default)]
struct App {
    // State fields...
}

#[derive(Debug, Clone)]
enum Message {
    // Message variants...
}

impl App {
    fn new() -> (Self, Task<Message>) {
        (Self::default(), Task::none())
    }

    #[allow(clippy::needless_pass_by_value, clippy::unused_self)]
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // Handle messages...
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        use iced::widget::{column, text};

        column![
            text("Hello, iced!"),
        ]
        .into()
    }
}
