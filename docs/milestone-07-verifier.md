# Milestone 7: Independent Verifier

## Goal

Build a standalone CLI tool that takes only public data (Bulletin Board export + election configuration) and independently verifies the entire election end-to-end. The verifier uses the same crypto primitives from `glasuvai-crypto` but trusts **nothing** from the other servers — it recomputes everything from scratch. Anyone can run this tool and confirm the election result is correct.

This is the **proof that the system is verifiable without doubt**. If the verifier passes, the election result is mathematically guaranteed to be correct, regardless of whether any server was compromised.

## Prerequisites

- **M2**: All crypto primitives (ZKP verification, DLOG equality verification)
- **M3**: Ballot deserialization, ZeroOne proof verification, ExactlyOne proof verification
- **M4**: BB data format, hash chain verification, Merkle tree verification
- **M6**: Aggregation verification, partial decryption proof verification, Hare-Niemeyer

## Deliverables

```
packages/verifier/
  Cargo.toml                # name = "glasuvai-verifier"
  src/
    main.rs                 # CLI binary
    lib.rs                  # Library root
    pipeline/
      mod.rs
      pipeline.rs           # Verification pipeline orchestrator
    checks/
      mod.rs
      chain.rs              # Hash chain integrity verification
      merkle.rs             # Merkle tree verification
      ballots.rs            # Ballot ZKP verification (all ballots)
      tokens.rs             # Token signature verification
      dedup.rs              # Re-vote de-duplication verification
      aggregation.rs        # Homomorphic aggregation verification
      decryption.rs         # Partial decryption proof verification
      tally.rs              # Final tally arithmetic verification
      seats.rs              # Seat allocation verification (Hare-Niemeyer)
    report/
      mod.rs
      report.rs             # Verification report generation
```

```toml
# packages/verifier/Cargo.toml
[package]
name = "glasuvai-verifier"
version = "0.1.0"
edition = "2021"

[dependencies]
glasuvai-crypto = { path = "../crypto" }   # Crypto primitives — zero transitive deps
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rayon = "1"                                 # Parallel ballot verification
```

## What the Verifier Checks

The verifier performs **10 independent checks**, each of which can pass or fail independently. All 10 must pass for the election to be verified.

```
┌────────────────────────────────────────────────────────────────────┐
│                    VERIFICATION PIPELINE                            │
│                                                                      │
│  Input: BB export (all entries) + election config + public keys      │
│                                                                      │
│  CHECK 1: HASH CHAIN INTEGRITY                                      │
│    For every consecutive pair of entries:                             │
│    H(entry[i]) == entry[i+1].prev_hash                              │
│    Result: chain is append-only and untampered                      │
│                                                                      │
│  CHECK 2: MERKLE TREE CONSISTENCY                                   │
│    Rebuild Merkle tree from all entry hashes                        │
│    Verify root matches published root                               │
│    Verify a sample of inclusion proofs                              │
│                                                                      │
│  CHECK 3: BALLOT VALIDITY (ALL BALLOTS)                             │
│    For every ballot entry on the BB:                                 │
│    a. Deserialize ciphertext matrix                                 │
│    b. Verify matrix dimensions match MIR's BallotSpec               │
│    c. Verify EVERY per-cell ZeroOneProof                            │
│    d. Verify ExactlyOneProof (sum of all cells = 1)                 │
│    Result: every ballot encrypts a valid choice                     │
│                                                                      │
│  CHECK 4: TOKEN SIGNATURES                                          │
│    For every ballot entry:                                           │
│    Verify RSA blind signature on token using MIR public key          │
│    Result: every ballot has a valid token from the IdP               │
│                                                                      │
│  CHECK 5: DE-DUPLICATION CORRECTNESS                                │
│    Recompute the de-duplicated ballot set:                           │
│    - Group by token_hash, keep latest per token                      │
│    - Remove overridden tokens                                        │
│    Compare with the published "final ballot set"                    │
│    Result: de-duplication was done correctly                        │
│                                                                      │
│  CHECK 6: HOMOMORPHIC AGGREGATION                                   │
│    Recompute the element-wise sum of all final ballots              │
│    Compare with the published aggregated ciphertexts                │
│    Result: aggregation was computed correctly                       │
│                                                                      │
│  CHECK 7: PARTIAL DECRYPTION PROOFS                                 │
│    For each trustee's partial decryption:                            │
│    Verify the DLOGEquality proof for every cell                     │
│    Result: each trustee used their correct secret share             │
│                                                                      │
│  CHECK 8: DECRYPTION COMBINATION                                    │
│    Recompute the Lagrange interpolation combining partial            │
│    decryptions. Verify the resulting plaintext points.              │
│    Recompute DLOG for each cell.                                    │
│    Compare with published tally counts.                             │
│    Result: decryption was performed correctly                       │
│                                                                      │
│  CHECK 9: TALLY ARITHMETIC                                          │
│    For each MIR:                                                     │
│    a. Sum of all cells == number of final ballots for this MIR      │
│    b. Party total == sum of party's column                          │
│    c. Cross-check national totals                                   │
│    Result: arithmetic is consistent                                 │
│                                                                      │
│  CHECK 10: SEAT ALLOCATION                                          │
│    Recompute:                                                        │
│    a. National 4% threshold filtering                                │
│    b. Hare-Niemeyer per MIR for qualifying parties                  │
│    c. 7% preference threshold per MIR per party                     │
│    d. Final MP list                                                  │
│    Compare with published results.                                  │
│    Result: seat allocation matches declared result                  │
│                                                                      │
│  OUTPUT: Verification report (pass/fail per check, details)         │
└────────────────────────────────────────────────────────────────────┘
```

## Data Structures

### Verification Pipeline (`pipeline/pipeline.rs`)

```rust
use std::time::{Duration, Instant};

/// Everything the verifier needs.
pub struct VerificationInput {
    // BB data
    pub bb_entries: Vec<Entry>,
    pub merkle_root: [u8; 32],

    // Election configuration
    pub election_config: ElectionConfig,
    pub ballot_specs: HashMap<u32, BallotSpec>,

    // Public keys
    pub election_pk: AffinePoint,
    pub mir_keys: HashMap<u32, (U3072, U3072)>,    // (n, e) per MIR
    pub trustee_public_shares: HashMap<u32, AffinePoint>,

    // Published results (what we're verifying)
    pub dedup_result: HashMap<u32, DedupResult>,
    pub aggregations: HashMap<u32, AggregationResult>,
    pub partial_decrypts: HashMap<u32, Vec<PartialDecryptionResult>>,
    pub declared_counts: HashMap<u32, Vec<Vec<u64>>>,
    pub declared_results: ElectionResults,
}

/// Outcome of one verification check.
pub struct CheckResult {
    pub name: String,
    pub passed: bool,
    pub duration: Duration,
    pub details: String,
    pub errors: Vec<String>,
    pub items_checked: u64,
}

/// Complete output of the verifier.
pub struct VerificationReport {
    pub election_id: String,
    pub all_passed: bool,
    pub checks: Vec<CheckResult>,
    pub summary: String,
}

/// Executes all verification checks.
pub fn run_pipeline(input: &VerificationInput) -> VerificationReport {
    let checks: Vec<fn(&VerificationInput) -> CheckResult> = vec![
        check_hash_chain,
        check_merkle_tree,
        check_ballot_validity,
        check_token_signatures,
        check_deduplication,
        check_aggregation,
        check_partial_decryptions,
        check_decryption_combination,
        check_tally_arithmetic,
        check_seat_allocation,
    ];

    let mut report = VerificationReport {
        election_id: input.election_config.election_id.to_string(),
        all_passed: true,
        checks: Vec::new(),
        summary: String::new(),
    };

    for check in &checks {
        let result = check(input);
        if !result.passed {
            report.all_passed = false;
        }
        report.checks.push(result);
    }

    report
}
```

### Individual Checks

```rust
// checks/chain.rs
pub fn check_hash_chain(input: &VerificationInput) -> CheckResult {
    // Verify every entry's hash and chain linkage
    // O(N) — scans all entries once
}

// checks/merkle.rs
pub fn check_merkle_tree(input: &VerificationInput) -> CheckResult {
    // Rebuild Merkle tree from scratch
    // Compare root with published root
}

// checks/ballots.rs
pub fn check_ballot_validity(input: &VerificationInput) -> CheckResult {
    // For every ballot entry:
    //   1. Deserialize
    //   2. Check dimensions against BallotSpec
    //   3. Verify all per-cell ZeroOneProofs
    //   4. Verify ExactlyOneProof
    //
    // MOST EXPENSIVE check — parallelized with rayon
}

// checks/tokens.rs
pub fn check_token_signatures(input: &VerificationInput) -> CheckResult {
    // Verify RSA blind signature on every ballot's token
}

// checks/dedup.rs
pub fn check_deduplication(input: &VerificationInput) -> CheckResult {
    // Recompute de-duplication from scratch
    // Compare with published DedupResult
}

// checks/aggregation.rs
pub fn check_aggregation(input: &VerificationInput) -> CheckResult {
    // Recompute element-wise ciphertext sum
    // Compare each cell with published aggregation
}

// checks/decryption.rs
pub fn check_partial_decryptions(input: &VerificationInput) -> CheckResult {
    // For each trustee's partial decryption:
    //   For each cell: verify DlogEquality proof
}

// checks/tally.rs
pub fn check_tally_arithmetic(input: &VerificationInput) -> CheckResult {
    // Recompute Lagrange interpolation + DLOG
    // Compare with declared counts
    // Cross-check: sum of cells per MIR == ballot count
}

// checks/seats.rs
pub fn check_seat_allocation(input: &VerificationInput) -> CheckResult {
    // Recompute Hare-Niemeyer + preference thresholds
    // Compare with declared results
}
```

## Performance: Parallel Ballot Verification

The ballot ZKP check (Check 3) is by far the most expensive. For the demo (50 ballots × 434 cells ≈ 21,700 proofs), this completes in seconds. For a real election (800K+ ballots), parallelization is essential.

`rayon` is used for data-parallel ballot verification:

```rust
use rayon::prelude::*;

pub fn check_ballot_validity(input: &VerificationInput) -> CheckResult {
    let start = Instant::now();

    let ballot_entries: Vec<_> = input.bb_entries.iter()
        .filter(|e| e.entry_type == EntryType::Ballot)
        .collect();

    // Parallel verification across all ballots
    let errors: Vec<String> = ballot_entries.par_iter()
        .filter_map(|entry| {
            match verify_one_ballot(entry, input) {
                Ok(()) => None,
                Err(e) => Some(format!("Entry {}: {}", entry.index, e)),
            }
        })
        .collect();

    CheckResult {
        name: "ballot_validity".into(),
        passed: errors.is_empty(),
        duration: start.elapsed(),
        details: format!("{} ballots verified", ballot_entries.len()),
        errors,
        items_checked: ballot_entries.len() as u64,
    }
}
```

## CLI Interface

```bash
# Full verification of an election
cargo run -p glasuvai-verifier -- verify \
    --bb-export /path/to/bb-export.json \
    --config /path/to/election-config.json \
    --results /path/to/declared-results.json \
    --output /path/to/verification-report.json

# Verify a single voter's receipt
cargo run -p glasuvai-verifier -- receipt \
    --bb-url https://bb.glasuvai.bg \
    --receipt-hash "a3f7-c9b2-..."

# Verify BB chain integrity only (quick check)
cargo run -p glasuvai-verifier -- chain \
    --bb-url https://bb.glasuvai.bg

# Verbose mode with per-ballot details
cargo run -p glasuvai-verifier -- verify --verbose \
    --bb-export bb.json --config config.json --results results.json
```

### Output Example

```
═══════════════════════════════════════════════════════════════
  GLASUVAI Election Verification Report
  Election: Избори за 51-о Народно събрание (bg-na51-2024)
  Verified: 2024-10-28T02:15:33Z
═══════════════════════════════════════════════════════════════

  Check  1/10: Hash Chain Integrity .............. ✓ PASS
    847,293 entries verified, chain consistent
    Duration: 1.2s

  Check  2/10: Merkle Tree Consistency ........... ✓ PASS
    Root matches: a3f7c9b2e1...
    Duration: 0.8s

  Check  3/10: Ballot Validity (ZKPs) ............ ✓ PASS
    847,000 ballots verified (434 cells each)
    367,598,000 individual ZKP checks passed
    Duration: 4m 23s (parallelized via rayon)

  Check  4/10: Token Signatures .................. ✓ PASS
    847,000 RSA signatures verified across 32 MIR keys
    Duration: 12.5s

  Check  5/10: De-duplication .................... ✓ PASS
    Re-votes resolved: 23,451 superseded entries
    Overrides: 1,847 tokens excluded
    Final ballot count: 821,702
    Duration: 0.3s

  Check  6/10: Homomorphic Aggregation ........... ✓ PASS
    32 MIR aggregations recomputed and matched
    Duration: 45s

  Check  7/10: Partial Decryption Proofs ......... ✓ PASS
    5 trustees × 32 MIRs × ~500 cells = ~80,000 proofs verified
    Duration: 28s

  Check  8/10: Decryption Combination ............ ✓ PASS
    All Lagrange interpolations correct
    All DLOG solutions match declared counts
    Duration: 15s

  Check  9/10: Tally Arithmetic .................. ✓ PASS
    Per-MIR totals consistent
    National totals consistent
    Duration: 0.1s

  Check 10/10: Seat Allocation ................... ✓ PASS
    4% threshold: 6 qualifying parties
    240 seats allocated via Hare-Niemeyer
    Preference thresholds: 47 candidates elected by preference
    Duration: 0.1s

═══════════════════════════════════════════════════════════════
  RESULT: ALL CHECKS PASSED ✓
  The declared election result is mathematically correct.
═══════════════════════════════════════════════════════════════
  Total verification time: 5m 41s
  BB entries processed: 847,293
  Cryptographic proofs verified: 367,691,847
═══════════════════════════════════════════════════════════════
```

## Implementation Steps

### Step 1: Pipeline Orchestrator

Implement `pipeline/pipeline.rs` with the check sequence and reporting.

### Step 2: Hash Chain Check

Implement `checks/chain.rs`. Reuse `verify_chain_segment` from M4.

### Step 3: Merkle Tree Check

Implement `checks/merkle.rs`. Rebuild tree from entry hashes, compare root.

### Step 4: Ballot ZKP Check

Implement `checks/ballots.rs` with `rayon` parallelism for verification.

### Step 5: Token Signature Check

Implement `checks/tokens.rs`. Straightforward RSA verification.

### Step 6: De-duplication Check

Implement `checks/dedup.rs`. Recompute from BB data, compare.

### Step 7: Aggregation Check

Implement `checks/aggregation.rs`. Recompute homomorphic sums.

### Step 8: Decryption Proof Check

Implement `checks/decryption.rs`. Verify all DLOG equality proofs.

### Step 9: Tally Arithmetic Check

Implement `checks/tally.rs`. Recompute Lagrange + DLOG.

### Step 10: Seat Allocation Check

Implement `checks/seats.rs`. Recompute Hare-Niemeyer + preferences.

### Step 11: CLI

Wire everything in `main.rs`.

## Acceptance Criteria

- [ ] Verifier passes on a correctly computed demo election
- [ ] Verifier fails Check 1 when a single entry hash is tampered
- [ ] Verifier fails Check 3 when a single ballot ZKP is invalid
- [ ] Verifier fails Check 4 when a token signature is forged
- [ ] Verifier fails Check 5 when de-duplication is done incorrectly
- [ ] Verifier fails Check 6 when aggregation has a wrong ciphertext
- [ ] Verifier fails Check 7 when a partial decryption proof is forged
- [ ] Verifier fails Check 10 when seat allocation uses wrong method
- [ ] Each check produces a clear, human-readable explanation
- [ ] CLI outputs both JSON (machine) and text (human) reports
- [ ] Receipt verification: voter can check their ballot hash on the BB
- [ ] Full pipeline completes for 50-ballot demo in < 30 seconds
- [ ] `cargo tree -p glasuvai-verifier` shows only allowed deps (crypto + serde + rayon)
- [ ] `cargo test -p glasuvai-verifier` passes all tests
