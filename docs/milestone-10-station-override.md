# Milestone 10: Station Override — Polling Station In-Person Override Module

## Goal

Implement the polling station application that commission members use to cancel a voter's online ballot when they choose to vote in person. The **Station Override** module runs on existing machine voting devices at ~12,000 polling stations. It: (1) queries the Identity Provider to check if a voter has an active online ballot, (2) sends a signed override message to the Voting Server (containing only the token hash — never the EGN), and (3) maintains a local tamper-evident audit log. This is the critical coercion countermeasure described in PLAN.md Section 4.4.

**This is NOT re-voting.** The in-person override is a one-way, irreversible operation: the voter's online ballot is permanently excluded from the tally, and the voter casts a paper/machine ballot at the station.

## Prerequisites

- **M1**: Election types, voter roll data structures
- **M2**: SHA-256 hashing, RSA signature verification (from crypto crate)
- **M4**: Bulletin Board API (override entry type)
- **M5**: Identity Provider (`/api/v1/voter/status` endpoint), Voting Server (override acceptance)

## Deliverables

```
packages/station-override/
  Cargo.toml                    # name = "glasuvai-station-override"
  src/
    main.rs                     # Application entry point (CLI or GUI mode)
    lib.rs                      # Library root
    auth/
      mod.rs
      commission.rs             # Commission member authentication (credentials)
      session.rs                # Two-member session enforcement
    query/
      mod.rs
      idp_client.rs             # Query IdP: "Has EGN X voted online?"
    override_flow/
      mod.rs
      protocol.rs               # Override message construction and signing
      submit.rs                 # Send override to Voting Server + BB
    ui/
      mod.rs
      terminal.rs               # Terminal-based UI (default for demo)
      touch.rs                  # Touch-screen UI stub (egui/iced for production)
    audit/
      mod.rs
      log.rs                    # Local tamper-evident hash-chained audit log
    config/
      mod.rs
      station.rs                # Station configuration (station_id, keys, endpoints)
```

```toml
# packages/station-override/Cargo.toml
[package]
name = "glasuvai-station-override"
version = "0.1.0"
edition = "2021"
description = "Polling station module for in-person override of online ballots"

[dependencies]
glasuvai-crypto = { path = "../crypto" }
axum = "0.7"             # HTTP client (reuse for IdP/VS API calls)
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

## Architecture

```
┌──────────────────────────┐        ┌──────────────────────┐
│  COMMISSION MEMBER        │        │  VOTER               │
│  (authenticates to module)│        │  (presents ID card)  │
└────────────┬─────────────┘        └────────────┬─────────┘
             │                                    │
             ▼                                    │
┌──────────────────────────────────────────────────────────┐
│  STATION OVERRIDE MODULE (on machine voting device)       │
│                                                           │
│  1. Commission member logs in (two-member auth)           │
│  2. Clerk enters/scans voter's EGN                        │
│  3. Module queries IdP: "Has EGN X voted online?"         │
│     ┌────────────────────────────────────────────┐        │
│     │  IdP responds with:                        │        │
│     │  - NO  → "No online vote. Proceed normally"│        │
│     │  - YES → returns token_hash H(T)           │        │
│     └────────────────────────────────────────────┘        │
│  4. Screen shows: "Voter has active online ballot"        │
│  5. Voter confirms: "I want to cancel my online vote"     │
│  6. Second commission member witnesses + confirms         │
│  7. Module sends signed override:                         │
│     {token_hash, station_id, timestamp, commission_sig}   │
│     → Voting Server                                       │
│     → BB appends override entry                           │
│  8. Confirmation: "Online vote cancelled. Issue paper."   │
│  9. Local audit log entry written                         │
│                                                           │
│  The module NEVER sends the EGN to the Voting Server.     │
│  Privacy separation: IdP knows WHO, VS knows WHICH token. │
└──────────────────────────────────────────────────────────┘
             │                            │
             │  HTTPS (EGN)               │  HTTPS (token_hash)
             ▼                            ▼
┌─────────────────────┐     ┌──────────────────────────────┐
│  IDENTITY PROVIDER   │     │  VOTING SERVER + BB          │
│                      │     │                              │
│  - Checks: has EGN   │     │  - Marks token as overridden │
│    been issued token? │     │  - BB appends override entry │
│  - Returns token_hash │     │  - Blocks further ballots   │
│    (NOT the EGN)      │     │    for this token            │
│  - Blocks further     │     │                              │
│    token issuance     │     │                              │
│    for this EGN       │     │                              │
└─────────────────────┘     └──────────────────────────────┘
```

## Commission Authentication (`auth/commission.rs`)

```rust
use serde::{Deserialize, Serialize};

/// A commission member's credentials for the override module.
/// In production, this would be a smart card or certificate.
/// For the demo, it's a username + password pair issued by ЦИК.
#[derive(Debug, Clone, Deserialize)]
pub struct CommissionCredentials {
    pub member_id: String,    // Unique ID (e.g., "station-2301-member-03")
    pub station_id: String,   // Polling station ID
    pub password: String,     // Demo: plaintext. Production: certificate/smart card.
}

/// An active commission session. Requires two authenticated members.
pub struct CommissionSession {
    pub station_id: String,
    pub member_1: String,     // First authenticated member
    pub member_2: Option<String>, // Second member (required for override confirmation)
    pub started_at: u64,      // Unix timestamp
}

impl CommissionSession {
    /// Authenticate first commission member. Returns partial session.
    pub fn begin(creds: &CommissionCredentials) -> Result<Self, AuthError> {
        // 1. Validate credentials against pre-loaded commission roster
        // 2. Create session with member_1 set
        // 3. member_2 is None until second member authenticates
        todo!()
    }

    /// Authenticate second commission member (two-member rule).
    /// Both members must be from the same station.
    pub fn add_witness(&mut self, creds: &CommissionCredentials) -> Result<(), AuthError> {
        // 1. Validate credentials
        // 2. Verify same station_id
        // 3. Verify different member_id (can't witness yourself)
        // 4. Set member_2
        todo!()
    }

    /// Check whether the session has two authenticated members.
    pub fn is_complete(&self) -> bool {
        self.member_2.is_some()
    }
}

pub enum AuthError {
    InvalidCredentials,
    StationMismatch,
    SameMember,
}
```

## IdP Query (`query/idp_client.rs`)

```rust
use serde::{Deserialize, Serialize};

/// Client for querying the Identity Provider about voter online ballot status.
pub struct IdpClient {
    pub base_url: String,
    pub station_cert: Vec<u8>,  // mTLS client cert (identifies polling station)
}

/// Response from IdP when querying voter override status.
#[derive(Debug, Deserialize)]
pub struct VoterOnlineStatus {
    pub has_online_ballot: bool,
    pub token_hash: Option<String>,  // H(T), present only if has_online_ballot=true
    pub already_overridden: bool,    // True if override already processed for this EGN
}

impl IdpClient {
    /// Query IdP: "Has this EGN cast an online ballot?"
    ///
    /// PRIVACY CRITICAL: The EGN is sent to the IdP (which already knows
    /// the voter's identity from authentication). The IdP returns only
    /// the token_hash — the voter's EGN is NEVER sent to the Voting Server.
    pub async fn check_voter_status(&self, egn: &str) -> Result<VoterOnlineStatus, QueryError> {
        // GET /api/v1/voter/status?egn={egn}
        // Auth: mTLS with station certificate
        // Response: VoterOnlineStatus
        //
        // If voter not in roll: 404
        // If voter has no online ballot: { has_online_ballot: false }
        // If voter has online ballot: { has_online_ballot: true, token_hash: "abc..." }
        // If already overridden: { has_online_ballot: false, already_overridden: true }
        todo!()
    }
}

pub enum QueryError {
    VoterNotFound,
    NetworkError(String),
    IdpUnavailable,
}
```

## Override Protocol (`override_flow/protocol.rs`)

```rust
use serde::{Deserialize, Serialize};

/// The override message sent to the Voting Server.
/// Contains NO voter identity information — only the token hash.
#[derive(Debug, Serialize)]
pub struct OverrideMessage {
    pub token_hash: String,         // H(T) from IdP response
    pub station_id: String,         // Polling station ID
    pub timestamp: u64,             // Unix timestamp
    pub member_1_id: String,        // First commission member
    pub member_2_id: String,        // Witnessing commission member
    pub commission_signature: Vec<u8>, // Signature over the above fields
}

/// Constructs and signs an override message.
///
/// The signing key is the station's commission key (pre-loaded during
/// station setup). In production, this would be an HSM-backed key.
/// For the demo, it's an RSA key loaded from configuration.
pub fn create_override_message(
    token_hash: &str,
    session: &CommissionSession,
    station_key: &[u8],
) -> OverrideMessage {
    // 1. Collect fields: token_hash, station_id, timestamp, member IDs
    // 2. Serialize canonical form (deterministic JSON or CBOR)
    // 3. Sign with station commission key
    // 4. Return OverrideMessage

    // CRITICAL: The OverrideMessage contains NO EGN, NO voter name,
    // NO identity information. The Voting Server receives ONLY the
    // token_hash and station metadata.
    todo!()
}
```

## Override Submission (`override_flow/submit.rs`)

```rust
/// Client for submitting override messages to the Voting Server.
pub struct VotingServerClient {
    pub base_url: String,
    pub station_cert: Vec<u8>,  // mTLS client cert
}

/// Response from the Voting Server after processing an override.
#[derive(Debug, Deserialize)]
pub struct OverrideResponse {
    pub success: bool,
    pub bb_entry_index: u64,    // BB entry index for the override record
    pub timestamp: String,
}

impl VotingServerClient {
    /// Submit the signed override message to the Voting Server.
    ///
    /// The Voting Server:
    /// 1. Verifies the commission signature
    /// 2. Looks up the ballot by token_hash
    /// 3. Marks the ballot as overridden (excluded from tally)
    /// 4. Publishes an override entry to the BB
    /// 5. Returns confirmation with BB entry index
    pub async fn submit_override(
        &self,
        msg: &OverrideMessage,
    ) -> Result<OverrideResponse, SubmitError> {
        // POST /api/v1/override
        // Auth: mTLS with station certificate
        // Body: OverrideMessage (JSON)
        //
        // Errors:
        //   404: token_hash not found (voter didn't vote online)
        //   409: already overridden
        //   403: invalid commission signature
        //   503: BB unavailable
        todo!()
    }
}

pub enum SubmitError {
    TokenNotFound,
    AlreadyOverridden,
    InvalidSignature,
    NetworkError(String),
    ServerUnavailable,
}
```

## Local Audit Log (`audit/log.rs`)

```rust
use serde::Serialize;

/// A single entry in the station's local tamper-evident audit log.
/// Hash-chained: each entry includes the hash of the previous entry.
#[derive(Debug, Serialize)]
pub struct AuditEntry {
    pub index: u64,
    pub prev_hash: [u8; 32],       // H(previous entry) — genesis uses zeros
    pub timestamp: u64,
    pub event: AuditEvent,
    pub entry_hash: [u8; 32],      // H(index || prev_hash || timestamp || event)
}

#[derive(Debug, Serialize)]
pub enum AuditEvent {
    /// Station module started
    StationOpened {
        station_id: String,
    },
    /// Commission member authenticated
    MemberAuthenticated {
        member_id: String,
    },
    /// Voter status queried (EGN is hashed for privacy in the log)
    VoterQueried {
        egn_hash: [u8; 32],        // H(EGN) — not plaintext
        has_online_ballot: bool,
    },
    /// Override processed successfully
    OverrideProcessed {
        token_hash: String,
        bb_entry_index: u64,
        member_1_id: String,
        member_2_id: String,
    },
    /// Override not needed (voter had no online ballot)
    NoOverrideNeeded {
        egn_hash: [u8; 32],
    },
    /// Override failed
    OverrideFailed {
        token_hash: Option<String>,
        reason: String,
    },
    /// Station module closed
    StationClosed {
        total_queries: u32,
        total_overrides: u32,
    },
}

/// Manages the append-only hash-chained audit log.
pub struct AuditLog {
    path: String,
    entries: Vec<AuditEntry>,
}

impl AuditLog {
    /// Open or create an audit log at the given path.
    pub fn open(path: &str) -> Result<Self, std::io::Error> {
        // If file exists: load and verify chain integrity
        // If new: create with empty chain
        todo!()
    }

    /// Append a new event to the log.
    pub fn append(&mut self, event: AuditEvent) -> Result<&AuditEntry, std::io::Error> {
        // 1. Get prev_hash (last entry's entry_hash, or zeros for genesis)
        // 2. Compute entry_hash = SHA-256(index || prev_hash || timestamp || event)
        // 3. Append to memory + flush to disk
        // 4. Return reference to new entry
        todo!()
    }

    /// Verify the entire chain is intact (no tampering).
    pub fn verify_integrity(&self) -> Result<(), AuditError> {
        // For each entry: recompute hash, compare with stored hash
        // Verify chain: entry[i].prev_hash == entry[i-1].entry_hash
        todo!()
    }
}

pub enum AuditError {
    BrokenChain { index: u64 },
    HashMismatch { index: u64 },
    IoError(std::io::Error),
}
```

## Full Override Workflow (`main.rs`)

```rust
/// The station override module.
///
/// Usage:
///   glasuvai-station-override \
///     --station-id station-2301 \
///     --idp-url https://idp.glasuvai.bg \
///     --voting-server-url https://vs.glasuvai.bg \
///     --station-key ./station-2301.key \
///     --roster ./commission-roster.json \
///     --audit-log ./audit.log \
///     --mode terminal
///
/// Options:
///   --station-id <ID>         Polling station identifier
///   --idp-url <URL>           Identity Provider API base URL
///   --voting-server-url <URL> Voting Server API base URL
///   --station-key <PATH>      Station commission signing key
///   --roster <PATH>           Commission member roster (JSON)
///   --audit-log <PATH>        Path for local audit log (default: ./audit.log)
///   --mode <MODE>             UI mode: terminal (default), touch
///
/// Interactive loop:
///   1. Commission members authenticate (two required)
///   2. Prompt: "Enter voter EGN: "
///   3. Query IdP for online ballot status
///   4. If no online ballot: "No online vote found. Proceed normally."
///   5. If online ballot exists:
///      a. Display: "Voter has active online ballot. Override? [Y/n]"
///      b. Require second commission member confirmation
///      c. Send signed override to Voting Server
///      d. Display: "Online vote cancelled. Issue paper ballot."
///   6. Log event and repeat
///
/// The voter's EGN is NEVER sent to the Voting Server.
fn main() {
    // Parse CLI args
    // Initialize IdP client, VS client, audit log
    // Authenticate commission members (two-member rule)
    // Enter interactive loop
    todo!()
}
```

## IdP API Extensions

The Identity Provider (milestone 5) needs an enhanced voter status endpoint for the override flow:

```
GET /api/v1/voter/status
  Request (query params): egn=8501011234
  Response (voter has online ballot):
    {
      "has_online_ballot": true,
      "token_hash": "a3f7c9b2...",
      "already_overridden": false
    }
  Response (voter has no online ballot):
    {
      "has_online_ballot": false,
      "already_overridden": false
    }
  Response (already overridden):
    {
      "has_online_ballot": false,
      "already_overridden": true
    }
  Auth: mTLS client cert (identifies polling station)
  Errors: 404 (not in voter roll), 403 (unauthorized station)
```

## Voting Server API Extensions

The Voting Server (milestone 5) needs an override acceptance endpoint:

```
POST /api/v1/override
  Request:
    {
      "token_hash": "a3f7c9b2...",
      "station_id": "station-2301",
      "timestamp": 1698412997,
      "member_1_id": "station-2301-member-03",
      "member_2_id": "station-2301-member-07",
      "commission_signature": "base64..."
    }
  Response:
    {
      "success": true,
      "bb_entry_index": 847294,
      "timestamp": "2026-10-27T14:23:17.442Z"
    }
  Auth: mTLS client cert (identifies polling station)
  Errors:
    404: token_hash not found
    409: already overridden
    403: invalid commission signature or unauthorized station
    503: BB unavailable
```

## Privacy Guarantees

| Data Flow | What Is Sent | What Is NOT Sent |
|---|---|---|
| Station → IdP | EGN (over mTLS) | Vote content, token T |
| IdP → Station | token_hash H(T), status | EGN (already known), token T |
| Station → Voting Server | token_hash, station_id, commission_sig | EGN, voter name, any identity info |
| Voting Server → BB | Override entry with token_hash | EGN, voter identity |

**Key property**: The Identity Provider knows WHO (EGN → token_hash mapping). The Voting Server knows WHICH token was overridden. Neither learns both. Same separation as the rest of the system.

## Implementation Steps

### Step 1: Commission Authentication

Implement `auth/commission.rs` and `auth/session.rs`. Two-member session management with credential validation.

**Test**: Single member auth → partial session. Two different members → complete session. Same member twice → rejected. Wrong station → rejected.

### Step 2: IdP Query Client

Implement `query/idp_client.rs`. Query the IdP voter status endpoint.

**Test**: Mock IdP returns has_online_ballot=true with token_hash. No online ballot → correct response. Already overridden → correct response. Network failure → graceful error.

### Step 3: Override Message Construction

Implement `override_flow/protocol.rs`. Create and sign override messages.

**Test**: Message contains correct fields. Signature verifies with station public key. Message does NOT contain EGN. Canonical serialization is deterministic.

### Step 4: Override Submission

Implement `override_flow/submit.rs`. Submit override to Voting Server.

**Test**: Mock VS accepts override → returns BB entry index. Token not found → 404 handled. Already overridden → 409 handled. Invalid signature → 403 handled.

### Step 5: Audit Log

Implement `audit/log.rs` with hash-chained entries.

**Test**: Write 50 entries covering all event types. Verify chain integrity. Tamper with one entry → detect breakage. Restart from existing log → chain continues correctly.

### Step 6: Terminal UI

Implement `ui/terminal.rs`. Interactive command-line interface for the override workflow.

**Test**: Full workflow with mock IdP + mock VS. Override succeeds. No-online-ballot case handled. Error cases display correctly.

### Step 7: CLI Binary

Implement `main.rs` with argument parsing and interactive loop.

### Step 8: Integration Test

Start IdP + Voting Server + BB. Run a full override flow:
1. Set up commission session (two members)
2. Voter authenticates and casts online ballot (via M5/M8 flow)
3. Station module queries IdP for voter's EGN → gets token_hash
4. Station sends override message to Voting Server
5. Verify ballot marked as overridden on BB
6. Verify IdP blocks further token issuance for this EGN
7. Verify tally excludes the overridden ballot (M6 integration)
8. Verify audit log contains complete chain of events

## Security Considerations

| Concern | Mitigation |
|---|---|
| Commission member impersonation | Two-member rule: both must authenticate. Production uses smart cards / certificates. |
| Unauthorized station | mTLS client certificates identify each registered polling station. Voting Server rejects unknown stations. |
| Replay of override messages | Timestamp + token_hash uniqueness. Voting Server rejects duplicate overrides (409). |
| Compromised station device | Audit log is hash-chained and synced. Post-election cross-verification between local logs and BB entries. |
| Network failure during override | Module retries with exponential backoff. If persistently unavailable, falls back to paper form (Section 4.4.4 in PLAN.md). Override state kept in local log for later batch processing. |
| EGN leakage to Voting Server | Protocol enforced: the module sends only token_hash to VS. EGN is only sent to IdP (which already knows it). Code review + integration tests verify this invariant. |
| Voter coerced to NOT override | Cannot be prevented technically. The existence of the override option (publicly known) creates uncertainty for coercers — they cannot verify whether the voter visited the station. |

## Acceptance Criteria

- [ ] Commission two-member authentication works correctly
- [ ] IdP query returns correct voter online ballot status
- [ ] Override message contains token_hash but NEVER contains EGN
- [ ] Override message signature verifies with station public key
- [ ] Voting Server accepts valid override and publishes to BB
- [ ] Voting Server rejects: unknown token, duplicate override, invalid signature
- [ ] IdP blocks further token issuance after override is processed
- [ ] Audit log entries are hash-chained and tamper-detectable
- [ ] Audit log covers all event types (open, auth, query, override, close)
- [ ] Full round-trip: voter votes online → station overrides → ballot excluded from tally
- [ ] Privacy invariant: EGN never reaches Voting Server (verified by integration test)
- [ ] Terminal UI handles all workflow states (no ballot, override, already overridden, errors)
- [ ] `cargo tree` shows only allowed dependencies (crypto + axum/tokio/serde)
- [ ] `cargo test` passes
