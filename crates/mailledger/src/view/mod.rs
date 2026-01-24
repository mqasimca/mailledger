//! View components for the application.

mod account_setup;
mod compose;
mod header;
mod message_list;
mod message_view;
mod pane_divider;
mod screener;
mod settings;
mod sidebar;

pub use account_setup::view_account_setup;
pub use compose::view_compose;
pub use header::view_header;
pub use message_list::view_message_list;
pub use message_view::view_message_content;
pub use pane_divider::view_pane_divider;
pub use screener::{PendingSender, view_screener};
pub use settings::view_settings;
pub use sidebar::view_sidebar;
