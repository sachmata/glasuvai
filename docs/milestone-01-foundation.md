# Milestone 1: Foundation & Election Data

## Goal

Establish the Rust workspace structure with a clean separation between the crypto crate (pure cryptographic primitives, `#[no_std]`, zero deps) and a shared library crate (election domain types + embedded ballot data). Real Bulgarian 51st National Assembly election data (October 27, 2024) is stored as TOML files and compiled into the shared crate with a SHA-256 integrity digest for anti-tampering.

## Prerequisites

None — this is the starting point.

## Deliverables

```
# Build reproducibility (pinned toolchains & dependencies)
rust-toolchain.toml         # Pins exact Rust compiler version (1.85.0) + WASM target
flake.nix                   # Nix flake: pins nixpkgs, Rust overlay, Node.js, all system deps
flake.lock                  # Auto-generated: locks every Nix input to exact git commit hash
.cargo/config.toml          # Cargo settings (WASM optimizations)

Cargo.toml                  # Workspace root (members = ["packages/*"])
Cargo.lock                  # Auto-generated: pins all Cargo dependency versions + checksums

# ── Election ballot data (serialised, human-editable) ──────────────────────
data/elections/bg-na51-2024/
  election.toml             # ElectionConfig (thresholds, dates, rules)
  mirs.toml                 # All 31+1 MIRs with seat counts
  parties.toml              # Registered parties/coalitions for 51st NA
  candidates/
    mir-23.toml             # Full candidate lists for demo MIR (Sofia 23)
    mir-24.toml             # Sofia 24 (secondary demo MIR)
    mir-25.toml             # Sofia 25 (secondary demo MIR)

# ── Crypto crate (pure primitives, zero deps, #[no_std]) ──────────────────
packages/crypto/
  Cargo.toml                # name = "glasuvai-crypto", zero external deps
  src/
    lib.rs                  # Crate root — only crypto module stubs in M1

# ── Election domain crate (types + embedded ballot data) ───────────────────
packages/election/
  Cargo.toml                # name = "glasuvai-election", depends on serde + toml
  build.rs                  # Computes SHA-256 digest of data/ tree at build time
  src/
    lib.rs                  # Crate root, re-exports modules
    election/
      mod.rs                # Module declarations
      types.rs              # Core election types (Mir, Party, Candidate, BallotSpec, ElectionConfig)
      data.rs               # Embeds TOML files via include_str!, parses & exposes election data
      validate.rs           # Validation functions (seat totals, ballot spec consistency)
      integrity.rs          # Compile-time SHA-256 digest of embedded ballot data

# ── Admin CLI ──────────────────────────────────────────────────────────────
packages/admin/
  Cargo.toml                # name = "glasuvai-admin"
  src/
    main.rs                 # CLI to validate & export election data as JSON
```

## Architecture: Why Two Crates?

```
┌──────────────────────────────────┐     ┌──────────────────────────────────┐
│  glasuvai-crypto                 │     │  glasuvai-election               │
│                                  │     │                                  │
│  #[no_std], ZERO external deps   │     │  serde + toml deps              │
│                                  │     │                                  │
│  • P-256 field/curve arithmetic  │     │  • Mir, Party, Candidate types   │
│  • ElGamal encryption            │     │  • ElectionConfig, BallotSpec    │
│  • Chaum-Pedersen ZKPs           │     │  • Embed TOML ballot data        │
│  • Pedersen DKG                  │     │  • SHA-256 integrity digest      │
│  • RSA blind signatures          │     │  • Validation functions          │
│  • SHA-256                       │     │                                  │
│                                  │     │  depends on: glasuvai-crypto     │
│  depends on: nothing             │     │  (for hash verification)         │
└──────────────────────────────────┘     └──────────────────────────────────┘
         ▲                                          ▲
         │                                          │
         └──── used by all packages ────────────────┘
```

**Rationale**: The crypto crate stays minimal — no data, no I/O, no `serde`, no `alloc`-heavy containers. It compiles cleanly to `#[no_std]` and WASM. The election crate owns all election domain types and ballot data; it uses `serde` for deserialization and depends on `glasuvai-crypto` only for hash verification of embedded data.

## Workspace Structure

```toml
# Root Cargo.toml
[workspace]
resolver = "2"
members = [
    "packages/crypto",
    "packages/election",
    "packages/crypto-wasm",
    "packages/admin",
    "packages/bulletin-board",
    "packages/identity-provider",
    "packages/voting-server",
    "packages/tally",
    "packages/verifier",
    "packages/trustee-tool",
]
```

```toml
# packages/crypto/Cargo.toml
[package]
name = "glasuvai-crypto"
version = "0.1.0"
edition = "2021"

# ZERO external dependencies — all crypto from first principles
[dependencies]
# (empty)

[features]
default = ["std"]
std = []
```

```toml
# packages/election/Cargo.toml
[package]
name = "glasuvai-election"
version = "0.1.0"
edition = "2021"
build = "build.rs"

[dependencies]
glasvai-crypto = { path = "../crypto" }
serde = { version = "1", features = ["derive"] }
toml = "0.8"

# Feature flags select which election's data gets embedded.
# Only one election should be active at a time.
[features]
default = ["bg-na51-2024"]
bg-na51-2024 = []   # 51st National Assembly, Oct 2024
# bg-na52-2025 = [] # future elections add a feature

[build-dependencies]
sha2 = "0.10"     # Used ONLY in build.rs to compute data digest
walkdir = "2"     # Used ONLY in build.rs to enumerate data files
```

## Ballot Data Format (TOML)

Real election data lives in `data/elections/bg-na51-2024/` as human-readable TOML files. These are the source of truth — editable, auditable, diffable in git.

### `election.toml`

```toml
election_id = "bg-na51-2024"
name = "Избори за 51-о Народно събрание"
date = "2024-10-27"
total_mirs = 32
national_threshold = 0.04
preference_threshold = 0.07
total_seats = 240
seat_allocation = "hare-niemeyer"
```

### `mirs.toml`

```toml
[[mir]]
id = 1
name = "Благоевград"
name_latin = "Blagoevgrad"
seats = 12

[[mir]]
id = 2
name = "Бургас"
name_latin = "Burgas"
seats = 13

# ... all 32 MIRs
# Total seats must sum to 240

[[mir]]
id = 32
name = "Чужбина"
name_latin = "Abroad (Diaspora)"
seats = 4
```

### `parties.toml`

```toml
[[party]]
number = 1
name = "МЕЧтА (Морал Единство Чест Алтернатива)"
name_latin = "MEChTA"
short = "МЕЧтА"
coalition = true

[[party]]
number = 3
name = "ГЕРБ-СДС"
name_latin = "GERB-SDS"
short = "ГЕРБ-СДС"
coalition = true

# ... all registered parties
```

### `candidates/mir-23.toml`

```toml
mir_id = 23

[[party_list]]
party_number = 3  # ГЕРБ-СДС

[[party_list.candidate]]
position = 1
first_name = "Бойко"
last_name = "Борисов"

[[party_list.candidate]]
position = 2
first_name = "Даниел"
last_name = "Митов"

[[party_list.candidate]]
position = 3
first_name = "Деница"
last_name = "Сачева"

# ... up to 32 candidates per list

[[party_list]]
party_number = 8  # ПП-ДБ

[[party_list.candidate]]
position = 1
first_name = "Кирил"
last_name = "Петков"

# ... all parties with candidate lists registered in MIR 23
```

> **IMPORTANT**: All party names, ballot numbers, and candidate data MUST be sourced from the official ЦИК register at `elections.bg`. The demo's credibility depends on using real data.

## Data Structures

### Election Types (`packages/election/src/election/types.rs`)

```rust
use serde::Deserialize;

/// Multi-member constituency (Многомандатен избирателен район — МИР).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Mir {
    pub id: u32,
    pub name: String,
    pub name_latin: String,
    pub seats: u32,
}

/// Registered political party or electoral coalition.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Party {
    pub number: u32,
    pub name: String,
    pub name_latin: String,
    pub short: String,
    pub coalition: bool,
}

/// Candidate on a party list for a specific MIR.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Candidate {
    pub position: u32,
    pub first_name: String,
    pub last_name: String,
}

/// A party's candidate list within a MIR.
#[derive(Debug, Clone, Deserialize)]
pub struct PartyList {
    pub party_number: u32,
    #[serde(rename = "candidate")]
    pub candidates: Vec<Candidate>,
}

/// Candidate file for a single MIR.
#[derive(Debug, Clone, Deserialize)]
pub struct MirCandidates {
    pub mir_id: u32,
    #[serde(rename = "party_list")]
    pub party_lists: Vec<PartyList>,
}

/// Complete ballot specification for a specific MIR (assembled at runtime).
#[derive(Debug, Clone)]
pub struct BallotSpec {
    pub mir_id: u32,
    pub parties: Vec<Party>,
    pub candidates: Vec<PartyList>,
    pub max_candidates: u32,
}

/// Election-wide configuration parameters.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ElectionConfig {
    pub election_id: String,
    pub name: String,
    pub date: String,
    pub total_mirs: u32,
    pub national_threshold: f64,
    pub preference_threshold: f64,
    pub total_seats: u32,
    pub seat_allocation: String,
}
```

Note: Types use `String` (not `&'static str`) because they are deserialized from TOML at runtime. The `Deserialize` derive comes from `serde`.

## Implementation Steps

### Step 0: Set Up Reproducible Build Infrastructure

Before writing any Rust code, establish pinned toolchains so every build is verifiable:

1. **`rust-toolchain.toml`** — pins Rust compiler to `1.85.0` with `wasm32-unknown-unknown` target and `rustfmt`/`clippy` components. Cargo/rustup automatically use this version.
2. **`flake.nix`** — Nix flake that pins `nixpkgs` (24.11 LTS), `rust-overlay` (for exact rustc), `wasm-pack`, `nodejs_22`, `sqlite`, `cargo-audit`, and all system deps. Provides `nix develop` shell with identical environment for every contributor.
3. **`flake.lock`** — auto-generated by `nix flake update`. Records exact git commit hash of every input. This is the master pin — anyone running `nix build` from the same commit gets bit-for-bit identical outputs.
4. **`.cargo/config.toml`** — workspace-wide Cargo settings (WASM size optimization flags).

```bash
cd /home/martin/Projects/glasuvai
nix develop
```

Verification: `rustc --version` shows `1.85.0`, `node --version` shows `v22.x`, `wasm-pack --version` is available.

### Step 1: Initialize Rust Workspace

```bash
cd /home/martin/Projects/glasuvai
# Root Cargo.toml already exists (workspace members)
# packages/crypto already exists (skeleton)
cargo init --lib packages/election --name glasuvai-election
# Commit Cargo.lock to git (mandatory for reproducible builds)
```

### Step 2: Create Ballot Data Files

Populate `data/elections/bg-na51-2024/` with TOML files containing real election data sourced from the official ЦИК register. Files:

- `election.toml` — election-wide config
- `mirs.toml` — all 32 MIRs
- `parties.toml` — all registered parties
- `candidates/mir-23.toml` — full candidate lists for MIR 23 (primary demo)
- `candidates/mir-24.toml` — MIR 24 (secondary)
- `candidates/mir-25.toml` — MIR 25 (secondary)

### Step 3: Define Election Types in Election Crate

Create `packages/election/src/election/types.rs` with the types above. All types derive `Debug`, `Clone`, and `serde::Deserialize`.

### Step 4: Embed and Parse Ballot Data

Create `packages/election/src/election/data.rs`:

```rust
use super::types::*;

// Embed TOML files at compile time, gated by feature flag.
// The active feature determines which election's data is compiled in.
// Any change to these files triggers a recompile.

#[cfg(feature = "bg-na51-2024")]
mod embedded {
    pub const ELECTION_TOML: &str = include_str!("../../../data/elections/bg-na51-2024/election.toml");
    pub const MIRS_TOML: &str = include_str!("../../../data/elections/bg-na51-2024/mirs.toml");
    pub const PARTIES_TOML: &str = include_str!("../../../data/elections/bg-na51-2024/parties.toml");
    pub const MIR23_TOML: &str = include_str!("../../../data/elections/bg-na51-2024/candidates/mir-23.toml");
    // ... additional MIRs as needed
}
use embedded::*;

/// SHA-256 hex digest of all embedded data files, computed at build time.
/// Used for anti-tampering verification — the verifier and bulletin board
/// can confirm that the election data matches the published digest.
pub const DATA_INTEGRITY_DIGEST: &str = env!("GLASUVAI_DATA_SHA256");

/// Parse the embedded election configuration.
pub fn election_config() -> ElectionConfig {
    toml::from_str(ELECTION_TOML).expect("embedded election.toml is valid")
}

/// Parse the embedded MIR table.
pub fn mirs() -> Vec<Mir> {
    #[derive(Deserialize)]
    struct MirFile { mir: Vec<Mir> }
    let f: MirFile = toml::from_str(MIRS_TOML).expect("embedded mirs.toml is valid");
    f.mir
}

/// Parse the embedded party table.
pub fn parties() -> Vec<Party> {
    #[derive(Deserialize)]
    struct PartyFile { party: Vec<Party> }
    let f: PartyFile = toml::from_str(PARTIES_TOML).expect("embedded parties.toml is valid");
    f.party
}

/// Parse the embedded candidate data for MIR 23.
pub fn candidates_mir23() -> MirCandidates {
    toml::from_str(MIR23_TOML).expect("embedded mir-23.toml is valid")
}

/// Build a complete BallotSpec for a MIR by combining party + candidate data.
pub fn ballot_spec(mir_id: u32) -> BallotSpec {
    let all_parties = parties();
    let mir_candidates = match mir_id {
        23 => candidates_mir23(),
        _ => panic!("candidate data not available for MIR {mir_id}"),
    };
    let registered_party_nums: Vec<u32> = mir_candidates.party_lists
        .iter().map(|pl| pl.party_number).collect();
    let mir_parties: Vec<Party> = all_parties.into_iter()
        .filter(|p| registered_party_nums.contains(&p.number))
        .collect();
    let max_candidates = mir_candidates.party_lists
        .iter().map(|pl| pl.candidates.len() as u32).max().unwrap_or(0);
    BallotSpec {
        mir_id,
        parties: mir_parties,
        candidates: mir_candidates.party_lists,
        max_candidates,
    }
}
```

### Step 5: Build Script for Integrity Digest

Create `packages/election/build.rs`:

```rust
//! Build script: computes a SHA-256 digest over all ballot data files
//! and exposes it as the `GLASUVAI_DATA_SHA256` environment variable
//! (accessible via `env!()` in the crate source).

use sha2::{Sha256, Digest};
use walkdir::WalkDir;
use std::fs;

fn main() {
    let data_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/elections");
    println!("cargo::rerun-if-changed={data_dir}");

    let mut hasher = Sha256::new();
    let mut paths: Vec<_> = WalkDir::new(data_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.into_path())
        .collect();

    // Sort for deterministic ordering across platforms
    paths.sort();

    for path in &paths {
        let content = fs::read(path).expect("failed to read data file");
        hasher.update(&content);
    }

    let digest = format!("{:x}", hasher.finalize());
    println!("cargo::rustc-env=GLASUVAI_DATA_SHA256={digest}");
}
```

Any change to any file under `data/elections/` triggers a rebuild and a new digest. The verifier tool (M7) and bulletin board can compare this digest against the published one.

### Step 6: Validation Functions

Create `packages/election/src/election/validate.rs`:

```rust
use super::types::{BallotSpec, Mir};

/// Checks that total seats across all MIRs equals the expected 240.
pub fn validate_mir_seats(mirs: &[Mir], expected: u32) -> Result<(), String> {
    let total: u32 = mirs.iter().map(|m| m.seats).sum();
    if total != expected {
        return Err(format!("expected {expected} total seats, got {total}"));
    }
    Ok(())
}

/// Checks that a BallotSpec is internally consistent.
pub fn validate_ballot_spec(spec: &BallotSpec) -> Result<(), String> {
    if spec.parties.is_empty() {
        return Err("no parties registered".into());
    }
    // Every party list references a party present in spec.parties
    for pl in &spec.candidates {
        if !spec.parties.iter().any(|p| p.number == pl.party_number) {
            return Err(format!("party list references unknown party {}", pl.party_number));
        }
        // Positions must be sequential 1..=N
        for (i, c) in pl.candidates.iter().enumerate() {
            if c.position != (i as u32 + 1) {
                return Err(format!(
                    "party {} candidate at index {} has position {} (expected {})",
                    pl.party_number, i, c.position, i + 1
                ));
            }
        }
    }
    // No duplicate party numbers
    let mut seen = std::collections::HashSet::new();
    for pl in &spec.candidates {
        if !seen.insert(pl.party_number) {
            return Err(format!("duplicate party list for party {}", pl.party_number));
        }
    }
    Ok(())
}
```

### Step 7: Clean Up Crypto Crate

Remove the `election` module from `packages/crypto`. In M1 the crypto crate is a skeleton — its only job is to exist with zero deps and compile under `#[no_std]`. Crypto primitives (P-256, ElGamal, ZKPs, SHA-256) are implemented in M2.

```rust
// packages/crypto/src/lib.rs
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
```

### Step 8: Admin CLI for Data Export

Create `packages/admin/`:

```toml
# packages/admin/Cargo.toml
[package]
name = "glasuvai-admin"
version = "0.1.0"
edition = "2021"

[dependencies]
glasuvai-election = { path = "../election" }
serde_json = "1"
```

```rust
// packages/admin/src/main.rs
use glasuvai_election::election::{data, validate};

fn main() {
    let config = data::election_config();
    let mirs = data::mirs();
    let parties = data::parties();

    // Validate
    validate::validate_mir_seats(&mirs, config.total_seats)
        .expect("MIR seat validation failed");

    println!("Election: {} ({})", config.name, config.date);
    println!("Data integrity: {}", data::DATA_INTEGRITY_DIGEST);
    println!("MIRs: {}, Parties: {}", mirs.len(), parties.len());

    // --mir 23 → export ballot spec as JSON
    let args: Vec<String> = std::env::args().collect();
    if let Some(mir_arg) = args.iter().position(|a| a == "--mir") {
        let mir_id: u32 = args[mir_arg + 1].parse().expect("invalid MIR number");
        let spec = data::ballot_spec(mir_id);
        validate::validate_ballot_spec(&spec)
            .expect("ballot spec validation failed");
        let json = serde_json::to_string_pretty(&spec)
            .expect("JSON serialisation failed");
        println!("{json}");
    }
}
```

```bash
cargo run -p glasuvai-admin -- --mir 23
```

## Acceptance Criteria

- [ ] `nix develop` enters shell with correct pinned toolchain versions
- [ ] `rustc --version` inside dev shell shows `1.85.0`
- [ ] `flake.lock` exists and pins all inputs to exact commit hashes
- [ ] `cargo build -p glasuvai-crypto` succeeds with zero warnings and zero external deps
- [ ] `cargo tree -p glasuvai-crypto` shows zero external dependencies
- [ ] Crypto crate compiles with `#[no_std]` (verified via feature flag)
- [ ] `cargo build -p glasuvai-election` succeeds — TOML files parse correctly
- [ ] `cargo test -p glasuvai-election` passes all validation tests
- [ ] `validate_mir_seats` confirms exactly 240 total seats
- [ ] `validate_ballot_spec` passes for MIR 23 demo data
- [ ] `DATA_INTEGRITY_DIGEST` is a stable SHA-256 hex string that changes only when data files change
- [ ] Admin CLI exports valid JSON for MIR 23
- [ ] All party names and candidate data match ЦИК official records
- [ ] TOML data files are valid, human-readable, and diffable in git
- [ ] `Cargo.lock` is committed to git
