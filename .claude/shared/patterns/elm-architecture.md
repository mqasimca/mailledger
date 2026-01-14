# Elm Architecture (TEA) for iced

The Model-View-Update pattern used by iced.

## Core Components

```rust
// 1. MODEL - Application state
#[derive(Debug, Default)]
struct App {
    count: i32,
    loading: bool,
}

// 2. MESSAGE - All possible events
#[derive(Debug, Clone)]
enum Message {
    Increment,
    Decrement,
    Reset,
    DataLoaded(Result<Data, Error>),
}

// 3. UPDATE - State transitions
impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Increment => {
                self.count += 1;
                Task::none()
            }
            Message::Decrement => {
                self.count -= 1;
                Task::none()
            }
            Message::Reset => {
                self.count = 0;
                Task::none()
            }
            Message::DataLoaded(result) => {
                self.loading = false;
                // handle result...
                Task::none()
            }
        }
    }

    // 4. VIEW - Render state to widgets
    fn view(&self) -> Element<'_, Message> {
        column![
            text(format!("Count: {}", self.count)),
            row![
                button("-").on_press(Message::Decrement),
                button("Reset").on_press(Message::Reset),
                button("+").on_press(Message::Increment),
            ].spacing(10),
        ]
        .padding(20)
        .into()
    }
}
```

## Async Commands

```rust
fn update(&mut self, message: Message) -> Task<Message> {
    match message {
        Message::LoadData => {
            self.loading = true;
            Task::perform(
                load_data_async(),
                Message::DataLoaded,
            )
        }
        // ...
    }
}

async fn load_data_async() -> Result<Data, Error> {
    // fetch from network, database, etc.
}
```

## Message Design

```rust
// Group related messages
enum Message {
    // Navigation
    Navigate(Route),

    // Account operations
    Account(AccountMessage),

    // Mail operations
    Mail(MailMessage),
}

enum AccountMessage {
    Select(AccountId),
    Add,
    Remove(AccountId),
    Updated(Result<Account, Error>),
}
```

## Data Flow

```
User Action → Message → Update → New State → View → UI
                ↑                              │
                └──────────────────────────────┘
                         (next interaction)
```
