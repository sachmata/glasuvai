# Milestone 3: Ballot Encoding & Encryption

## Goal

Implement the ballot matrix structure that maps a voter's choice (party + optional preference candidate) to a matrix of ElGamal ciphertexts, along with the zero-knowledge proof that exactly one cell contains 1 and all others contain 0. This milestone also implements the Benaloh challenge (cast-or-audit) and receipt generation.

## Prerequisites

- **M1**: Election types (`BallotSpec`, `Party`, `Candidate`)
- **M2**: ElGamal encryption, `ZeroOneProof`, domain-separated hashing

## Deliverables

```
packages/crypto/
  src/
    ballot/
      mod.rs              # Module declarations
      encode.rs           # Plaintext ballot matrix from voter choice
      encrypt.rs          # Encrypt ballot matrix cell-by-cell
      exactly_one.rs      # Disjunctive proof: exactly one cell = 1
      challenge.rs        # Benaloh challenge (audit/spoil)
      receipt.rs          # Receipt hash computation
      serialize.rs        # Ballot serialization (for BB submission)
```

All code lives in the `glasuvai-crypto` crate — zero external dependencies. Tests use `#[cfg(test)]` modules within each file.

## Ballot Matrix Structure

### Encoding Scheme

For a MIR with P registered parties and each party having up to Cₚ candidates on their list:

```
Matrix layout (rows × columns):

                    Party1      Party2      ...     PartyP
Row 0 (NoPref):     E(0/1)      E(0/1)     ...     E(0/1)
Row 1 (Cand 1):     E(0/1)      E(0/1)     ...     E(0/1)
Row 2 (Cand 2):     E(0/1)      E(0/1)     ...     E(0/1)
  ...
Row Cmax (Cand Cmax): E(0/1)    E(0/1)     ...     E(0/1)
```

- **Rows**: Row 0 = "vote for party with no preference", Rows 1..Cmax = specific candidate
- **Columns**: One per registered party, ordered by ballot number
- **Exactly ONE cell** in the entire matrix is E(1); all others are E(0)
- **Sparse optimization**: Parties with fewer candidates than Cmax still have their shorter columns padded with E(0) cells. These padding cells are never E(1) (enforced by the proof).

### Example: MIR 23 Demo

MIR 23 has ~14 registered parties and a maximum of ~30 candidates per list.
Matrix size: 14 columns × 31 rows = 434 cells.
Each cell: ElGamal ciphertext (64 bytes) + ZeroOneProof (~128 bytes).
Total ballot: ~434 × 192 ≈ 83 KB.

## Data Structures

### Plaintext and Encrypted Ballots

```rust
use crate::{AffinePoint, Ciphertext, Scalar, ZeroOneProof, DlogEqualityProof};
use crate::election::types::{BallotSpec};

/// What the voter selected.
pub struct VoterChoice {
    pub party_number: u32,   // Which party (ballot number)
    pub candidate_pos: u32,  // 0 = no preference, 1..N = candidate position
}

/// Unencrypted ballot matrix (used only client-side, never sent).
pub struct PlaintextBallot {
    pub mir_id: u32,
    pub rows: usize,           // 1 + max_candidates
    pub cols: usize,           // number of parties
    pub matrix: Vec<Vec<u8>>,  // matrix[row][col] ∈ {0, 1}, exactly one cell = 1
    pub choice: VoterChoice,
}

/// Encrypted ballot ready for submission.
pub struct EncryptedBallot {
    pub mir_id: u32,
    pub rows: usize,
    pub cols: usize,
    pub ciphertexts: Vec<Vec<Ciphertext>>,    // matrix[row][col]
    pub proofs: Vec<Vec<ZeroOneProof>>,        // per-cell proof that cell ∈ {0,1}
    pub sum_proof: ExactlyOneProof,            // proof that exactly one cell = 1
    pub randomness: Vec<Vec<Scalar>>,          // encryption randomness (client-side for Benaloh)
}

/// Proves that the sum of all cells = 1.
/// Strategy: per-cell 0/1 proofs + sum proof showing the homomorphic sum
/// of ALL ciphertexts encrypts exactly 1. Since each cell is 0 or 1 and
/// the sum is 1, exactly one cell must be 1.
pub struct ExactlyOneProof {
    /// The homomorphic sum of all ciphertexts
    pub sum_ciphertext: Ciphertext,
    /// Proof that sum_ciphertext encrypts 1 (Chaum-Pedersen proof)
    /// log_G(sum_C1) == log_{PK}(sum_C2 - G) with witness sum_r
    pub proof: DlogEqualityProof,
}

/// Complete package sent to the voting server.
pub struct BallotSubmission {
    pub mir_id: u32,
    pub token: Vec<u8>,             // blind-signed token
    pub token_signature: Vec<u8>,   // RSA signature from IdP
    pub ciphertexts: Vec<Vec<[Vec<u8>; 2]>>, // serialized ciphertext matrix
    pub cell_proofs: Vec<Vec<Vec<u8>>>,      // serialized per-cell ZK proofs
    pub sum_proof: Vec<u8>,         // serialized exactly-one proof
    pub receipt_hash: [u8; 32],     // SHA-256 of encrypted ballot
}
```

## Implementation Steps

### Step 1: Ballot Encoding (`encode.rs`)

```rust
/// Creates a plaintext ballot matrix from a voter's choice.
pub fn encode_ballot(
    spec: &BallotSpec,
    choice: &VoterChoice,
) -> Result<PlaintextBallot, &'static str> {
    // Validate:
    //   - choice.party_number exists in spec.parties
    //   - choice.candidate_pos == 0 (no preference) OR
    //     choice.candidate_pos ∈ [1, len(candidates for this party)]
    //
    // Create matrix of zeros
    // Set matrix[choice.candidate_pos][party_column_index] = 1
    //
    // party_column_index is the index into spec.parties (not the ballot number)
    // Column ordering is by ballot number (sorted)
}
```

**Test**: Encode choice (Party 3, Candidate 5) → verify matrix has exactly one 1 at the correct position. Encode (Party 8, NoPreference) → verify 1 is in row 0 of Party 8's column.

### Step 2: Ballot Encryption (`encrypt.rs`)

```rust
/// Encrypts a plaintext ballot under the election public key.
pub fn encrypt_ballot(
    pk: &AffinePoint,
    pt: &PlaintextBallot,
) -> EncryptedBallot {
    // For each cell [row][col]:
    //   1. Generate fresh randomness r
    //   2. Encrypt: ct = encrypt(pk, matrix[row][col], &r)
    //   3. Generate ZeroOneProof for this cell
    //   4. Store r in randomness matrix (for Benaloh challenge)
    //
    // Compute sum_proof:
    //   1. Sum all ciphertexts homomorphically
    //   2. Sum all randomness: sum_r = Σ r[row][col] mod n
    //   3. The sum encrypts 1 (exactly one cell is 1)
    //   4. Prove: log_G(sum.c1) == log_{PK}(sum.c2 - G) with witness sum_r
}
```

**Test**: Encrypt ballot → decrypt each cell → verify matches plaintext. Verify all per-cell proofs. Verify sum proof. Verify homomorphic sum decrypts to 1.

### Step 3: Exactly-One Proof (`exactly_one.rs`)

```rust
/// Proves that the homomorphic sum of all ciphertexts encrypts 1.
pub fn prove_exactly_one(
    pk: &AffinePoint,
    ciphertexts: &[Vec<Ciphertext>],
    randomness: &[Vec<Scalar>],
) -> ExactlyOneProof {
    // 1. Compute sum ciphertext: sum = Σ_ij ct[i][j]
    // 2. Compute sum randomness: sum_r = Σ_ij r[i][j] mod n
    // 3. sum encrypts 1 with randomness sum_r:
    //    sum.c1 = sum_r·G, sum.c2 = G + sum_r·PK
    // 4. Prove DLOG equality: log_G(sum.c1) == log_{PK}(sum.c2 - G)
    //    Witness: sum_r
}

/// Verifies the exactly-one proof.
pub fn verify_exactly_one(
    pk: &AffinePoint,
    ciphertexts: &[Vec<Ciphertext>],
    proof: &ExactlyOneProof,
) -> bool {
    // 1. Recompute sum ciphertext from the matrix
    // 2. Verify it matches proof.sum_ciphertext
    // 3. Verify the DLOG equality proof
}
```

### Step 4: Benaloh Challenge (`challenge.rs`)

```rust
/// Data revealed when a voter audits a ballot.
/// The ballot is SPOILED after this — voter must re-encrypt.
pub struct ChallengeResponse {
    pub plaintext: PlaintextBallot,
    pub randomness: Vec<Vec<Scalar>>,
}

/// Reveals the ballot's secrets for verification.
pub fn respond_to_challenge(
    eb: &EncryptedBallot,
    pt: &PlaintextBallot,
) -> ChallengeResponse {
    // Return the plaintext and randomness.
    // Any observer with this data can:
    //   1. Re-encrypt from scratch using the same randomness
    //   2. Verify ciphertexts match
    //   3. Verify plaintext is a valid ballot (exactly one 1)
    //   → Proves the voting app encrypted correctly
}

/// Checks that a challenge response matches the encrypted ballot.
pub fn verify_challenge(
    pk: &AffinePoint,
    eb: &EncryptedBallot,
    cr: &ChallengeResponse,
) -> bool {
    // 1. For each cell: re-encrypt cr.plaintext[row][col] with cr.randomness[row][col]
    // 2. Verify ciphertext matches eb.ciphertexts[row][col]
    // 3. Verify plaintext has exactly one 1
}
```

### Step 5: Receipt Generation (`receipt.rs`)

```rust
/// Computes the receipt hash for an encrypted ballot.
/// Deterministic — same ciphertexts always produce the same receipt.
pub fn compute_receipt(eb: &EncryptedBallot) -> [u8; 32] {
    // h = sha256("ballot-receipt" || mir_id || serialized ciphertexts in row-major order)
    // Each ciphertext serialized as: c1.to_bytes() || c2.to_bytes()
}

/// Formats a receipt hash for display to the voter.
/// Format: XXXX-XXXX-XXXX-XXXX (hex, 16 chars = first 8 bytes of hash)
pub fn format_receipt(receipt: &[u8; 32]) -> String { ... }
```

### Step 6: Serialization (`serialize.rs`)

```rust
/// Converts an encrypted ballot to a BallotSubmission for JSON transport.
pub fn serialize_ballot(
    eb: &EncryptedBallot,
    token: &[u8],
    token_sig: &[u8],
) -> BallotSubmission { ... }

/// Reconstructs an EncryptedBallot from a BallotSubmission.
pub fn deserialize_ballot(sub: &BallotSubmission) -> Result<EncryptedBallot, &'static str> { ... }
```

## Verification Pipeline

The ballot verification pipeline (used by the voting server and verifier) runs these checks:

```
1. Deserialize ballot
2. Check matrix dimensions match the MIR's BallotSpec
3. For each cell [i][j]:
   a. Verify the point is on P-256 (deserialization does this)
   b. Verify ZeroOneProof for this cell
4. Verify ExactlyOneProof (sum = 1)
5. Verify token signature (RSA blind signature from correct MIR key)
6. Compute receipt hash and include in response
```

If ANY check fails, the ballot is rejected with a specific error code.

## Performance Targets

For MIR 23 (~434 cells):

| Operation | Target | Notes |
|---|---|---|
| Encode plaintext | < 1ms | Simple array operations |
| Encrypt all cells | < 2s | 434 × (1 scalar_base_mul + 1 scalar_mul + 1 add) |
| Generate all ZK proofs | < 5s | 434 × ZeroOneProof + 1 ExactlyOneProof |
| Verify all ZK proofs | < 3s | 434 × verify_zero_one + 1 verify_exactly_one |
| Serialize | < 100ms | Binary encoding |
| Total encrypt + prove | < 8s | Acceptable for WASM client |
| Total verify | < 4s | Acceptable for server |

Targets apply to both native and WASM — the same Rust code compiles to both. WASM performance is typically within 1.5-2× of native.

## Acceptance Criteria

- [ ] `encode_ballot` correctly encodes all valid choices for MIR 23 demo data
- [ ] `encode_ballot` rejects invalid choices (wrong party, wrong candidate position)
- [ ] Encrypted ballot decrypts back to plaintext for every cell
- [ ] All per-cell `ZeroOneProof`s verify
- [ ] `ExactlyOneProof` verifies
- [ ] A ballot with two cells set to 1 cannot produce a valid `ExactlyOneProof`
- [ ] Benaloh challenge: `verify_challenge` returns true for honest encryption
- [ ] Benaloh challenge: `verify_challenge` returns false if any ciphertext is tampered
- [ ] Receipt hash is deterministic (same encrypted ballot → same receipt)
- [ ] Serialization round-trips: `deserialize(serialize(ballot))` == ballot
- [ ] Full encrypt-prove cycle completes in < 10s (native Rust)
- [ ] Full verify cycle completes in < 5s (native Rust)
- [ ] `cargo test -p glasuvai-crypto` passes all ballot tests
- [ ] `cargo tree -p glasuvai-crypto` still shows zero external dependencies
