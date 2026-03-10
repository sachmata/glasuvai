//! Election domain module.
//!
//! Contains typed accessors for embedded ballot data (TOML),
//! validation functions, and a compile-time integrity digest.

pub mod data;
pub mod integrity;
pub mod types;
pub mod validate;
