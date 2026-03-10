//! Compile-time SHA-256 integrity digest of embedded ballot data.
//!
//! The digest is computed by `build.rs` over all files in
//! `data/elections/` and set as the `GLASUVAI_DATA_SHA256` env var.
//! This module re-exports it and provides a convenience function.

use super::data::DATA_INTEGRITY_DIGEST;

/// Returns the SHA-256 hex digest of the embedded election data.
///
/// This value is computed at build time and is stable as long as the
/// data files remain unchanged. The verifier and bulletin board use
/// this to confirm that the election data matches the published digest.
pub fn data_digest() -> &'static str {
    DATA_INTEGRITY_DIGEST
}
