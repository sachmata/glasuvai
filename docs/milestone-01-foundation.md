# Milestone 1: Foundation & Election Data

## Goal

Establish the Rust workspace structure, define all shared types, and encode real Bulgarian 51st National Assembly election data (October 27, 2024) as structured constants. Every subsequent milestone imports from this foundation.

## Prerequisites

None — this is the starting point.

## Deliverables

```
Cargo.toml                  # Workspace root (members = ["packages/*"])

packages/crypto/
  Cargo.toml                # name = "glasuvai-crypto", no external deps
  src/
    lib.rs                  # Crate root, re-exports modules
    election/
      mod.rs                # Module declarations
      types.rs              # Core election types (MIR, Party, Candidate, Ballot spec)
      mir.rs                # All 31+1 MIRs with seat counts
      parties.rs            # Registered parties for 51st NA
      candidates_mir23.rs   # Full candidate lists for demo MIR (Sofia 23)
      candidates_mir24.rs   # Sofia 24 (secondary demo MIR)
      candidates_mir25.rs   # Sofia 25 (secondary demo MIR)
      config.rs             # Election parameters (thresholds, dates, rules)
      validate.rs           # Validation functions for election data

packages/admin/
  Cargo.toml                # name = "glasuvai-admin"
  src/
    main.rs                 # CLI to print election data as JSON (for web client)
```

## Workspace Structure

The project uses a Cargo workspace so all packages share a single `Cargo.lock` and can depend on each other with simple path dependencies:

```toml
# Root Cargo.toml
[workspace]
resolver = "2"
members = [
    "packages/crypto",
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

# Optional std feature; the crate is #[no_std]-compatible
[features]
default = ["std"]
std = []
```

## Data Structures

### Core Types (`packages/crypto/src/election/types.rs`)

```rust
/// MIR represents a multi-member constituency (Многомандатен избирателен район)
#[derive(Debug, Clone)]
pub struct MIR {
    pub id: u32,           // 1-31 domestic, 32 diaspora
    pub name: &'static str,       // Bulgarian name, e.g., "Благоевград"
    pub name_latin: &'static str, // Latin transliteration
    pub seats: u32,        // Number of parliamentary seats allocated
}

/// Party represents a registered party or coalition
#[derive(Debug, Clone)]
pub struct Party {
    pub number: u32,        // Ballot number (assigned by ЦИК lottery)
    pub name: &'static str,        // Official name in Bulgarian
    pub name_latin: &'static str,  // Latin transliteration
    pub short: &'static str,       // Abbreviation (e.g., "ГЕРБ-СДС")
    pub coalition: bool,    // true if coalition, false if single party
}

/// Candidate represents a candidate on a party list for a specific MIR
#[derive(Debug, Clone)]
pub struct Candidate {
    pub position: u32,       // Position on the party list (1-indexed)
    pub first_name: &'static str,  // Given name in Bulgarian
    pub last_name: &'static str,   // Family name in Bulgarian
    pub party_num: u32,      // Reference to Party.number
    pub mir_id: u32,         // Reference to MIR.id
}

/// BallotSpec defines the ballot structure for a specific MIR
#[derive(Debug, Clone)]
pub struct BallotSpec {
    pub mir_id: u32,                               // Which MIR
    pub parties: Vec<Party>,                       // Parties registered in this MIR (ordered by ballot number)
    pub candidates: Vec<(u32, Vec<Candidate>)>,    // (party_number, ordered candidates)
    pub max_candidates: u32,                       // Maximum candidates per list in this MIR
}

/// ElectionConfig holds election-wide parameters
#[derive(Debug, Clone)]
pub struct ElectionConfig {
    pub election_id: &'static str,        // e.g., "bg-na51-2024"
    pub name: &'static str,               // "Избори за 51-о Народно събрание"
    pub date: &'static str,               // "2024-10-27"
    pub total_mirs: u32,                   // 32
    pub national_threshold: f64,           // 0.04 (4%)
    pub preference_threshold: f64,         // 0.07 (7%)
    pub total_seats: u32,                  // 240
    pub seat_allocation: &'static str,     // "hare-niemeyer" (largest remainder)
}
```

## Implementation Steps

### Step 1: Initialize Rust Workspace

```bash
cd /home/martin/Projects/glasuvai
# Create root Cargo.toml with workspace members
# Create packages/crypto/Cargo.toml with zero deps
cargo init --lib packages/crypto --name glasuvai-crypto
```

### Step 2: Define Election Types

Create `election/types.rs` with the types above. Keep types simple — just data structures. Derive `Debug` and `Clone` for all types.

### Step 3: Encode MIR Data (Real)

Create `election/mir.rs` with all 32 MIRs from the 51st National Assembly:

```rust
use super::types::MIR;

/// All 32 multi-member constituencies for the 51st National Assembly
pub const MIRS: &[MIR] = &[
    MIR { id: 1, name: "Благоевград", name_latin: "Blagoevgrad", seats: 12 },
    MIR { id: 2, name: "Бургас", name_latin: "Burgas", seats: 13 },
    MIR { id: 3, name: "Варна", name_latin: "Varna", seats: 14 },
    MIR { id: 4, name: "Велико Търново", name_latin: "Veliko Tarnovo", seats: 8 },
    MIR { id: 5, name: "Видин", name_latin: "Vidin", seats: 4 },
    MIR { id: 6, name: "Враца", name_latin: "Vratsa", seats: 6 },
    MIR { id: 7, name: "Габрово", name_latin: "Gabrovo", seats: 4 },
    MIR { id: 8, name: "Добрич", name_latin: "Dobrich", seats: 6 },
    MIR { id: 9, name: "Кърджали", name_latin: "Kardzhali", seats: 5 },
    MIR { id: 10, name: "Кюстендил", name_latin: "Kyustendil", seats: 5 },
    MIR { id: 11, name: "Ловеч", name_latin: "Lovech", seats: 5 },
    MIR { id: 12, name: "Монтана", name_latin: "Montana", seats: 5 },
    MIR { id: 13, name: "Пазарджик", name_latin: "Pazardzhik", seats: 9 },
    MIR { id: 14, name: "Перник", name_latin: "Pernik", seats: 5 },
    MIR { id: 15, name: "Плевен", name_latin: "Pleven", seats: 9 },
    MIR { id: 16, name: "Пловдив-град", name_latin: "Plovdiv City", seats: 12 },
    MIR { id: 17, name: "Пловдив-област", name_latin: "Plovdiv Region", seats: 7 },
    MIR { id: 18, name: "Разград", name_latin: "Razgrad", seats: 4 },
    MIR { id: 19, name: "Русе", name_latin: "Ruse", seats: 8 },
    MIR { id: 20, name: "Силистра", name_latin: "Silistra", seats: 4 },
    MIR { id: 21, name: "Сливен", name_latin: "Sliven", seats: 6 },
    MIR { id: 22, name: "Смолян", name_latin: "Smolyan", seats: 4 },
    MIR { id: 23, name: "София 23", name_latin: "Sofia 23", seats: 16 },
    MIR { id: 24, name: "София 24", name_latin: "Sofia 24", seats: 16 },
    MIR { id: 25, name: "София 25", name_latin: "Sofia 25", seats: 16 },
    MIR { id: 26, name: "Софийска област", name_latin: "Sofia Region", seats: 8 },
    MIR { id: 27, name: "Стара Загора", name_latin: "Stara Zagora", seats: 11 },
    MIR { id: 28, name: "Търговище", name_latin: "Targovishte", seats: 4 },
    MIR { id: 29, name: "Хасково", name_latin: "Haskovo", seats: 8 },
    MIR { id: 30, name: "Шумен", name_latin: "Shumen", seats: 6 },
    MIR { id: 31, name: "Ямбол", name_latin: "Yambol", seats: 5 },
    MIR { id: 32, name: "Чужбина", name_latin: "Abroad (Diaspora)", seats: 4 },
];
// Total: 240 seats
```

### Step 4: Encode Party Data (Real — 51st NA)

Create `election/parties.rs` with parties that participated in the 51st NA elections:

```rust
use super::types::Party;

/// Parties/coalitions registered for the 51st National Assembly elections
/// Ballot numbers assigned by ЦИК lottery draw
pub const PARTIES_51NA: &[Party] = &[
    Party { number: 1, name: "МЕЧтА (Морал Единство Чест Алтернатива)", name_latin: "MEChTA", short: "МЕЧтА", coalition: true },
    Party { number: 2, name: "АЛТЕРНАТИВА ЗА БЪЛГАРСКО ВЪЗРАЖДАНЕ", name_latin: "ABV", short: "АБВ", coalition: false },
    Party { number: 3, name: "ГЕРБ-СДС", name_latin: "GERB-SDS", short: "ГЕРБ-СДС", coalition: true },
    Party { number: 4, name: "ДВИЖЕНИЕ ЗА ПРАВА И СВОБОДИ – НОВО НАЧАЛО", name_latin: "DPS-New Beginning", short: "ДПС-НН", coalition: false },
    Party { number: 5, name: "БСП – ОБЕДИНЕНА ЛЕВИЦА", name_latin: "BSP-United Left", short: "БСП-ОЛ", coalition: true },
    Party { number: 6, name: "ИМА ТАКЪВ НАРОД", name_latin: "ITN", short: "ИТН", coalition: false },
    Party { number: 7, name: "ВЕЛИЧИЕ", name_latin: "Velichie", short: "ВЕЛИЧИЕ", coalition: false },
    Party { number: 8, name: "ПРОДЪЛЖАВАМЕ ПРОМЯНАТА – ДЕМОКРАТИЧНА БЪЛГАРИЯ", name_latin: "PP-DB", short: "ПП-ДБ", coalition: true },
    Party { number: 9, name: "ВЪЗРАЖДАНЕ", name_latin: "Vazrazhdane", short: "ВЪЗРАЖДАНЕ", coalition: false },
    Party { number: 10, name: "СИНЯ БЪЛГАРИЯ", name_latin: "Blue Bulgaria", short: "СБ", coalition: false },
    Party { number: 11, name: "БЪЛГАРСКИ НАЦИОНАЛЕН СЪЮЗ – НД", name_latin: "BNS-ND", short: "БНС-НД", coalition: false },
    Party { number: 12, name: "ЛЕВИЦАТА!", name_latin: "The Left!", short: "ЛЕВИЦАТА!", coalition: false },
    Party { number: 13, name: "ПОЛИТИЧЕСКО ДВИЖЕНИЕ СОЦИАЛДЕМОКРАТИ", name_latin: "Political Movement Social Democrats", short: "ПДСД", coalition: false },
    Party { number: 14, name: "ДВИЖЕНИЕ ЗА ПРАВА И СВОБОДИ", name_latin: "DPS", short: "ДПС", coalition: false },
];
```

> **Note**: Exact ballot numbers and the full list of registered parties should be cross-referenced with the official ЦИК register at `elections.bg`. The above is representative of the major parties. The demo must include all parties registered in the demo MIR.

### Step 5: Encode Candidate Data for Demo MIR

Create `election/candidates_mir23.rs` with candidate lists for MIR 23 (Sofia 23).

Each party that registered in this MIR submits a candidate list of up to N candidates (N ≤ 2× seat count, so up to 32 for a 16-seat MIR):

```rust
use super::types::Candidate;

/// Candidate lists for MIR 23 (Sofia 23) for the 51st National Assembly elections.
///
/// Data source: ЦИК official register (elections.bg)
/// Each tuple is (party_number, candidates).
pub fn candidates_mir23() -> Vec<(u32, Vec<Candidate>)> {
    vec![
        (3, vec![ // ГЕРБ-СДС
            Candidate { position: 1, first_name: "Бойко", last_name: "Борисов", party_num: 3, mir_id: 23 },
            Candidate { position: 2, first_name: "Даниел", last_name: "Митов", party_num: 3, mir_id: 23 },
            Candidate { position: 3, first_name: "Деница", last_name: "Сачева", party_num: 3, mir_id: 23 },
            // ... up to 32 candidates per list
            // Full list to be populated from ЦИК data
        ]),
        (8, vec![ // ПП-ДБ
            Candidate { position: 1, first_name: "Кирил", last_name: "Петков", party_num: 8, mir_id: 23 },
            Candidate { position: 2, first_name: "Асен", last_name: "Василев", party_num: 8, mir_id: 23 },
            Candidate { position: 3, first_name: "Христо", last_name: "Иванов", party_num: 8, mir_id: 23 },
            // ... full list
        ]),
        (9, vec![ // Възраждане
            Candidate { position: 1, first_name: "Костадин", last_name: "Костадинов", party_num: 9, mir_id: 23 },
            // ... full list
        ]),
        (4, vec![ // ДПС-НН
            Candidate { position: 1, first_name: "Делян", last_name: "Пеевски", party_num: 4, mir_id: 23 },
            // ... full list
        ]),
        (5, vec![ // БСП-ОЛ
            Candidate { position: 1, first_name: "Корнелия", last_name: "Нинова", party_num: 5, mir_id: 23 },
            // ... full list
        ]),
        // ... all registered parties in MIR 23
    ]
}
```

> **IMPORTANT**: The candidate data above uses list leaders as known from public sources. Full candidate lists (all positions) MUST be sourced from the official ЦИК register for authenticity. The demo's credibility depends on using real data.

### Step 6: Election Configuration

Create `election/config.rs`:

```rust
use super::types::ElectionConfig;

/// Election configuration for the 51st National Assembly
pub const CONFIG_51NA: ElectionConfig = ElectionConfig {
    election_id: "bg-na51-2024",
    name: "Избори за 51-о Народно събрание",
    date: "2024-10-27",
    total_mirs: 32,
    national_threshold: 0.04,
    preference_threshold: 0.07,
    total_seats: 240,
    seat_allocation: "hare-niemeyer",
};
```

### Step 7: Validation Functions

Create `election/validate.rs` with functions to check data consistency:

```rust
use super::types::{BallotSpec, MIR};

/// Checks that a BallotSpec is internally consistent
pub fn validate_ballot_spec(spec: &BallotSpec) -> Result<(), String> {
    // - Every candidate references a party in the spec
    // - Candidate positions are sequential (1, 2, 3, ...)
    // - No duplicate party numbers
    // - mir_id matches across all entries
    // - At least one party registered
    Ok(())
}

/// Checks that total seats across all MIRs = 240
pub fn validate_mir_seats(mirs: &[MIR]) -> Result<(), String> {
    let total: u32 = mirs.iter().map(|m| m.seats).sum();
    if total != 240 {
        return Err(format!("expected 240 total seats, got {}", total));
    }
    Ok(())
}
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
glasuvai-crypto = { path = "../crypto" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

```rust
// packages/admin/src/main.rs
// CLI to export election data as JSON (for web client and testing)
fn main() {
    // Parse args: --mir 23 --format json
    // Read election data from glasuvai-crypto
    // Validate data
    // Output JSON
}
```

```bash
cargo run -p glasuvai-admin -- --mir 23 --format json > test/data/mir23.json
```

## Acceptance Criteria

- [ ] `cargo build -p glasuvai-crypto` succeeds with zero warnings
- [ ] `cargo test -p glasuvai-crypto` passes all tests
- [ ] `validate_mir_seats` confirms exactly 240 total seats
- [ ] `validate_ballot_spec` passes for MIR 23 demo data
- [ ] Admin CLI exports valid JSON for MIR 23
- [ ] All party names and candidate data match ЦИК official records
- [ ] `cargo tree -p glasuvai-crypto` shows zero external dependencies
- [ ] Crypto crate compiles with `#[no_std]` (verified via feature flag)
