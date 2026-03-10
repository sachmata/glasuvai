## 8. SYSTEM COMPONENTS & TECH STACK

### 8.1 Component Map

```
┌─────────────────────────────────────────────────────────┐
│                    glasuvai MONOREPO                       │
│                                                           │
│  /data                                                    │
│  └── /elections                                           │
│      └── /bg-na51-2024    Real ballot data (TOML files)   │
│          ├── election.toml     Election config             │
│          ├── mirs.toml         32 MIRs + seat counts       │
│          ├── parties.toml      Registered parties          │
│          └── candidates/       Per-MIR candidate lists     │
│                                                           │
│  /packages                                                │
│  ├── /crypto          (Rust, #[no_std]-compatible)        │
│  │   ├── src/point.rs         P-256 point operations      │
│  │   ├── src/scalar.rs        Scalar arithmetic mod N     │
│  │   ├── src/field.rs         Prime field Fp arithmetic   │
│  │   ├── src/elgamal.rs       Exponential ElGamal         │
│  │   ├── src/zkp.rs           Chaum-Pedersen proofs       │
│  │   ├── src/threshold.rs     Pedersen DKG + threshold    │
│  │   ├── src/blind.rs         RSA blind signatures        │
│  │   ├── src/ballot/          Ballot encode/encrypt       │
│  │   └── Cargo.toml           ZERO external crates        │
│  │   Pure crypto primitives only. Compiles to both native │
│  │   and WASM. One codebase for all crypto — server and   │
│  │   client use identical code.                           │
│  │                                                        │
│  ├── /election        (Rust) Election domain types + data │
│  │   ├── build.rs             SHA-256 digest of data/     │
│  │   ├── src/election/                                    │
│  │   │   ├── types.rs         Mir, Party, Candidate,      │
│  │   │   │                    BallotSpec, ElectionConfig   │
│  │   │   ├── data.rs          Embed TOML via include_str! │
│  │   │   ├── validate.rs      Seat totals, ballot checks  │
│  │   │   └── integrity.rs     DATA_INTEGRITY_DIGEST       │
│  │   └── Cargo.toml           depends on crypto + serde   │
│  │                              + toml                    │
│  │   Election domain types (Mir, Party, Candidate, etc.)  │
│  │   and real ballot data from TOML files under data/.    │
│  │   Feature flags select which election is embedded      │
│  │   (default: bg-na51-2024). Build script computes       │
│  │   SHA-256 integrity digest over all data files.        │
│  │                                                        │
│  ├── /crypto-wasm     (Rust→WASM) Thin WASM wrapper       │
│  │   ├── src/lib.rs           wasm-bindgen JS glue        │
│  │   └── Cargo.toml           depends on crypto + wasm-   │
│  │   Output: ~200-500 KB .wasm                            │
│  │   Only dep: wasm-bindgen (JS interop, not crypto)      │
│  │                                                        │
│  ├── /identity-provider (Rust) Authentication service     │
│  │   ├── src/auth.rs          Code + cert authentication  │
│  │   ├── src/token.rs         Blind signature issuance    │
│  │   ├── src/voter.rs         ГРАО voter roll lookup      │
│  │   └── src/api.rs           HTTP handlers (axum)        │
│  │                                                        │
│  ├── /voting-server   (Rust)  Ballot acceptance service   │
│  │   ├── src/submit.rs        Accept + validate ballots   │
│  │   ├── src/revote.rs        Handle re-vote replacement  │
│  │   └── src/publish.rs       Push to bulletin board      │
│  │                                                        │
│  ├── /bulletin-board  (Rust)  Append-only public ledger   │
│  │   ├── src/chain.rs         Hash-chain management       │
│  │   ├── src/merkle.rs        Merkle tree                 │
│  │   ├── src/api.rs           Public read API (axum)      │
│  │   └── src/store.rs         Storage (rusqlite)          │
│  │                                                        │
│  ├── /tally           (Rust)  Tallying CLI tool            │
│  │   ├── src/aggregate.rs     Homomorphic aggregation     │
│  │   ├── src/decrypt.rs       Threshold decryption        │
│  │   ├── src/hare.rs          Hare-Niemeyer allocation    │
│  │   ├── src/preference.rs    Preference threshold check  │
│  │   └── src/report.rs        Results report generation   │
│  │                                                        │
│  ├── /verifier        (Rust)  Independent verifier tool   │
│  │   ├── src/checks/          All 10 verification checks  │
│  │   ├── src/pipeline.rs      Verification orchestrator   │
│  │   └── src/main.rs          CLI: "verify full election" │
│  │                                                        │
│  ├── /trustee-tool    (Rust)  Key ceremony + decryption   │
│  │   ├── src/keygen.rs        DKG participation           │
│  │   ├── src/decrypt.rs       Partial decryption          │
│  │   └── src/main.rs          CLI for trustees            │
│  │                                                        │
│  ├── /web-client      (TypeScript + React)                │
│  │   ├── /components                                      │
│  │   │   ├── BallotForm.tsx   Voter UI                    │
│  │   │   ├── Verify.tsx       Receipt verification        │
│  │   │   ├── Challenge.tsx    Benaloh challenge UI        │
│  │   │   └── BuildHash.tsx    Build attestation display   │
│  │   ├── /crypto                                          │
│  │   │   └── wasm_bridge.ts   Bridge to Rust WASM          │
│  │   └── /auth                                            │
│  │       ├── cert.ts          QES authentication          │
│  │       └── code.ts          Offline code auth           │
│  │                                                        │
│  ├── /mobile-client   (TypeScript + React Native)         │
│  │   └── (mirrors web-client structure)                   │
│  │                                                        │
│  ├── /admin           (Rust)  Election setup tools        │
│  │   ├── src/election_setup.rs Configure election params  │
│  │   ├── src/voter_roll.rs     Import voter roll          │
│  │   └── src/code_gen.rs       Generate identity codes    │
│  │                                                        │
│  ├── /station-override (Rust)  Polling station override   │
│  │   ├── src/auth/            Commission member auth      │
│  │   ├── src/query/           IdP query: "has EGN voted?" │
│  │   ├── src/override/        Override msg to Voting Srv  │
│  │   ├── src/ui/              Touch-screen UI (egui/iced) │
│  │   └── src/audit/           Local tamper-evident log    │
│  │   Runs on machine voting devices at ~12,000 stations.  │
│  │   Separate process from machine voting software.       │
│  │   Queries IdP by EGN, receives token_hash, sends       │
│  │   override to Voting Server (never sends EGN to VS).   │
│  │                                                        │
│  └── /code-kiosk      (Rust)  Municipal code distribution │
│      ├── src/generate/        CSPRNG code gen + hashing   │
│      ├── src/clerk/           Issuance workflow + session  │
│      ├── src/idp_client/      IdP eligibility + hash reg  │
│      ├── src/output/          Voter-facing output sinks   │
│      │   (stdout, serial, file/pipe)                      │
│      └── src/audit/           Hash-chained audit log      │
│                                                           │
│  /nix                 Reproducible build definitions       │
│  /docs                Protocol specification               │
│  /test                End-to-end test suite                │
└─────────────────────────────────────────────────────────┘
```

### 8.2 Technology Choices

| Component | Technology | Rationale |
|---|---|---|
| Crypto library | Rust `#[no_std]` crate, P-256 from first principles | One implementation for both server and client. Zero external crypto crates. Every operation traceable to textbook definitions. |
| Elliptic curve | P-256 (NIST secp256r1) | Well-studied, government-approved standard. Implemented from first principles in Rust. |
| Browser crypto | Rust→WASM (via `wasm-bindgen`) | ~200-500 KB binary. Same crypto crate compiled to WASM — guaranteed identical behavior to server. |
| Server framework | Rust + `axum` | Lightweight, well-audited async HTTP. Comparable simplicity to Go `net/http`. |
| BB storage | SQLite via `rusqlite` | Embedded, zero-config, sufficient for demo. |
| Web client | TypeScript + React (Vite) | Thin UI shell; all crypto in Rust→WASM |
| Mobile client | React Native (TypeScript) | Shared logic with web client |
| Build system | Nix + Cargo + `wasm-pack` | Reproducible builds, deterministic outputs |
| CI/CD | GitHub Actions | Public, auditable pipeline |

### 8.3 Dependency Policy

**Two-tier rule**: absolute zero external crates on the crypto path; minimal, well-audited crates for non-crypto server plumbing.

| Tier | Scope | External crates allowed |
|---|---|---|
| **Crypto path** (security-critical) | `packages/crypto` — ElGamal, ZKPs, DKG, blind sigs, ballot encoding | **ZERO**. Only `core`/`alloc`/`std`. P-256 field arithmetic, scalar arithmetic, point operations — all from first principles. `#[no_std]`-compatible. |
| **Election domain** (data integrity) | `packages/election` — election types (Mir, Party, Candidate), embedded ballot data, validation | `serde` (deserialization), `toml` (TOML parsing). Depends on `glasuvai-crypto` for SHA-256 integrity verification. Build-time only: `sha2`, `walkdir` (for data digest computation). Feature flags select which election's data is embedded (`bg-na51-2024`, etc.). |
| **Server plumbing** (not security-critical) | HTTP handlers, JSON serialization, SQLite storage, async runtime | `axum`, `tokio`, `serde`/`serde_json`, `rusqlite`. Each is mature, widely audited, and replaceable. |
| **WASM bridge** | `packages/crypto-wasm` — JS interop only | `wasm-bindgen` (for JS glue). No crypto logic in this layer. |
| **Web client** | UI only | React (via npm). All crypto delegated to WASM module. |

**Why this is stricter than the original Go plan**: Go's `crypto/elliptic` and `math/big` are stdlib but still external code trusted implicitly. In the Rust version, the crypto path trusts **no code outside this repository** — P-256 is implemented from scratch.

**Single implementation advantage**: Since server and client share the same Rust crypto crate, there are no cross-language consistency issues. No shared test vectors needed for sync — `cargo test` covers everything. One bug fix applies everywhere.
