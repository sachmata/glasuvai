//! `glasuvai-election` — election domain types and embedded ballot data.
//!
//! This crate provides:
//! - Core election types ([`Mir`], [`Party`], [`Candidate`], [`BallotSpec`], [`ElectionConfig`])
//! - Embedded TOML ballot data for the active election (feature-gated)
//! - SHA-256 integrity digest computed at build time
//! - Validation functions for data consistency
//!
//! # Feature Flags
//!
//! - `bg-na51-2024` (default) — 51st National Assembly, October 2024
//! - `serde-serialize` — adds `Serialize` derives for JSON export

pub mod election;
