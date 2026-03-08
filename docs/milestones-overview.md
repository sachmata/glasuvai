# Glasuvai — Milestone Implementation Plans Overview

## Demo Target

A fully functional demonstration of verifiable online voting using **real ballot data from the latest Bulgarian National Assembly elections** (51st National Assembly, October 27, 2024). The demo proves the complete cryptographic pipeline — from ballot encryption through homomorphic tallying to threshold decryption — is correct and independently verifiable.

## Core Constraints

| Constraint | Rule |
|---|---|
| **No fancy libraries** | Crypto crate (`packages/crypto`): `#[no_std]`-compatible, ZERO external crates. P-256 curve operations, ElGamal, ZKPs, DKG, blind signatures — all from first principles using only `core`/`alloc`/`std`. |
| **No fancy libraries** | Server plumbing: minimal well-audited crates for non-crypto infrastructure — `axum` (HTTP), `serde` (JSON), `rusqlite` (SQLite), `tokio` (async runtime). |
| **One implementation** | The same Rust crypto crate compiles to both native (servers, CLI tools) and WASM (browser client). No duplicate implementations, no cross-language test vectors needed. |
| **Verifiable without doubt** | Every cryptographic operation must be traceable to textbook definitions. Code reads like a math paper. `cargo test` covers all crypto for both server and client. |
| **Real ballot data** | Party lists, candidate names, MIR structure match the actual 51st National Assembly elections. Demo runs a simulated election in MIR 23-Sofia with real parties and candidates. |
| **Elliptic curve** | P-256 (NIST secp256r1) — implemented from first principles in Rust. Both server and client use the same code. |

## Milestone Map

```
M1: Foundation ──────┐
                     ├──> M2: Crypto ──────┐
                     │    Primitives        │
                     │                      ├──> M3: Ballot ──────┐
                     │                      │    Encoding         │
                     │                      │                     │
                     │                      ├──> M4: Bulletin ────┤
                     │                      │    Board            │
                     │                      │                     │
                     │                      ├──> M5: Identity &  ─┤
                     │                      │    Voting Server    │
                     │                      │                     │
                     │                      └──> M6: Tally & ────┤
                     │                           Decryption      │
                     │                                            │
                     │                      M7: Verifier ─────────┤
                     │                      (depends on M4+M6)    │
                     │                                            │
                     └────────────────────> M8: Web Client ───────┘
                                           & E2E Demo
                                           (depends on all)
```

## Milestones Summary

| # | Name | Packages | Est. Effort | Key Deliverable |
|---|---|---|---|---|
| M1 | Foundation & Election Data | `/packages/crypto` | 1 week | Rust workspace, Bulgarian election types, real 51st NA ballot data, test infrastructure |
| M2 | Cryptographic Primitives | `/packages/crypto` | 2–3 weeks | ElGamal, Chaum-Pedersen ZKPs, Pedersen DKG, RSA blind signatures — all from first principles, `#[no_std]`-compatible, zero external crates |
| M3 | Ballot Encoding & Encryption | `/packages/crypto` | 1–2 weeks | Ballot matrix encryption, disjunctive ZKP (exactly-one-of-N), Benaloh challenge, receipt hash |
| M4 | Bulletin Board | `/packages/bulletin-board` | 1–2 weeks | Append-only hash chain, Merkle tree, REST API (axum), mirror sync, SQLite storage (rusqlite) |
| M5 | Identity Provider & Voting Server | `/packages/identity-provider`, `/packages/voting-server` | 2 weeks | Blind signature token flow (axum), ballot acceptance, ZKP validation, re-vote handling |
| M6 | Tally & Threshold Decryption | `/packages/tally` | 1–2 weeks | Homomorphic aggregation, threshold decryption ceremony, Hare-Niemeyer seat allocation, 7% preference threshold |
| M7 | Independent Verifier | `/packages/verifier` | 1 week | CLI tool that verifies an entire election: ZKPs, chain, tally, decryption proofs |
| M8 | Web Client & E2E Demo | `/packages/web-client`, `/packages/crypto-wasm`, `/packages/trustee-tool` | 2 weeks | React UI with real Bulgarian ballots, Rust→WASM crypto from shared crate (~200-500 KB), full demo script from vote to verified result |

**Total estimated: 10–14 weeks**

## Demo Scenario

The demo simulates a complete election for **MIR 23 — София (Sofia)** using real party/candidate data from the 51st National Assembly elections:

1. **Setup**: 5-of-9 threshold key ceremony generates election keypair
2. **Identity**: 50 simulated voters authenticate and receive blind-signed tokens
3. **Voting**: Voters cast encrypted ballots choosing from real parties/candidates
4. **Re-vote**: Some voters re-vote (demonstrating last-vote-counts)
5. **Bulletin Board**: All ballots visible on public hash-chained ledger
6. **Tally**: Homomorphic aggregation + threshold decryption by 5 trustees
7. **Results**: Hare-Niemeyer seat allocation with preference threshold applied
8. **Verification**: Independent verifier confirms entire election is correct

Every step produces cryptographic proof. Anyone can run the verifier and confirm the result.

## Technology Decisions for Demo

| Component | Technology | Rationale |
|---|---|---|
| Crypto library | Rust `#[no_std]` crate — P-256 from first principles | One implementation shared by server and client. Zero external crates on crypto path. Every operation traceable to textbook. Compiles to both native and WASM. |
| WASM bridge | `packages/crypto-wasm` — thin `wasm-bindgen` wrapper | ~200-500 KB `.wasm`. Only dep is `wasm-bindgen` (JS interop glue, not crypto). |
| Blind signatures | RSA from first principles in `packages/crypto` | No external RSA crate. Uses Rust's big-integer arithmetic. |
| Hash function | SHA-256 (Rust `std` or hand-implemented for `#[no_std]`) | Standard, ubiquitous |
| Server framework | Rust + `axum` | Lightweight, well-audited async HTTP. Comparable to Go `net/http` in simplicity. |
| Storage | SQLite via `rusqlite` | Embedded, zero-config, sufficient for demo |
| Web client | TypeScript + React (Vite) | Thin UI shell; all crypto in Rust→WASM |
| Build | Cargo + `wasm-pack` + `make` | Simple, reproducible |

## File Naming Convention

Each milestone document: `docs/milestone-NN-short-name.md`

Each document contains:
- **Goal**: What this milestone achieves
- **Prerequisites**: Which milestones must be complete
- **Deliverables**: Exact files and packages produced
- **Data Structures**: Go type definitions
- **Implementation Steps**: Ordered, specific tasks
- **Test Vectors**: Expected inputs/outputs for verification
- **Acceptance Criteria**: How to verify the milestone is complete
