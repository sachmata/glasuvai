# Glasuvai — Project Overview

## Purpose
Experimentally verifiable online voting system for Bulgarian parliamentary elections.

**Status:** Research prototype — NOT audited or approved for production use.

## Core Properties
- **Ballot secrecy**: Homomorphic encryption (ElGamal on Ristretto255/P-256); votes never decrypted individually
- **E2E verifiability**: Public bulletin board, Benaloh challenges, receipt hashes, Merkle inclusion proofs
- **No single point of trust**: 5-of-9 threshold decryption across independent stakeholders
- **Coercion resistance**: Online re-voting + in-person paper ballot override
- **Identity unlinking**: RSA blind signatures separate identity from vote
- **Open source**: Reproducible Nix builds, runtime attestation

## Tech Stack
- **Rust** (1.85.0) — all backend/CLI components, crypto library
- **TypeScript + React** — web client
- **React Native** — mobile client
- **Rust → WASM** (via wasm-pack) — client-side crypto bridge
- **axum** — HTTP framework for Rust services
- **SQLite / rusqlite** — storage for bulletin board / voting server
- **Nix flakes** — reproducible builds, pinned to nixos-24.11

## Repository Structure
```
rust-toolchain.toml      Pins Rust 1.85.0 + wasm32 target
flake.nix                Nix flake
flake.lock               Pinned input hashes (always committed)
Cargo.lock               Pinned crate versions (always committed)

packages/
  crypto/                Rust #[no_std] core crypto library
  crypto-wasm/           Rust→WASM thin bridge
  identity-provider/     Authentication + blind signatures
  voting-server/         Ballot acceptance
  bulletin-board/        Append-only public ledger
  tally/                 Homomorphic aggregation + threshold decrypt
  verifier/              Independent election verification CLI
  trustee-tool/          Key ceremony + partial decryption CLI
  web-client/            TypeScript + React voter UI
  mobile-client/         React Native mobile voter UI
  admin/                 Election setup tools
  station-override/      Polling station in-person override module

nix/                     Additional Nix build definitions
docs/                    Protocol specification
test/                    End-to-end test suite
scripts/
  docker-dev.sh          Docker dev environment helper
```

## Elliptic Curve
P-256 (NIST secp256r1), implemented from first principles in Rust with ZERO external crypto crates.

## Identity Model
- **Path A**: eID / QES certificate (B-Trust, etc.)
- **Path B**: Offline 12-char identity code (72 bits entropy) distributed at municipal offices

## Blind Signature Protocol
RSA blind signatures separate identity from vote. IdP knows WHO voted but not HOW. Voting Server sees encrypted ballots but not WHO cast them.

## Ballot Encoding
P × (C+1) matrix of ElGamal ciphertexts (parties × candidates+1). One cell = E(1), all others = E(0). Enforced by disjunctive Chaum-Pedersen ZKP. Supports homomorphic tallying.

## Trustees
5-of-9 Pedersen threshold decryption across: ЦИК, 3 largest parties, 2 opposition parties, Bulgarian Academy of Sciences, civil society NGO, international auditor.
