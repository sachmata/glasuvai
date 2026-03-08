# Milestone 6: Tally & Threshold Decryption

## Goal

Implement the complete post-election tallying pipeline: resolve re-votes and overrides, perform homomorphic aggregation of encrypted ballots, execute the threshold decryption ceremony with proofs, solve discrete logarithms to recover vote counts, and apply the Bulgarian Hare-Niemeyer seat allocation algorithm with preference thresholds. Every step produces publicly verifiable proof.

## Prerequisites

- **M1**: Election types, MIR data, election config
- **M2**: ElGamal (homomorphic add, partial decryption, DLOG equality proof, baby-step giant-step)
- **M3**: Ballot deserialization
- **M4**: Bulletin Board data access

## Deliverables

```
packages/tally/
  Cargo.toml                # name = "glasuvai-tally"
  src/
    main.rs                 # Tally CLI binary
    lib.rs                  # Library root
    resolve/
      mod.rs
      dedup.rs              # Re-vote de-duplication
      override.rs           # In-person override exclusion
    aggregate/
      mod.rs
      aggregate.rs          # Homomorphic aggregation per MIR
    decrypt/
      mod.rs
      partial.rs            # Partial decryption by one trustee
      combine.rs            # Lagrange interpolation to combine partials
      dlog.rs               # Baby-step giant-step DLOG solver
      ceremony.rs           # Full decryption ceremony orchestrator
    allocate/
      mod.rs
      hare.rs               # Hare-Niemeyer largest remainder method
      preference.rs         # 7% preference threshold evaluation
      results.rs            # Final results compilation
    report/
      mod.rs
      generate.rs           # Results report generation (JSON + human-readable)

packages/trustee-tool/
  Cargo.toml                # name = "glasuvai-trustee-tool"
  src/
    main.rs                 # Trustee CLI (keygen, decrypt, verify)
```

```toml
# packages/tally/Cargo.toml
[package]
name = "glasuvai-tally"
version = "0.1.0"
edition = "2021"

[dependencies]
glasuvai-crypto = { path = "../crypto" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

```toml
# packages/trustee-tool/Cargo.toml
[package]
name = "glasuvai-trustee-tool"
version = "0.1.0"
edition = "2021"

[dependencies]
glasuvai-crypto = { path = "../crypto" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

## Phase 1: Resolve Re-Votes and Overrides

### De-Duplication (`resolve/dedup.rs`)

```rust
use std::collections::HashMap;

/// The de-duplicated ballot set for a MIR.
pub struct DedupResult {
    pub mir_id: u32,
    pub final_ballots: Vec<FinalBallot>,  // Only the latest ballot per token
    pub superseded: Vec<u64>,             // BB entry indices of superseded ballots
    pub overridden: Vec<[u8; 32]>,        // Token hashes of overridden voters
    pub total_submitted: usize,           // Total ballots before de-dup
    pub total_after_dedup: usize,         // Total ballots after de-dup
}

pub struct FinalBallot {
    pub token_hash: [u8; 32],
    pub entry_index: u64,
    pub ciphertexts: Vec<Vec<Ciphertext>>, // Deserialized ballot matrix
}

/// Resolves re-votes by keeping only the last ballot per token.
pub fn deduplicate_ballots(entries: &[Entry]) -> DedupResult {
    // 1. Group all "ballot" entries by token_hash
    // 2. For each token: keep only the entry with the highest index
    // 3. Collect all "override" entries
    // 4. Remove ballots whose token_hash appears in the override set
    // 5. Return de-duplicated set
    //
    // Verification: anyone with the full BB can independently compute
    // this exact result.
}
```

## Phase 2: Homomorphic Aggregation

### Aggregation (`aggregate/aggregate.rs`)

```rust
/// Element-wise sum of all encrypted ballots for a MIR.
pub struct AggregationResult {
    pub mir_id: u32,
    pub rows: usize,
    pub cols: usize,
    pub aggregated: Vec<Vec<Ciphertext>>,  // Matrix of summed ciphertexts
    pub ballot_count: usize,               // Number of ballots included
    pub ballot_hashes: Vec<[u8; 32]>,      // Hashes of all included ballots
}

/// Computes the element-wise homomorphic sum of all ballots.
pub fn aggregate_ballots(
    ballots: &[FinalBallot],
    rows: usize,
    cols: usize,
) -> AggregationResult {
    // Initialize result matrix with identity ciphertexts:
    //   Enc(0) = (Identity, Identity)
    //
    // For each ballot, for each cell [row][col]:
    //   result[row][col] = add(&result[row][col], &ballot.ciphertexts[row][col])
    //
    // result[row][col] now encrypts the COUNT of ballots that had a 1 in that cell.
}
```

## Phase 3: Threshold Decryption Ceremony

### Partial Decryption (`decrypt/partial.rs`)

```rust
/// One trustee's contribution to decryption.
pub struct PartialDecryptionResult {
    pub trustee_index: u32,
    pub mir_id: u32,
    pub partials: Vec<Vec<AffinePoint>>,        // D_i[row][col] = share_i · C1[row][col]
    pub proofs: Vec<Vec<DlogEqualityProof>>,     // Proof that D_i is correct
}

/// Computes a trustee's partial decryption of the aggregated ciphertext matrix.
pub fn compute_partial_decryption(
    trustee_index: u32,
    secret_share: &Scalar,
    public_share: &AffinePoint,  // = secret_share · G
    agg: &AggregationResult,
) -> PartialDecryptionResult {
    // For each cell [row][col]:
    //   C1 = agg.aggregated[row][col].c1
    //   D_i = secret_share · C1
    //   Proof: prove_dlog_equality(secret_share, G, C1, public_share, D_i)
}
```

### Combining Partials (`decrypt/combine.rs`)

```rust
/// Decrypted tally for a MIR.
pub struct CombineResult {
    pub mir_id: u32,
    pub rows: usize,
    pub cols: usize,
    pub plaintext_points: Vec<Vec<AffinePoint>>, // m·G for each cell
    pub counts: Vec<Vec<u64>>,                    // Decrypted vote counts
}

/// Combines t-of-n partial decryptions using Lagrange interpolation.
pub fn combine_partial_decryptions(
    partials: &[PartialDecryptionResult], // Exactly threshold count
    agg: &AggregationResult,
    threshold: u32,
) -> CombineResult {
    // 1. Compute Lagrange coefficients for participating trustee indices
    //    λ_j = Π_{k≠j} (iₖ / (iₖ - iⱼ)) mod n
    //
    // 2. For each cell [row][col]:
    //    Combined D = Σ λ_j · D_j[row][col]
    //    Plaintext point = C2[row][col] - Combined D = m·G
    //
    // 3. Solve DLOG for each cell: m = solve_dlog(&point, max_votes)
}

/// Computes Lagrange coefficients for a set of trustee indices.
pub fn lagrange_coefficients(indices: &[u32]) -> HashMap<u32, Scalar> {
    // λ_j = Π_{k∈S, k≠j} (k / (k - j)) mod n
}
```

### DLOG Solver (`decrypt/dlog.rs`)

```rust
use std::collections::HashMap;

/// Baby-step giant-step DLOG solver.
/// Precomputes a table for O(√max_m) time and space.
pub struct DlogSolver {
    table: HashMap<[u8; 65], u64>,  // point bytes → index
    giant_step: AffinePoint,         // (-s)·G
    s: u64,                          // step size = ceil(√max_m)
}

impl DlogSolver {
    /// Precomputes the baby step table.
    pub fn new(max_m: u64) -> Self {
        // s = ceil(√max_m)
        // For j = 0 to s-1: table[j·G.to_bytes()] = j
        // giant_step = (-s)·G
    }

    /// Finds m such that m·G = target.
    pub fn solve(&self, target: &AffinePoint) -> Option<u64> {
        // γ = target
        // For i = 0 to s-1:
        //   if γ in table: return Some(i*s + table[γ])
        //   γ = γ + giant_step
        // None (m > max_m)
    }
}
```

### Ceremony Orchestrator (`decrypt/ceremony.rs`)

```rust
/// Orchestrates the full threshold decryption process.
pub struct Ceremony {
    pub election_pk: AffinePoint,
    pub threshold: u32,
    pub total_trustees: u32,
    pub public_shares: HashMap<u32, AffinePoint>,
    pub aggregations: HashMap<u32, AggregationResult>, // mir_id → aggregation
}

impl Ceremony {
    /// Executes the full decryption ceremony.
    /// In production this is distributed; in the demo, simulated locally.
    pub fn run(
        &self,
        trustee_shares: &HashMap<u32, Scalar>, // For demo: all shares available locally
    ) -> HashMap<u32, CombineResult> {
        // For each MIR:
        //   1. Select first `threshold` trustees
        //   2. Each computes partial decryption with proof
        //   3. Verify all proofs
        //   4. Combine partial decryptions
        //   5. Solve DLOGs for all cells
        //   6. Validate: sum across all cells == ballot_count
    }
}

/// Independently verifies all partial decryption proofs.
pub fn verify_ceremony(
    partials: &HashMap<u32, Vec<PartialDecryptionResult>>,
    public_shares: &HashMap<u32, AffinePoint>,
    aggregations: &HashMap<u32, AggregationResult>,
) -> Result<(), String> {
    // For each MIR, for each trustee's partial, for each cell:
    //   verify the DLOG equality proof
}
```

## Phase 4: Bulgarian Seat Allocation

### Hare-Niemeyer Method (`allocate/hare.rs`)

```rust
/// National-level result for one party.
pub struct PartyResult {
    pub party_number: u32,
    pub total_votes: u64,
    pub vote_share: f64,
    pub passed_threshold: bool, // >= 4% nationally
    pub total_seats: u32,
}

/// Per-MIR results.
pub struct MIRResult {
    pub mir_id: u32,
    pub total_ballots: u64,
    pub party_votes: HashMap<u32, u64>,    // party_number → votes
    pub seat_allocation: HashMap<u32, u32>, // party_number → seats
}

/// Filters parties passing the 4% national threshold.
pub fn national_threshold(
    party_votes_by_mir: &HashMap<u32, HashMap<u32, u64>>,
    threshold: f64,
) -> Vec<u32> {
    // 1. Sum votes nationally for each party
    // 2. Compute total valid votes
    // 3. Return party numbers where total_votes / total_valid >= threshold
}

/// Allocates seats in one MIR using the Hare-Niemeyer largest remainder method.
pub fn hare_niemeyer(
    mir_seats: u32,
    party_votes: &HashMap<u32, u64>, // Only qualifying parties
) -> HashMap<u32, u32> {
    // Hare quota = total_votes / mir_seats
    //
    // For each qualifying party:
    //   automatic_seats = floor(party_votes / hare_quota)
    //   remainder = party_votes - (automatic * hare_quota)
    //
    // Remaining seats distributed to parties with largest remainders
    // Tie-breaking: by total votes, then by party ballot number (lower wins)
}
```

### Preference Threshold (`allocate/preference.rs`)

```rust
/// Preference vote results for one candidate.
pub struct CandidateResult {
    pub position: u32,
    pub preference_votes: u64,
    pub party_total: u64,
    pub preference_pct: f64,
    pub meets_threshold: bool, // >= 7%
}

/// Determines which candidates meet the 7% preference threshold.
pub fn evaluate_preferences(
    tally_cells: &[Vec<u64>],  // Decrypted count matrix [row][col]
    spec: &BallotSpec,
    threshold: f64,            // 0.07
) -> HashMap<u32, Vec<CandidateResult>> {
    // For each party (column):
    //   party_total = sum of entire column
    //   For each candidate position (rows 1..N):
    //     pref_votes = tally_cells[position][party_col]
    //     pct = pref_votes / party_total
    //     meets = pct >= 0.07
    //
    // Elected candidates:
    //   1. Those meeting 7% threshold, ordered by preference votes (desc)
    //   2. Remaining seats filled by original list order
}
```

### Results Compilation (`allocate/results.rs`)

```rust
/// The complete, verifiable election result.
pub struct ElectionResults {
    pub election_id: String,
    pub total_valid_ballots: u64,
    pub total_overridden: u64,
    pub total_superseded: u64,

    pub national_party_results: Vec<PartyResult>,
    pub qualifying_parties: Vec<u32>,

    pub mir_results: HashMap<u32, MIRResult>,
    pub preference_results: HashMap<u32, HashMap<u32, Vec<CandidateResult>>>,
    pub elected_mps: Vec<ElectedMP>,

    pub verification_bundle: VerificationBundle,
}

pub struct ElectedMP {
    pub name: String,
    pub party: String,
    pub mir_id: u32,
    pub mir_name: String,
    pub position: u32,
    pub by_preference: bool,
    pub preference_votes: u64,
}

pub struct VerificationBundle {
    pub aggregated_ciphertexts: HashMap<u32, Vec<Vec<Ciphertext>>>,
    pub partial_decryptions: HashMap<u32, Vec<PartialDecryptionResult>>,
    pub decrypted_counts: HashMap<u32, Vec<Vec<u64>>>,
    pub dedup_audit: HashMap<u32, DedupResult>,
}
```

## Report Generation (`report/generate.rs`)

```rust
/// Creates human-readable and machine-readable reports.
pub fn generate_report(results: &ElectionResults) -> Report {
    // Generates:
    // 1. JSON: full machine-readable results with verification bundle
    // 2. Text: national summary table
    // 3. Text: per-MIR breakdown
    // 4. Text: preference results
    // 5. Text: final MP list (240 members)
}
```

## Trustee Tool

### CLI for Trustees (`packages/trustee-tool/src/main.rs`)

```bash
# Commands:
glasuvai-trustee keygen    # Participate in DKG ceremony
glasuvai-trustee decrypt   # Compute partial decryption of aggregated ballots
glasuvai-trustee verify    # Verify other trustees' partial decryptions
```

- **keygen**: Input: trustee index, total, threshold. Output: secret share (saved encrypted to file), public share, Feldman commitments.
- **decrypt**: Input: secret share file, aggregation data (from BB). Output: partial decryption with proofs.
- **verify**: Input: all partial decryptions. Output: verification report.

## Implementation Steps

### Step 1: De-Duplication and Override Resolution

Implement `resolve/dedup.rs` and `resolve/override.rs`.

**Test**: 100 ballots, 10 re-votes, 5 overrides → verify final set is 85.

### Step 2: Homomorphic Aggregation

Implement `aggregate/aggregate.rs`.

**Test**: Aggregate 3 known ballots → decrypt → verify correct per-cell counts.

### Step 3: DLOG Solver

Implement `decrypt/dlog.rs`.

**Test**: Solve for m = 0, 1, 42, 1000, 100000. Benchmark for max_m = 10,000,000 (< 5 seconds).

### Step 4: Partial Decryption

Implement `decrypt/partial.rs`.

**Test**: One trustee partial → proof verifies. Wrong share → proof fails.

### Step 5: Combining Partials

Implement `decrypt/combine.rs`.

**Test**: 5-of-9 → correct plaintext. 4-of-9 → fails.

### Step 6: Ceremony Orchestrator

Implement `decrypt/ceremony.rs`. Demo runs all trustee shares locally.

**Test**: Full ceremony → all proofs verify → correct counts.

### Step 7: Hare-Niemeyer Seat Allocation

Implement `allocate/hare.rs`.

**Test with real data**: Use published 51st NA results for MIR 23, verify same seat allocation as official ЦИК results.

### Step 8: Preference Threshold

Implement `allocate/preference.rs`.

**Test**: 7% boundary correctly evaluated — 7.5% passes, 6.5% fails.

### Step 9: Results Compilation and Reporting

Implement `allocate/results.rs` and `report/generate.rs`.

### Step 10: Tally CLI

Wire together in `main.rs`:

```bash
cargo run -p glasuvai-tally -- --bb-url http://localhost:8080 --mir 23 \
  --trustee-shares shares/ --output results/
```

### Step 11: Trustee Tool CLI

Implement `packages/trustee-tool/src/main.rs`.

## Acceptance Criteria

- [ ] De-duplication correctly keeps latest ballot per token and removes overridden
- [ ] Homomorphic aggregation: sum of 1000 random ballots decrypts to correct counts
- [ ] DLOG solver: solves for m up to 10,000,000 in < 5 seconds
- [ ] Partial decryption: proofs verify for all cells
- [ ] Combining: any 5-of-9 subset recovers correct plaintext; 4-of-9 fails
- [ ] Lagrange coefficients computed correctly (test against known values)
- [ ] Hare-Niemeyer: matches known real election results (ЦИК data for 51st NA)
- [ ] Preference threshold: 7% boundary correctly evaluated
- [ ] Full tally pipeline: 50 demo ballots → correct seat allocation
- [ ] Sum of all decrypted cells per MIR == number of final ballots (sanity check)
- [ ] Verification bundle contains everything needed for independent audit
- [ ] Trustee tool: keygen, decrypt, and verify subcommands work
- [ ] Reports generated in JSON and human-readable formats
- [ ] `cargo tree -p glasuvai-tally` shows only allowed deps (crypto + serde)
- [ ] `cargo test -p glasuvai-tally` passes all tests
