//! Core SMTP types.

mod address;
mod extension;
mod reply;

pub use address::{Address, Mailbox};
pub use extension::{AuthMechanism, Extension};
pub use reply::{Reply, ReplyCode};
