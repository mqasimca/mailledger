//! Inline image state for message rendering.

use iced::widget::image;

/// Inline image loading state.
#[derive(Debug, Clone)]
pub enum InlineImageState {
    /// Image is still loading.
    Loading,
    /// Image successfully loaded.
    Ready(image::Handle),
    /// Image failed to load.
    Failed(String),
}

/// Inline image entry for a message.
#[derive(Debug, Clone)]
pub struct InlineImage {
    /// Source URL for the image.
    pub url: String,
    /// Current loading state.
    pub state: InlineImageState,
}
