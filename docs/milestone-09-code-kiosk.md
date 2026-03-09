# Milestone 9: Code Kiosk — Municipal Identity Code Distribution

## Goal

Implement the municipal desk application that clerks use to issue identity codes to voters on demand. The **Code Kiosk** is a lightweight CLI/daemon that: (1) verifies voter eligibility against the Identity Provider, (2) generates a cryptographically random 12-character identity code, (3) hashes and registers it with the IdP backend, and (4) outputs the plaintext code to stdout or a configured output sink (serial port for voter-facing display, pipe for receipt printer, etc.). The clerk's terminal never displays the code — only a success/failure status.

This package implements the on-demand random code generation workflow described in PLAN.md Section 11.

## Prerequisites

- **M1**: Election types, voter roll data structures
- **M2**: SHA-256 hashing (from crypto crate)
- **M5**: Identity Provider running with voter roll and authentication API

## Deliverables

```
packages/code-kiosk/
  Cargo.toml                  # name = "glasuvai-code-kiosk"
  src/
    main.rs                   # CLI binary entry point
    lib.rs                    # Library root
    generate/
      mod.rs
      random.rs               # CSPRNG-based 12-char code generation (Base32, 72 bits)
      hash.rs                 # H(EGN || code || election_id || salt) computation
    clerk/
      mod.rs
      session.rs              # Clerk session management (auth, desk ID)
      workflow.rs             # Issue/re-issue workflow state machine
    idp_client/
      mod.rs
      register.rs             # Register code hash with IdP backend
      eligibility.rs          # Check voter eligibility (not already issued, not overridden)
    output/
      mod.rs
      stdout.rs               # Default: print code to stdout
      serial.rs               # Serial port output for voter-facing LCD/display
      file.rs                 # File/pipe output (for external printer scripts)
    audit/
      mod.rs
      log.rs                  # Local tamper-evident audit log
```

```toml
# packages/code-kiosk/Cargo.toml
[package]
name = "glasuvai-code-kiosk"
version = "0.1.0"
edition = "2021"
description = "Municipal desk application for on-demand identity code generation and voter-facing output"

[dependencies]
glasuvai-crypto = { path = "../crypto" }
axum = "0.7"             # Lightweight HTTP client (reuse for IdP API calls)
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

## Architecture

```
┌──────────────────────┐         ┌──────────────────────┐
│  CLERK'S TERMINAL    │         │  VOTER-FACING DEVICE  │
│                      │         │  (LCD / receipt       │
│  code-kiosk CLI      │         │   printer)            │
│  ┌────────────────┐  │   USB   │                      │
│  │ 1. Scan voter  │  │  serial │  Displays:           │
│  │    ID (EGN)    │  │ ──────> │  XXXX-XXXX-XXXX      │
│  │ 2. Check       │  │  or     │                      │
│  │    eligibility │  │  pipe   │  (voter reads code,   │
│  │ 3. Generate    │  │         │   takes receipt)      │
│  │    random code │  │         │                      │
│  │ 4. Hash + send │  │         └──────────────────────┘
│  │    to IdP      │  │
│  │ 5. Output code │  │         ┌──────────────────────┐
│  │    to device   │  │  HTTPS  │  IDENTITY PROVIDER   │
│  │ 6. Show clerk: │  │ ──────> │                      │
│  │    "Success"   │  │         │  - Eligibility check │
│  └────────────────┘  │         │  - Store code_hash   │
│                      │         │  - Log issuance      │
│  Clerk sees ONLY:    │         └──────────────────────┘
│  "Code issued for    │
│   voter #N at HH:MM" │
└──────────────────────┘
```

## Code Generation (`generate/random.rs`)

```rust
use std::fmt;

/// A 12-character identity code: 72 bits of entropy, Base32-encoded.
/// Format: XXXX-XXXX-XXXX
/// Character set: A-Z, 2-9 (excludes 0, 1, I, O, L to avoid ambiguity)
#[derive(Clone, Zeroize)]
pub struct IdentityCode([u8; 12]);

/// Base32 alphabet excluding ambiguous characters.
/// 32 characters → 5 bits per character → 12 characters → 60 bits.
/// We use full bytes from CSPRNG and mask to the alphabet, giving 72 bits
/// of entropy in the random source (12 bytes), mapped to 12 Base32 chars.
const BASE32_ALPHABET: &[u8; 32] = b"ABCDEFGHJKMNPQRSTUVWXYZ23456789";

impl IdentityCode {
    /// Generate a new random identity code from the system CSPRNG.
    ///
    /// Security: uses getrandom (via OsRng) for entropy. The code exists
    /// only in memory and must be transmitted to the voter-facing device
    /// immediately, then zeroized.
    pub fn generate() -> Self {
        // 1. Draw 12 random bytes from OS CSPRNG
        let mut entropy = [0u8; 12];
        getrandom::getrandom(&mut entropy)
            .expect("CSPRNG failure — cannot generate identity code");

        // 2. Map each byte to a Base32 character (modulo bias is negligible
        //    for 256/32 = 8 exact divisions)
        let mut code = [0u8; 12];
        for i in 0..12 {
            code[i] = BASE32_ALPHABET[(entropy[i] % 32) as usize];
        }

        // 3. Zeroize entropy source
        entropy.iter_mut().for_each(|b| *b = 0);

        IdentityCode(code)
    }

    /// Format as XXXX-XXXX-XXXX for display on voter-facing device.
    pub fn formatted(&self) -> String {
        format!(
            "{}-{}-{}",
            std::str::from_utf8(&self.0[0..4]).unwrap(),
            std::str::from_utf8(&self.0[4..8]).unwrap(),
            std::str::from_utf8(&self.0[8..12]).unwrap(),
        )
    }

    /// Compute the code hash for storage: H(EGN || code || election_id || salt).
    /// The plaintext code is NOT stored — only this hash.
    pub fn compute_hash(
        &self,
        egn: &str,
        election_id: &str,
        salt: &[u8; 16],
    ) -> [u8; 32] {
        // Uses SHA-256 from glasuvai-crypto (same implementation as IdP)
        let mut hasher = glasuvai_crypto::sha256::Sha256::new();
        hasher.update(egn.as_bytes());
        hasher.update(&self.0);
        hasher.update(election_id.as_bytes());
        hasher.update(salt);
        hasher.finalize()
    }
}

impl Drop for IdentityCode {
    fn drop(&mut self) {
        // Zeroize code bytes on drop to minimize exposure window
        self.0.iter_mut().for_each(|b| *b = 0);
    }
}
```

## Issuance Workflow (`clerk/workflow.rs`)

```rust
/// The state machine for a single code issuance interaction.
pub enum IssuanceStep {
    /// Clerk scans voter's ID card → extracts EGN
    AwaitingVoterIdentification,
    /// System checks eligibility with IdP
    CheckingEligibility { egn: String },
    /// Code generated, awaiting hash registration with IdP
    RegisteringCode { egn: String, code: IdentityCode },
    /// Code sent to voter-facing output device
    DisplayingCode { voter_number: u32 },
    /// Clerk confirms voter received the code
    AwaitingClerkConfirmation { voter_number: u32 },
    /// Complete — audit log entry written
    Complete { voter_number: u32, timestamp: u64 },
    /// Failed at some step
    Failed { reason: String },
}

/// Result of an eligibility check against the IdP.
pub struct EligibilityResult {
    pub eligible: bool,
    pub voter_number: u32,
    pub mir_id: u32,
    pub name: String,           // Shown to clerk for identity verification
    pub already_issued: bool,   // True if re-issuance (previous code will be replaced)
}

/// Drives a single issuance from start to finish.
///
/// The clerk never sees the code. The workflow:
/// 1. Clerk enters/scans EGN
/// 2. System checks eligibility (IdP API)
/// 3. If eligible: generate random code, compute hash
/// 4. Register hash with IdP (replaces old hash if re-issuance)
/// 5. Output plaintext code to voter-facing device
/// 6. Clerk confirms handover
/// 7. Zeroize code from memory, write audit log
pub async fn run_issuance(
    egn: &str,
    election_id: &str,
    idp_client: &IdpClient,
    output: &dyn CodeOutput,
    audit: &AuditLog,
) -> Result<IssuanceReceipt, IssuanceError> {
    // Step 1-2: Check eligibility
    let eligibility = idp_client.check_eligibility(egn).await?;
    if !eligibility.eligible {
        return Err(IssuanceError::NotEligible);
    }

    // Show clerk: voter name + number for identity verification
    // (clerk compares with physical ID card)
    println!(
        "Voter: {} (#{}, MIR {})",
        eligibility.name, eligibility.voter_number, eligibility.mir_id
    );
    if eligibility.already_issued {
        println!("NOTE: Re-issuance — previous code will be invalidated.");
    }

    // Step 3: Generate random code
    let code = IdentityCode::generate();
    let salt = generate_random_salt(); // 16 random bytes
    let code_hash = code.compute_hash(egn, election_id, &salt);

    // Step 4: Register hash with IdP
    idp_client
        .register_code_hash(egn, &code_hash, &salt, eligibility.already_issued)
        .await?;

    // Step 5: Output code to voter-facing device
    output.display_code(&code.formatted()).await?;

    // Step 6: Clerk confirms (blocks until confirmation)
    // After confirmation, code is zeroized (dropped)

    // Step 7: Audit log
    let receipt = IssuanceReceipt {
        voter_number: eligibility.voter_number,
        timestamp: current_timestamp(),
        reissue: eligibility.already_issued,
    };
    audit.record(&receipt).await?;

    Ok(receipt)
    // `code` is dropped here → IdentityCode::drop() zeroizes memory
}
```

## IdP Client (`idp_client/register.rs`)

```rust
/// Client for the Identity Provider's code registration API.
///
/// The code-kiosk communicates with the IdP over HTTPS (mTLS in production).
/// Two endpoints are used:
///
/// 1. GET  /api/v1/eligibility/{egn}  → EligibilityResult
///    - Returns voter status: eligible, voter_number, name, already_issued
///    - Requires clerk authentication (mTLS client cert or session token)
///
/// 2. POST /api/v1/code-hash
///    - Body: { egn, code_hash (hex), salt (hex), reissue (bool) }
///    - IdP stores/replaces the code_hash for this voter
///    - Returns: { success: true, voter_number }
///    - If reissue=true: old hash is replaced, old code becomes invalid
pub struct IdpClient {
    base_url: String,
    client: reqwest::Client,  // or raw hyper/axum client
    desk_id: String,          // Identifies this municipal desk for audit
}

impl IdpClient {
    pub async fn check_eligibility(&self, egn: &str) -> Result<EligibilityResult, IdpError> {
        // GET {base_url}/api/v1/eligibility/{egn}
        // Headers: X-Desk-Id, authentication
        todo!()
    }

    pub async fn register_code_hash(
        &self,
        egn: &str,
        code_hash: &[u8; 32],
        salt: &[u8; 16],
        reissue: bool,
    ) -> Result<(), IdpError> {
        // POST {base_url}/api/v1/code-hash
        // Body: { egn, code_hash, salt, reissue }
        todo!()
    }
}
```

## Output Sinks (`output/`)

```rust
/// Trait for sending the generated code to the voter-facing device.
/// Implementations handle different physical output methods.
#[async_trait]
pub trait CodeOutput: Send + Sync {
    /// Display the formatted code (XXXX-XXXX-XXXX) on the voter-facing device.
    /// Must block until the code is confirmed displayed/printed.
    async fn display_code(&self, formatted_code: &str) -> Result<(), OutputError>;

    /// Clear the display after the voter has read the code.
    async fn clear(&self) -> Result<(), OutputError>;
}

/// Stdout output — prints the code to stdout (for demo/testing).
/// In production, stdout can be piped to a display driver or printer script.
pub struct StdoutOutput;

#[async_trait]
impl CodeOutput for StdoutOutput {
    async fn display_code(&self, formatted_code: &str) -> Result<(), OutputError> {
        println!("\n╔══════════════════════════════╗");
        println!("║  YOUR IDENTITY CODE:         ║");
        println!("║                              ║");
        println!("║     {}         ║", formatted_code);
        println!("║                              ║");
        println!("║  Keep this code secret.      ║");
        println!("║  You will need it to vote    ║");
        println!("║  online at glasuvai.bg       ║");
        println!("╚══════════════════════════════╝\n");
        Ok(())
    }

    async fn clear(&self) -> Result<(), OutputError> {
        // Stdout: nothing to clear
        Ok(())
    }
}

/// Serial port output — sends code to a USB-connected voter-facing display.
/// The display protocol is simple: send the formatted string, device renders it.
pub struct SerialOutput {
    port_path: String,  // e.g., "/dev/ttyUSB0"
    baud_rate: u32,     // typically 9600 or 115200
}

/// File/pipe output — writes code to a file or named pipe.
/// Useful for integration with external receipt printer scripts (e.g., CUPS).
pub struct FileOutput {
    path: String,
}
```

## Audit Log (`audit/log.rs`)

```rust
/// Tamper-evident local audit log for code issuance events.
///
/// Each entry is hash-chained to the previous one, making retroactive
/// modification detectable. The log is stored locally on the clerk's
/// terminal and periodically synced to the IdP for central aggregation.
///
/// Entries contain NO codes and NO EGNs — only voter numbers and timestamps.
pub struct AuditLog {
    path: String,
    prev_hash: [u8; 32],
}

#[derive(Serialize)]
pub struct AuditEntry {
    pub sequence: u64,
    pub voter_number: u32,
    pub timestamp: u64,
    pub reissue: bool,
    pub desk_id: String,
    pub prev_hash: String,  // hex
    pub entry_hash: String, // hex: H(prev_hash || sequence || voter_number || timestamp || ...)
}

/// IssuanceReceipt — returned to the clerk's terminal after successful issuance.
/// Does NOT contain the code.
#[derive(Debug)]
pub struct IssuanceReceipt {
    pub voter_number: u32,
    pub timestamp: u64,
    pub reissue: bool,
}
```

## CLI Interface (`main.rs`)

```rust
/// Code Kiosk — Municipal Identity Code Distribution Terminal
///
/// Usage:
///   glasuvai-code-kiosk --idp-url <URL> --election-id <ID> --desk-id <ID> [OPTIONS]
///
/// Options:
///   --idp-url <URL>       Identity Provider API base URL
///   --election-id <ID>    Election identifier (e.g., "bg-parliament-2026")
///   --desk-id <ID>        Municipal desk identifier (e.g., "sofia-desk-042")
///   --output <MODE>       Output mode: stdout (default), serial, file
///   --serial-port <PATH>  Serial port for voter-facing display (e.g., /dev/ttyUSB0)
///   --serial-baud <RATE>  Baud rate (default: 9600)
///   --file-path <PATH>    File/pipe path for code output
///   --audit-log <PATH>    Path for local audit log (default: ./audit.log)
///
/// Interactive loop:
///   1. Prompt: "Scan voter ID or enter EGN: "
///   2. Check eligibility
///   3. Show voter name to clerk for verification
///   4. Generate and output code
///   5. Prompt: "Confirm voter received code? [Y/n]: "
///   6. Log and repeat
///
/// The clerk's terminal NEVER displays the identity code.
fn main() {
    // Parse CLI args
    // Initialize IdP client, output sink, audit log
    // Enter interactive loop
    todo!()
}
```

## IdP API Extensions

The Identity Provider (milestone 5) needs two new endpoints to support the code-kiosk:

```
POST /api/v1/eligibility
  Request:  { "egn": "8501011234" }
  Response: {
    "eligible": true,
    "voter_number": 42,
    "mir_id": 23,
    "name": "Иван Петров Иванов",
    "already_issued": false
  }
  Auth: mTLS client cert (identifies the municipal desk)
  Errors: 404 (not in voter roll), 403 (unauthorized desk)

POST /api/v1/code-hash
  Request:  {
    "egn": "8501011234",
    "code_hash": "a3f7c9b2...",
    "salt": "deadbeef...",
    "reissue": false
  }
  Response: { "success": true, "voter_number": 42 }
  Auth: mTLS client cert
  Errors: 409 (already issued and reissue=false), 404, 403

GET /api/v1/distribution-stats
  Response: {
    "total_issued": 1234,
    "total_reissued": 7,
    "by_municipality": { "sofia": 500, "plovdiv": 200, ... }
  }
  Auth: admin only
  Note: Published daily as part of the distribution transparency log
```

## Implementation Steps

### Step 1: Code Generation Module

Implement `generate/random.rs` and `generate/hash.rs`. The `IdentityCode` struct with CSPRNG generation, Base32 encoding, hash computation, and zeroization on drop.

**Test**: Generate 10,000 codes — all unique, all 12 chars, all from the Base32 alphabet. Hash computation matches manual SHA-256. Zeroize verifiable (debug build with memory inspection).

### Step 2: Output Sinks

Implement `output/stdout.rs`, `output/serial.rs`, `output/file.rs` behind the `CodeOutput` trait.

**Test**: Stdout output produces expected formatted box. File output writes to temp file. Serial output opens/writes to a mock port (or skip on CI).

### Step 3: IdP Client

Implement `idp_client/eligibility.rs` and `idp_client/register.rs`.

**Test**: Mock HTTP server returns eligibility responses. Registration succeeds. Re-issuance replaces hash. Error cases (not found, already issued without reissue flag).

### Step 4: Audit Log

Implement `audit/log.rs` with hash-chained entries.

**Test**: Write 100 entries, verify chain integrity. Tamper with one entry, detect breakage.

### Step 5: Issuance Workflow

Implement `clerk/workflow.rs` and `clerk/session.rs`. Wire all components together.

**Test**: Full issuance workflow with mock IdP and stdout output. Re-issuance workflow. Failure at each step (ineligible, IdP down, output failure).

### Step 6: CLI Binary

Implement `main.rs` with argument parsing and interactive loop.

**Test**: End-to-end: start IdP (from M5), run code-kiosk against it, issue code for demo voter, verify code_hash stored in IdP matches.

### Step 7: IdP API Extensions

Add the `/api/v1/eligibility`, `/api/v1/code-hash`, and `/api/v1/distribution-stats` endpoints to the Identity Provider (M5 package).

**Test**: Integration test: code-kiosk issues code → IdP stores hash → voter authenticates with code → blind-signed token issued.

## Security Considerations

| Concern | Mitigation |
|---|---|
| Code exposure in clerk terminal memory | `IdentityCode` zeroizes on drop. Code never stored to disk. Code never logged. |
| Clerk sees code | Code is output ONLY to the voter-facing device (via `CodeOutput` trait). Clerk terminal shows only "Code issued for voter #N". |
| Code interception on USB/serial | Voter-facing device has no network. USB/serial carries only the code string. Physical security of the desk area is an operational requirement. |
| IdP compromise during issuance | Only the hash is sent to IdP, never the plaintext code. A compromised IdP cannot recover codes from hashes (72 bits entropy + salt). |
| Replay/duplicate code | Each code is generated from CSPRNG — probability of collision is $2^{-72}$ (negligible). IdP enforces one active code per voter. |
| Audit log tampering | Hash-chained entries. Periodic sync to IdP. Post-election cross-verification against IdP's distribution log. |
| Clerk issues code to wrong person | Clerk must visually verify ID card matches voter name displayed by system. Party observers present as witnesses. Same process as existing paper ballot distribution. |

## Acceptance Criteria

- [ ] `IdentityCode::generate()` produces 12-char codes from the correct Base32 alphabet
- [ ] Codes have at least 72 bits of entropy (verified by statistical test on 100K samples)
- [ ] `compute_hash()` output matches manual SHA-256(EGN || code || election_id || salt)
- [ ] Code bytes are zeroized after drop (verified in debug build)
- [ ] Clerk terminal output NEVER contains the identity code
- [ ] Stdout, serial, and file output sinks all implement `CodeOutput` correctly
- [ ] IdP eligibility check correctly identifies: eligible, already-issued, not-in-roll, overridden
- [ ] Re-issuance replaces old code hash and invalidates old code
- [ ] Audit log entries are hash-chained and tamper-detectable
- [ ] Full round-trip: kiosk generates code → IdP stores hash → voter authenticates with code → success
- [ ] Full round-trip: re-issuance → old code fails authentication → new code succeeds
- [ ] `cargo tree` shows only allowed dependencies (crypto + axum/tokio/serde)
- [ ] `cargo test` passes
