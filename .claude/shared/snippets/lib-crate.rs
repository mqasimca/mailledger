//! # crate-name
//!
//! Brief description of what this crate does.
//!
//! ## Example
//!
//! ```ignore
//! use crate_name::Thing;
//!
//! let thing = Thing::new();
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

mod error;

pub use error::{Error, Result};

// Re-export important types
// pub use types::{...};
