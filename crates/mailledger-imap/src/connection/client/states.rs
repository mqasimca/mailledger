//! Type-state markers for IMAP client connection states.

/// Marker type for the not-authenticated state.
#[derive(Debug, Clone, Copy)]
pub struct NotAuthenticated;

/// Marker type for the authenticated state.
#[derive(Debug, Clone, Copy)]
pub struct Authenticated;

/// Marker type for the selected state.
#[derive(Debug, Clone, Copy)]
pub struct Selected;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::redundant_clone, clippy::manual_string_new, clippy::needless_collect, clippy::unreadable_literal, clippy::used_underscore_items, clippy::similar_names)]
mod tests {
    use super::*;

    fn _assert_send<T: Send>() {}
    fn _assert_sync<T: Sync>() {}

    #[test]
    fn test_state_markers_are_send_sync() {
        _assert_send::<NotAuthenticated>();
        _assert_sync::<NotAuthenticated>();
        _assert_send::<Authenticated>();
        _assert_sync::<Authenticated>();
        _assert_send::<Selected>();
        _assert_sync::<Selected>();
    }
}
