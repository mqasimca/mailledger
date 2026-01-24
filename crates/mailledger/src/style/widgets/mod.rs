//! Polished widget styles with shadows, gradients, and rounded corners.

#![allow(dead_code)] // Utility functions for themeable components
#![allow(unused_imports)] // Re-exports for external theming use
#![allow(clippy::needless_update)] // Explicit struct updates for clarity

mod buttons;
mod containers;
mod inputs;
pub mod palette;
mod shadows;

// Re-export palette for external access
pub use palette::*;

// Re-export radius constants
pub use shadows::radius;

// Re-export shadow functions
pub use shadows::{
    large as shadow_large, medium as shadow_medium, none as shadow_none, small as shadow_small,
    subtle as shadow_subtle,
};

// Re-export container styles
pub use containers::{
    card_style, elevated_card_style, header_style, message_content_style, message_header_style,
    message_list_style, message_row_border_style, message_row_selected_style, message_row_style,
    selected_style, sidebar_style, toolbar_style,
};

// Re-export button styles
pub use buttons::{
    folder_button_selected_style, folder_button_style, ghost_button_style, message_button_style,
    primary_button_style, secondary_button_style, toolbar_button_style,
};

// Re-export input styles
pub use inputs::{scrollable_style, search_input_style};
