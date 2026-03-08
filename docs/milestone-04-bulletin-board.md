# Milestone 4: Bulletin Board

## Goal

Implement the append-only public ledger that records all encrypted ballots, overrides, and election events. The bulletin board provides two integrity mechanisms: a **hash chain** (append-only sequencing) and a **Merkle tree** (efficient inclusion proofs). Anyone can run a mirror and independently verify the entire chain.

## Prerequisites

- **M1**: Election types
- **M2**: Hash functions (domain-separated SHA-256)

## Deliverables

```
packages/bulletin-board/
  Cargo.toml                # name = "glasuvai-bulletin-board"
  src/
    main.rs                 # BB server binary
    lib.rs                  # Library root
    chain/
      mod.rs                # Module declarations
      entry.rs              # BB entry data structure
      hashchain.rs          # Hash chain: H_N = H(H_{N-1} || entry_N)
      merkle.rs             # Incremental Merkle tree
    store/
      mod.rs                # Storage trait
      sqlite.rs             # SQLite implementation (demo) — uses rusqlite
      memory.rs             # In-memory implementation (tests)
    api/
      mod.rs                # Module declarations
      server.rs             # HTTP server (axum)
      handlers.rs           # Request handlers
      middleware.rs          # Logging, CORS, rate limiting
    mirror/
      mod.rs
      sync.rs               # Mirror sync protocol (polling-based for demo)
      verify.rs             # Chain verification logic

packages/bb-mirror/
  Cargo.toml                # name = "glasuvai-bb-mirror"
  src/
    main.rs                 # Mirror binary
```

```toml
# packages/bulletin-board/Cargo.toml
[package]
name = "glasuvai-bulletin-board"
version = "0.1.0"
edition = "2021"

[dependencies]
glasuvai-crypto = { path = "../crypto" }  # hash functions only — zero transitive deps
axum = "0.7"                               # HTTP framework
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.31", features = ["bundled"] }
```

## Data Structures

### BB Entry (`chain/entry.rs`)

```rust
use crate::chain;

/// Distinguishes different kinds of BB entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryType {
    Ballot,   // Encrypted ballot submission
    Override, // In-person override cancellation
    Event,    // Election lifecycle event (open, close, etc.)
}

/// A single record on the bulletin board.
#[derive(Debug, Clone)]
pub struct Entry {
    pub index: u64,                     // Sequential entry number (0-based)
    pub prev_hash: [u8; 32],            // Hash of previous entry (zeros for genesis)
    pub timestamp: String,              // RFC 3339 timestamp
    pub entry_type: EntryType,          // Entry type
    pub mir_id: u32,                    // Which MIR (0 for election-wide events)
    pub token_hash: [u8; 32],           // H(token) — pseudonymous voter identifier
    pub payload: Vec<u8>,               // Serialized ballot, override msg, or event
    pub payload_hash: [u8; 32],         // SHA-256 of payload
    pub supersedes: Option<u64>,        // Index of entry this replaces (re-vote)
    pub server_signature: Vec<u8>,      // Server's Ed25519 signature over this entry
    pub entry_hash: [u8; 32],           // H(Index || PrevHash || Timestamp || ...)
}

/// Computes the canonical hash of an entry.
pub fn compute_entry_hash(e: &Entry) -> [u8; 32] {
    // h = sha256("bb-entry" || u64_be(index) || prev_hash || timestamp_bytes ||
    //            type_tag || u32_be(mir_id) || token_hash || payload_hash ||
    //            supersedes_or_zero)
    // supersedes: 8 zero bytes if None, otherwise u64_be(value)
}
```

### Hash Chain (`chain/hashchain.rs`)

```rust
/// Maintains the append-only hash chain state.
pub struct Chain {
    pub length: u64,
    pub head_hash: [u8; 32], // Hash of the latest entry (chain tip)
}

impl Chain {
    /// Adds a new entry to the chain.
    /// Returns the completed entry with prev_hash, index, and entry_hash filled in.
    pub fn append(&mut self, e: &mut Entry) {
        // 1. e.index = self.length
        // 2. e.prev_hash = self.head_hash (or zeros for genesis)
        // 3. e.payload_hash = sha256(&e.payload)
        // 4. e.entry_hash = compute_entry_hash(e)
        // 5. self.head_hash = e.entry_hash
        // 6. self.length += 1
    }
}

/// Verifies that a sequence of entries forms a valid chain.
pub fn verify_chain_segment(entries: &[Entry]) -> Result<(), String> {
    // For each consecutive pair:
    //   1. entries[i+1].prev_hash == entries[i].entry_hash
    //   2. entries[i+1].index == entries[i].index + 1
    //   3. entries[i+1].entry_hash == recomputed hash
    //   4. timestamps are non-decreasing
}
```

### Merkle Tree (`chain/merkle.rs`)

```rust
/// Incremental (append-only) binary Merkle tree for efficient inclusion proofs.
pub struct MerkleTree {
    leaves: Vec<[u8; 32]>,  // Leaf hashes (= entry hashes)
    levels: Vec<[u8; 32]>,  // Internal node cache for incremental updates
}

impl MerkleTree {
    /// Adds a new leaf (entry hash) to the tree.
    pub fn append(&mut self, entry_hash: [u8; 32]) { ... }

    /// Returns the current Merkle root.
    pub fn root(&self) -> [u8; 32] { ... }

    /// Generates an inclusion proof for the entry at the given index.
    pub fn prove(&self, index: u64) -> Option<InclusionProof> { ... }
}

/// Merkle inclusion proof.
pub struct InclusionProof {
    pub leaf_index: u64,
    pub leaf_hash: [u8; 32],
    pub siblings: Vec<[u8; 32]>,  // Sibling hashes from leaf to root
    pub directions: Vec<bool>,     // true = sibling is on the right
}

/// Verifies a Merkle inclusion proof against a known root.
pub fn verify_inclusion(proof: &InclusionProof, root: &[u8; 32]) -> bool {
    // Walk from leaf to root, hashing with siblings at each level
    // Final hash must equal root
}
```

### Storage Trait (`store/mod.rs`)

```rust
/// Persistence interface for the bulletin board.
pub trait Store: Send + Sync {
    /// Stores a new entry (must be next in sequence).
    fn append(&self, entry: &Entry) -> Result<(), String>;

    /// Retrieves an entry by index.
    fn get(&self, index: u64) -> Result<Option<Entry>, String>;

    /// Retrieves entries in [start, end) range.
    fn get_range(&self, start: u64, end: u64) -> Result<Vec<Entry>, String>;

    /// Retrieves all entries for a given token hash.
    fn get_by_token_hash(&self, token_hash: &[u8; 32]) -> Result<Vec<Entry>, String>;

    /// Retrieves all entries for a given MIR.
    fn get_by_mir(&self, mir_id: u32, entry_type: EntryType) -> Result<Vec<Entry>, String>;

    /// Returns the total number of entries.
    fn length(&self) -> Result<u64, String>;

    /// Returns the hash of the latest entry.
    fn latest_hash(&self) -> Result<[u8; 32], String>;
}
```

### SQLite Storage (`store/sqlite.rs`)

```rust
use rusqlite::Connection;

/// SQLite-backed bulletin board storage.
pub struct SqliteStore {
    conn: Connection,
}
```

SQLite schema:
```sql
CREATE TABLE entries (
    idx        INTEGER PRIMARY KEY,
    prev_hash  BLOB NOT NULL,
    timestamp  TEXT NOT NULL,
    type       TEXT NOT NULL,
    mir_id     INTEGER NOT NULL,
    token_hash BLOB NOT NULL,
    payload    BLOB NOT NULL,
    payload_hash BLOB NOT NULL,
    supersedes INTEGER,
    server_sig BLOB NOT NULL,
    entry_hash BLOB NOT NULL
);
CREATE INDEX idx_token_hash ON entries(token_hash);
CREATE INDEX idx_mir_id ON entries(mir_id, type);
```

`rusqlite` with the `bundled` feature compiles SQLite from source — no system dependency needed. This is the only external dependency beyond the server plumbing crates.

## HTTP API

### Endpoints

All endpoints use `axum` (Rust HTTP framework built on `tokio` and `hyper`).

```
POST   /api/v1/entries              # Submit a new entry (from voting server only)
GET    /api/v1/entries/:index       # Get entry by index
GET    /api/v1/entries?start=N&end=M # Get range of entries
GET    /api/v1/entries/by-token/:hash # Get entries by token hash
GET    /api/v1/entries/by-mir/:id   # Get entries by MIR
GET    /api/v1/chain/head           # Get chain head (index + hash + merkle root)
GET    /api/v1/chain/verify?start=N&end=M # Verify chain segment
GET    /api/v1/merkle/proof/:index  # Get Merkle inclusion proof for entry
GET    /api/v1/merkle/root          # Get current Merkle root
GET    /api/v1/status               # Health check + chain statistics
GET    /api/v1/export               # Full chain export (for mirrors and verifiers)
```

### Server Setup (`api/server.rs`)

```rust
use axum::{Router, routing::{get, post}};
use std::sync::Arc;

pub struct AppState {
    pub store: Box<dyn Store>,
    pub chain: std::sync::Mutex<Chain>,
    pub merkle: std::sync::Mutex<MerkleTree>,
}

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/v1/entries", post(handlers::submit_entry))
        .route("/api/v1/entries/:index", get(handlers::get_entry))
        .route("/api/v1/chain/head", get(handlers::get_chain_head))
        .route("/api/v1/merkle/proof/:index", get(handlers::get_merkle_proof))
        .route("/api/v1/merkle/root", get(handlers::get_merkle_root))
        .route("/api/v1/status", get(handlers::status))
        .route("/api/v1/export", get(handlers::export))
        .with_state(state)
}
```

### Chain Head Response

```json
{
    "index": 847293,
    "entry_hash": "a3f7c9b2...",
    "merkle_root": "7e2b4f1a...",
    "timestamp": "2024-10-27T14:23:17Z",
    "entries_by_type": {
        "ballot": 847000,
        "override": 250,
        "event": 43
    },
    "entries_by_mir": {
        "23": 52341,
        "24": 48922
    }
}
```

### Entry Submission (Voting Server → BB)

```json
POST /api/v1/entries
Authorization: Bearer <server-to-server-token>

{
    "type": "ballot",
    "mir_id": 23,
    "token_hash": "hex...",
    "payload": "base64...",
    "supersedes": null
}

Response 201:
{
    "index": 847294,
    "entry_hash": "hex...",
    "merkle_root": "hex...",
    "receipt_hash": "hex..."
}
```

## Mirror Sync Protocol

Mirrors use simple HTTP polling for the demo (production would use WebSocket/SSE):

```rust
/// Pulls new entries from the primary BB and verifies them.
pub struct MirrorSync {
    pub primary_url: String,
    pub local_store: Box<dyn Store>,
    pub local_chain: Chain,
    pub local_merkle: MerkleTree,
    pub poll_interval_secs: u64, // e.g., 5 seconds for demo
}

impl MirrorSync {
    /// Fetches and verifies new entries since our last known index.
    pub async fn sync(&mut self) -> Result<u64, String> {
        // 1. GET /api/v1/chain/head from primary
        // 2. If primary.index > local.length: fetch missing entries
        // 3. GET /api/v1/entries?start={local.length}&end={primary.index+1}
        // 4. For each new entry:
        //    a. Verify entry hash
        //    b. Verify chain link (prev_hash matches our chain head)
        //    c. Verify server signature
        //    d. Append to local store and chain
        //    e. Update local Merkle tree
        // 5. Compare local Merkle root with primary's — detect equivocation
    }
}
```

### Equivocation Detection

```rust
/// Compares chain heads across multiple sources.
pub async fn detect_equivocation(
    sources: &[String],
) -> Result<EquivocationReport, String> {
    // 1. Fetch /api/v1/chain/head from each source
    // 2. For each pair at the same index:
    //    if hash differs → EQUIVOCATION DETECTED
    // 3. Return report listing all sources and their chain heads
}
```

## Implementation Steps

### Step 1: Entry Data Structure and Hashing

Implement `chain/entry.rs` with `compute_entry_hash`.

**Test**: Two entries with different data → different hashes. Same data → same hash. Changing any field changes the hash.

### Step 2: Hash Chain

Implement `chain/hashchain.rs`.

**Test**: Append 1000 entries. Verify `verify_chain_segment` passes. Tamper with one entry's prev_hash → detection. Tamper with one entry's payload → hash mismatch.

### Step 3: Merkle Tree

Implement `chain/merkle.rs`.

**Test**: Build tree with 1, 2, 3, 4, 8, 100, 1000 leaves. Generate and verify inclusion proofs for random indices. Proof for leaf N does not verify against a different root.

### Step 4: Storage Backend

Implement `store/memory.rs` (for tests) and `store/sqlite.rs` (for demo).

### Step 5: HTTP API

Implement `api/server.rs` and `api/handlers.rs` using `axum`.

**Key design**: The BB server is **append-only**. There is no update or delete endpoint. The only write endpoint is `POST /api/v1/entries`, authenticated via server-to-server bearer token.

### Step 6: Mirror Sync

Implement `mirror/sync.rs` and `mirror/verify.rs`.

**Test**: Start primary BB, submit 100 entries. Start mirror, sync, verify all entries match. Submit 50 more, sync again, verify consistency.

### Step 7: BB Server Binary

Wire everything together in `main.rs`:

```rust
#[tokio::main]
async fn main() {
    // 1. Parse args: --port, --db-path, --server-key
    // 2. Open SQLite store
    // 3. Initialize chain and Merkle tree from existing entries
    // 4. Build axum router
    // 5. Start server with graceful shutdown on SIGTERM
}
```

## Acceptance Criteria

- [ ] Hash chain: 10,000 entries appended and verified in < 2 seconds
- [ ] Hash chain: single-bit tampering detected 100% of the time
- [ ] Merkle tree: inclusion proofs verify for all 10,000 entries
- [ ] Merkle tree: proof size is O(log N) — for 10K entries, ~14 hashes
- [ ] SQLite store: round-trip (write → read) correct for all entry types
- [ ] HTTP API: all endpoints return correct data (integration tests)
- [ ] Mirror: syncs 1000 entries from primary and verifies chain integrity
- [ ] Mirror: detects equivocation (primary shows different data to two mirrors)
- [ ] Server starts, accepts entries, serves queries, shuts down cleanly
- [ ] `cargo tree -p glasuvai-bulletin-board` shows only allowed deps (axum, tokio, serde, rusqlite)
- [ ] Export endpoint produces a complete, self-contained chain dump
- [ ] `cargo test -p glasuvai-bulletin-board` passes all tests
