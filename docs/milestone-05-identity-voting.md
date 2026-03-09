# Milestone 5: Identity Provider & Voting Server

## Goal

Implement the two backend services that handle the voter authentication → ballot submission flow. The **Identity Provider (IdP)** validates voter identity and issues blind-signed tokens. The **Voting Server** accepts encrypted ballots, verifies proofs and tokens, and publishes to the Bulletin Board. These two services are operated by different entities and never share data directly — this separation is the foundation of ballot secrecy.

## Prerequisites

- **M1**: Election types, BallotSpec
- **M2**: RSA blind signatures, ElGamal (for ZKP verification)
- **M3**: Ballot deserialization, proof verification
- **M4**: Bulletin Board API (entry submission)

## Deliverables

```
packages/identity-provider/
  Cargo.toml                # name = "glasuvai-identity-provider"
  src/
    main.rs                 # IdP server binary
    lib.rs                  # Library root
    auth/
      mod.rs
      code.rs               # Offline identity code authentication (EGN + code)
      cert.rs               # Stub for QES certificate auth (demo: simulated)
    token/
      mod.rs
      blind.rs              # Blind signature token issuance
      keys.rs               # Per-MIR RSA key management
    voter/
      mod.rs
      roll.rs               # Voter roll lookup and management
      generate.rs           # Demo voter data generation
    api/
      mod.rs
      server.rs             # HTTP API (axum)
      handlers.rs

packages/voting-server/
  Cargo.toml                # name = "glasuvai-voting-server"
  src/
    main.rs                 # Voting server binary
    lib.rs                  # Library root
    submit/
      mod.rs
      accept.rs             # Ballot acceptance pipeline
      validate.rs           # Full ballot validation (ZKPs, token sig, dimensions)
    revote/
      mod.rs
      handler.rs            # Re-vote detection and replacement
    publish/
      mod.rs
      bb_client.rs          # Client for BB API (pushes accepted ballots)
    api/
      mod.rs
      server.rs             # HTTP API (axum)
      handlers.rs
```

```toml
# packages/identity-provider/Cargo.toml
[package]
name = "glasuvai-identity-provider"
version = "0.1.0"
edition = "2021"

[dependencies]
glasuvai-crypto = { path = "../crypto" }
axum = "0.7"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

```toml
# packages/voting-server/Cargo.toml
[package]
name = "glasuvai-voting-server"
version = "0.1.0"
edition = "2021"

[dependencies]
glasuvai-crypto = { path = "../crypto" }
axum = "0.7"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.31", features = ["bundled"] }
```

## Identity Provider

### Voter Roll (`voter/roll.rs`)

For the demo, the voter roll is loaded from a JSON file (representing the ГРАО database export):

```rust
use std::collections::HashMap;
use std::sync::RwLock;

/// A registered voter.
#[derive(Debug, Clone)]
pub struct Voter {
    pub egn: String,          // ЕГН (10-digit citizen number)
    pub voter_number: u32,    // Sequence number in voter roll
    pub mir_id: u32,          // Assigned MIR
    pub name: String,         // Full name (for display in demo only)
    pub code_hash: [u8; 32],  // H(EGN || identity_code || election_id)
    pub token_issued: bool,   // Whether a blind-signed token has been issued
    pub token_count: u32,     // How many tokens issued (for re-vote tracking)
    pub overridden: bool,     // Whether in-person override was recorded
}

/// Manages the voter roll.
pub struct Roll {
    voters: RwLock<HashMap<String, Voter>>,  // key: EGN
}

impl Roll {
    /// Validates EGN + identity code combination.
    pub fn authenticate(
        &self,
        egn: &str,
        identity_code: &str,
        election_id: &str,
    ) -> Result<Voter, AuthError> {
        // 1. Look up voter by EGN
        // 2. If not found: return ErrNotRegistered
        // 3. Compute expected = sha256(egn || identity_code || election_id)
        // 4. Compare with stored code_hash (constant-time comparison)
        // 5. If mismatch: return ErrInvalidCode
        // 6. If voter.overridden: return ErrVoterOverridden
        // 7. Return voter
    }
}

pub enum AuthError {
    NotRegistered,
    InvalidCode,
    VoterOverridden,
}
```

### Demo Voter Generation (`voter/generate.rs`)

```rust
/// Creates a set of test voters for the demo.
/// All assigned to a specific MIR with randomly generated identity codes.
///
/// Identity codes are generated from fresh randomness, matching the on-demand
/// random generation described in PLAN.md Section 11. For the demo, a seeded
/// PRNG is used for reproducibility (production uses CSPRNG).
pub fn generate_demo_voters(
    count: u32,
    mir_id: u32,
    election_id: &str,
) -> (Vec<Voter>, HashMap<String, String>) {
    // Returns:
    //   voters: the voter records (with code_hash)
    //   codes: map[EGN] -> identity_code (cleartext codes, for demo client)
    //
    // Each voter:
    //   EGN: 10-digit number (demo uses "0000000001", "0000000002", ...)
    //   Identity code: 12 random Base32 chars (72 bits entropy),
    //                  formatted as XXXX-XXXX-XXXX (excluding ambiguous chars)
    //   code_hash: sha256(egn || code || election_id)
    //   mir_id: the provided mir_id
    //   name: "Тестов Гласоподавател N" (Test Voter N)
    //
    // For reproducibility in tests, the demo uses a seeded PRNG (not CSPRNG).
    // The demo client receives cleartext codes via voter_codes.json.
    // In production, codes are generated from CSPRNG and never retained.
}
```

### Blind Signature Token Issuance (`token/blind.rs`)

```rust
use serde::{Deserialize, Serialize};

/// What the voter's client sends after authentication.
#[derive(Deserialize)]
pub struct TokenRequest {
    pub blinded_token: Vec<u8>,  // Blinded token value
    pub mir_id: u32,             // MIR for which to sign
}

/// The IdP's response.
#[derive(Serialize)]
pub struct TokenResponse {
    pub blind_signature: Vec<u8>, // Signed blinded token
}

/// Processes a token request from an authenticated voter.
pub fn issue_token(
    voter: &mut Voter,
    req: &TokenRequest,
    keys: &HashMap<u32, BlindSignatureKey>,
) -> Result<TokenResponse, &'static str> {
    // 1. Verify voter.mir_id == req.mir_id
    // 2. Get MIR-specific RSA key
    // 3. Blind-sign: sig = blind_sign(&key.d, &key.n, &req.blinded_token)
    // 4. Record: voter.token_issued = true, voter.token_count += 1
    // 5. Return signature
    //
    // CRITICAL: We do NOT log or store the blinded token value.
    // We only record THAT this voter requested a token, not WHAT it is.
}
```

### Per-MIR Key Management (`token/keys.rs`)

```rust
/// Generates one RSA key pair per MIR for blind signatures.
pub fn generate_mir_keys(
    mir_ids: &[u32],
) -> HashMap<u32, BlindSignatureKey> {
    // For each MIR: generate 3072-bit RSA key
    // Return map[mir_id] -> key
}

/// Exports all public keys for publication on the BB.
pub fn export_public_keys(
    keys: &HashMap<u32, BlindSignatureKey>,
) -> HashMap<u32, Vec<u8>> {
    // DER encoding of each public key (n, e)
}
```

### IdP HTTP API

```
POST /api/v1/auth/code          # Authenticate with EGN + identity code
POST /api/v1/auth/cert          # Authenticate with QES certificate (stub)
POST /api/v1/token/issue        # Request blind signature on token (requires auth)
GET  /api/v1/token/public-keys  # Get all MIR public keys
GET  /api/v1/stats              # Status: total tokens issued per MIR
GET  /api/v1/voter/status       # Check if a voter has online vote (for override flow)
```

### Authentication Flow

```
Client                    Identity Provider
  │                             │
  │  POST /auth/code            │
  │  {egn, identity_code}       │
  │ ──────────────────────────> │   Validate credentials
  │                             │   Check voter roll
  │  200: {session_token,       │
  │        mir_id, voter_name}  │
  │ <────────────────────────── │
  │                             │
  │  [Client computes token     │
  │   T = H(EGN||code||elID)   │
  │   Blinds: T' = Blind(T)]   │
  │                             │
  │  POST /token/issue          │
  │  Authorization: session_tok │
  │  {blinded_token, mir_id}    │
  │ ──────────────────────────> │   Blind-sign
  │                             │   Record issuance
  │  200: {blind_signature}     │
  │ <────────────────────────── │
  │                             │
  │  [Client unblinds:          │
  │   sig = Unblind(blind_sig)] │
  │   Now has (T, sig) — valid  │
  │   token for this MIR        │
```

### Session Management

For the demo, sessions are simple bearer tokens (random 32-byte hex strings) stored in-memory with a 30-minute expiry. Production would use signed JWTs with shorter lifetimes.

```rust
pub struct Session {
    pub token: String,
    pub egn: String,
    pub mir_id: u32,
    pub expires_at: u64, // Unix timestamp
}
```

## Voting Server

### Ballot Acceptance Pipeline (`submit/accept.rs`)

```rust
/// Processes a submitted ballot through the validation pipeline.
pub async fn accept_ballot(
    sub: &BallotSubmission,
    mir_keys: &HashMap<u32, (U3072, U3072)>, // (n, e) per MIR
    election_pk: &AffinePoint,
    spec: &BallotSpec,
    store: &dyn BallotStore,
    bb_client: &BBClient,
) -> Result<AcceptResponse, Vec<ValidationError>> {
    // Pipeline (each step can reject):
    //
    // 1. DESERIALIZE: Parse ballot submission
    // 2. DIMENSION CHECK: Matrix dimensions match MIR BallotSpec
    // 3. TOKEN VERIFY: Verify RSA blind signature on token for this MIR
    // 4. ZKP VERIFY: Verify all per-cell ZeroOneProofs
    // 5. SUM VERIFY: Verify ExactlyOneProof
    // 6. RE-VOTE CHECK: Check if this token has a previous ballot
    //    - If yes: mark old ballot as superseded
    // 7. STORE: Save ballot keyed by token hash
    // 8. PUBLISH: Push entry to Bulletin Board
    // 9. RESPOND: Return receipt hash + BB entry index
}

/// Returned to the voter on successful submission.
#[derive(Serialize)]
pub struct AcceptResponse {
    pub entry_index: u64,
    pub receipt_hash: String,
    pub timestamp: String,
}
```

### Ballot Validation (`submit/validate.rs`)

```rust
/// Performs all cryptographic and structural checks.
pub fn validate_ballot(
    sub: &BallotSubmission,
    mir_keys: &HashMap<u32, (U3072, U3072)>,
    election_pk: &AffinePoint,
    spec: &BallotSpec,
) -> Vec<ValidationError> {
    let mut errs = Vec::new();

    // 1. Token signature verification
    //    verify_blind_signature(n, e, &sub.token, &sub.token_signature)
    //    If fails: push ErrInvalidTokenSignature

    // 2. Matrix dimensions
    //    expected_rows = 1 + spec.max_candidates
    //    expected_cols = spec.parties.len()
    //    If mismatch: push ErrInvalidDimensions

    // 3. Per-cell ZKP verification
    //    For each cell [i][j]:
    //      if !verify_zero_one(election_pk, &ct[i][j], &proof[i][j]):
    //        push ErrInvalidCellProof { row: i, col: j }

    // 4. Exactly-one proof verification
    //    if !verify_exactly_one(election_pk, &all_ciphertexts, &sum_proof):
    //      push ErrInvalidSumProof

    errs
}
```

### Re-Vote Handling (`revote/handler.rs`)

```rust
/// Manages ballots keyed by token hash.
pub trait BallotStore: Send + Sync {
    fn get_by_token_hash(&self, token_hash: &[u8; 32]) -> Option<StoredBallot>;
    fn store(&self, token_hash: [u8; 32], ballot: StoredBallot);
    fn mark_superseded(&self, old_entry_index: u64, new_entry_index: u64);
    fn get_latest_by_mir(&self, mir_id: u32) -> Vec<StoredBallot>;
}

/// Checks for existing ballots and manages supersession.
pub fn handle_revote(
    token_hash: &[u8; 32],
    new_ballot: &StoredBallot,
    store: &dyn BallotStore,
) -> Option<u64> {
    // 1. Look up existing ballot by token hash
    // 2. If exists: return Some(old_entry_index) for BB "supersedes" field
    // 3. Store new ballot
}
```

### Publishing to BB (`publish/bb_client.rs`)

```rust
/// Communicates with the Bulletin Board server.
pub struct BBClient {
    pub base_url: String,
    pub auth_token: String, // Server-to-server bearer token
}

impl BBClient {
    /// Sends an accepted ballot to the Bulletin Board.
    pub async fn publish_ballot(
        &self,
        mir_id: u32,
        token_hash: &[u8; 32],
        payload: &[u8],
        supersedes: Option<u64>,
    ) -> Result<PublishResponse, String> {
        // POST /api/v1/entries with entry type = "ballot"
    }

    /// Sends an in-person override to the Bulletin Board.
    pub async fn publish_override(
        &self,
        token_hash: &[u8; 32],
        station_id: &str,
        commission_sig: &[u8],
    ) -> Result<PublishResponse, String> {
        // POST /api/v1/entries with entry type = "override"
    }
}
```

### Voting Server HTTP API

```
POST /api/v1/ballot/submit         # Submit encrypted ballot
GET  /api/v1/ballot/receipt/:hash  # Verify receipt exists on BB
GET  /api/v1/status                # Server health + statistics
GET  /api/v1/election/config       # Get election configuration
GET  /api/v1/election/pk           # Get election public key
```

### Full Voting Flow (End-to-End)

```
Voter's Browser                  IdP                    Voting Server           BB
      │                           │                          │                   │
      │  1. Auth (EGN+code)       │                          │                   │
      │ ────────────────────────> │                          │                   │
      │  session token            │                          │                   │
      │ <──────────────────────── │                          │                   │
      │                           │                          │                   │
      │  2. Compute T=H(EGN||    │                          │                   │
      │     code||elID)           │                          │                   │
      │     Blind T → T'          │                          │                   │
      │                           │                          │                   │
      │  3. Request blind sig     │                          │                   │
      │ ────────────────────────> │                          │                   │
      │  blind_sig(T')            │  Record: "voter X got   │                   │
      │ <──────────────────────── │   token" (not what T is) │                   │
      │                           │                          │                   │
      │  4. Unblind → (T, sig)    │                          │                   │
      │                           │                          │                   │
      │  5. Select party+pref     │                          │                   │
      │     Encode ballot matrix  │                          │                   │
      │     Encrypt (via WASM)    │                          │                   │
      │     Generate ZKPs         │                          │                   │
      │     Compute receipt hash  │                          │                   │
      │                           │                          │                   │
      │  6. POST /ballot/submit   │                          │                   │
      │     {T, sig, encrypted    │                          │                   │
      │      ballot, ZKPs, MIR}   │                          │                   │
      │ ────────────────────────────────────────────────────>│                   │
      │                           │                          │ 7. Validate:      │
      │                           │                          │    sig? ZKPs?     │
      │                           │                          │    dimensions?    │
      │                           │                          │    re-vote?       │
      │                           │                          │                   │
      │                           │                          │ 8. Publish ──────>│
      │                           │                          │                   │ Append
      │  9. {entry_index,         │                          │                   │
      │      receipt_hash}        │                          │                   │
      │ <────────────────────────────────────────────────────│                   │
      │                           │                          │                   │
      │  10. Voter stores receipt │                          │                   │
```

## Implementation Steps

### Step 1: Voter Roll Module

Implement `voter/roll.rs`, `voter/generate.rs`. Generate 50 demo voters for MIR 23.

**Test**: Authenticate valid voter → success. Wrong EGN → fail. Wrong code → fail. Overridden voter → fail.

### Step 2: Blind Signature Token Flow

Implement `token/blind.rs`, `token/keys.rs`.

**Test**: Full blinding → signing → unblinding → verification cycle. Verify IdP cannot determine token from blinded value.

### Step 3: IdP HTTP API

Implement `api/server.rs`, `api/handlers.rs` using `axum`. Wire auth → token issuance flow.

**Test**: HTTP integration tests for auth and token endpoints.

### Step 4: Ballot Validation

Implement `submit/validate.rs`. This is the voting server's core logic.

**Test**: Valid ballot → all checks pass. Invalid ZKP → rejected. Wrong MIR token → rejected. Wrong dimensions → rejected.

### Step 5: Re-Vote Handling

Implement `revote/handler.rs` with in-memory ballot store (rusqlite for production).

**Test**: First vote stored. Second vote with same token → first marked superseded. Third vote → second superseded.

### Step 6: BB Client

Implement `publish/bb_client.rs`.

**Test**: Mock BB server, verify correct HTTP requests generated.

### Step 7: Voting Server HTTP API

Implement `api/server.rs`, `api/handlers.rs` using `axum`. Wire the full pipeline.

### Step 8: Integration Test

Start IdP + Voting Server + BB. Run a full voting flow:
1. Generate demo voters
2. Authenticate voter
3. Get blind-signed token
4. Encrypt ballot (via glasuvai-crypto directly)
5. Submit to voting server
6. Verify ballot appears on BB
7. Verify receipt matches

## Acceptance Criteria

- [ ] IdP authenticates valid voters and rejects invalid credentials
- [ ] IdP issues blind-signed tokens that verify against MIR public keys
- [ ] IdP records token issuance count but cannot determine token values
- [ ] Voting server accepts valid ballots and rejects invalid ones
- [ ] Voting server correctly detects and handles re-votes
- [ ] Each accepted ballot appears on the Bulletin Board
- [ ] Receipt hash matches between client computation and server response
- [ ] Re-vote: BB entry has correct `supersedes` field pointing to old entry
- [ ] Full flow: auth → token → encrypt → submit → BB → receipt verification works
- [ ] `cargo tree` for both crates shows only allowed deps (crypto + axum/tokio/serde/rusqlite)
- [ ] 50 simulated voters can all vote successfully in sequence
- [ ] Server-to-server authentication prevents unauthorized BB writes
- [ ] `cargo test` passes for both packages
