#![cfg_attr(not(feature = "std"), no_std)]

//! `glasuvai-crypto` — cryptographic primitives from first principles.
//!
//! This crate has **zero external dependencies**. Every algorithm is
//! implemented from first principles, traceable to its textbook definition.
//!
//! Primitives (added in M2):
//! - P-256 (secp256r1) field & curve arithmetic
//! - ElGamal encryption (homomorphic)
//! - Chaum-Pedersen ZKPs
//! - Pedersen DKG
//! - RSA blind signatures
//! - SHA-256
