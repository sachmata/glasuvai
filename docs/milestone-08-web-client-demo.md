# Milestone 8: Web Client & End-to-End Demo

## Goal

Build the voter-facing web application with **real Bulgarian 51st National Assembly ballot data** and wire together all components (IdP, Voting Server, BB, Tally, Verifier) into a complete end-to-end demo. The demo runs a simulated election for MIR 23 (София) with 50 voters, demonstrating every phase from key ceremony through verified results.

This is the final milestone — the proving ground for the entire system.

## Prerequisites

- **M1–M7**: All previous milestones complete

## Deliverables

```
packages/web-client/
  package.json
  tsconfig.json
  vite.config.ts
  index.html
  public/
    glasuvai_crypto_bg.wasm  # Shared crypto crate → WASM (~200-500 KB)
  src/
    main.tsx                # Application entry point
    App.tsx                 # Route setup
    crypto/
      wasm_bridge.ts        # TypeScript → Rust WASM bridge (via wasm-bindgen)
      wasm_types.ts         # Type definitions for WASM exports
    auth/
      AuthPage.tsx          # Authentication page (EGN + code)
      useAuth.ts            # Auth state management
    components/
      BallotForm.tsx        # Main ballot selection UI
      PartyList.tsx         # Party list with ballot numbers
      CandidateList.tsx     # Expandable candidate list per party
      BallotReview.tsx      # Review screen before casting
      BenalohChallenge.tsx  # Audit/spoil option
      Receipt.tsx           # Post-cast receipt display
      ReceiptVerify.tsx     # Receipt verification against BB
      BuildHash.tsx         # Build hash transparency display
      MIRInfo.tsx           # MIR information display
    i18n/
      bg.json               # Bulgarian translations
      en.json               # English translations
    styles/
      ballot.css            # Ballot-specific styles
      main.css              # Global styles

packages/crypto-wasm/
  Cargo.toml                # Thin wasm-bindgen wrapper around glasuvai-crypto
  src/
    lib.rs                  # WASM entry point, wasm-bindgen exports

packages/admin/
  Cargo.toml                # name = "glasuvai-admin"
  src/
    main.rs                 # CLI for demo-setup, demo-run, data export

test/
  e2e/
    e2e_test.rs             # End-to-end test: setup → vote → tally → verify
  demo/
    scenario.rs             # Demo scenario definition
    voters.json             # 50 demo voter records
    expected_results.json   # Expected results for verification
```

## Rust → WASM Bridge

### Building the WASM Module (`packages/crypto-wasm/`)

The WASM module is a **thin wrapper** around the shared `glasuvai-crypto` crate. There is no duplicate crypto implementation — the same P-256, ElGamal, ZKP, and blind signature code that runs on the servers compiles to WASM for the browser.

```toml
# packages/crypto-wasm/Cargo.toml
[package]
name = "glasuvai-crypto-wasm"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
glasuvai-crypto = { path = "../crypto" }  # THE shared crypto — zero external deps
wasm-bindgen = "0.2"                       # JS interop only

# That's it. All crypto comes from glasuvai-crypto.
# The WASM binary is just: glasuvai-crypto + wasm-bindgen glue.

[profile.release]
opt-level = "s"             # Optimize for size (~200-500 KB .wasm)
lto = true
```

```rust
// src/lib.rs — thin wasm-bindgen wrapper around glasuvai-crypto
use wasm_bindgen::prelude::*;
use glasuvai_crypto::{
    hash, elgamal, blind, ballot, AffinePoint, Scalar,
};

/// Generate deterministic voter token: T = SHA256(egn || code || electionId || "token")
#[wasm_bindgen(js_name = "generateToken")]
pub fn generate_token(egn: &str, code: &str, election_id: &str) -> String {
    let token = hash::hash("token", &[
        egn.as_bytes(), code.as_bytes(), election_id.as_bytes(),
    ]);
    // Inline hex encoding (no external hex crate)
    token.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Blind a token for the IdP: returns JSON { blinded, blindingFactor }
#[wasm_bindgen(js_name = "blindToken")]
pub fn blind_token(token: &str, mir_public_key_json: &str) -> Result<String, JsValue> {
    // Parse RSA public key from JSON
    // Call blind::blind_message(n, e, token_bytes)
    // Return JSON with blinded value and r
    todo!()
}

/// Unblind a signature from the IdP
#[wasm_bindgen(js_name = "unblindSignature")]
pub fn unblind_signature(
    blind_sig: &str,
    blinding_factor: &str,
    mir_public_key_json: &str,
) -> Result<String, JsValue> {
    // Call blind::unblind_signature(n, blind_sig, r)
    todo!()
}

/// Encrypt a ballot: returns serialized EncryptedBallot JSON with all ZKPs
#[wasm_bindgen(js_name = "encryptBallot")]
pub fn encrypt_ballot(
    election_pk_hex: &str,
    mir_id: u32,
    party_num: u32,
    candidate_pos: u32,
    ballot_spec_json: &str,
) -> Result<String, JsValue> {
    // 1. Parse election PK from hex → AffinePoint
    // 2. Parse ballot spec from JSON
    // 3. ballot::encode_ballot(&spec, &choice)
    // 4. ballot::encrypt_ballot(&pk, &plaintext)
    // 5. ballot::serialize_ballot(&encrypted, token, sig)
    // 6. Return JSON string
    todo!()
}

/// Compute receipt hash for an encrypted ballot
#[wasm_bindgen(js_name = "computeReceipt")]
pub fn compute_receipt(encrypted_ballot_json: &str) -> Result<String, JsValue> {
    // ballot::compute_receipt(...)
    todo!()
}

/// Reveal randomness for Benaloh challenge (spoils the ballot)
#[wasm_bindgen(js_name = "respondChallenge")]
pub fn respond_challenge(encrypted_ballot_json: &str) -> Result<String, JsValue> {
    // ballot::respond_to_challenge(...)
    todo!()
}

/// Verify a Benaloh challenge response
#[wasm_bindgen(js_name = "verifyChallenge")]
pub fn verify_challenge(
    encrypted_ballot_json: &str,
    challenge_response_json: &str,
    election_pk_hex: &str,
) -> Result<bool, JsValue> {
    // ballot::verify_challenge(...)
    todo!()
}
```

### Why This Works

Because `glasuvai-crypto` is `#[no_std]`-compatible with zero external dependencies:
- It compiles to native (for servers, tally, verifier, trustee tool)
- It compiles to `wasm32-unknown-unknown` (for the browser via `crypto-wasm`)
- **One implementation, two targets** — no cross-implementation bugs possible
- No "test vectors must match between Go and Rust" problem — there's only one codebase
- The WASM wrapper is ~50 lines of `wasm-bindgen` glue code

## Web Client UI

### Authentication Page (`AuthPage.tsx`)

```
┌─────────────────────────────────────────────────────┐
│                                                       │
│            🇧🇬 Гласувай                               │
│         Избори за 51-о Народно събрание              │
│            27 октомври 2024                           │
│                                                       │
│  ┌─────────────────────────────────────────────────┐ │
│  │  ЕГН (Personal identification number):          │ │
│  │  ┌─────────────────────────────────────────┐    │ │
│  │  │ __ __ __ __ __ __ __ __ __ __           │    │ │
│  │  └─────────────────────────────────────────┘    │ │
│  │                                                  │ │
│  │  Код за идентификация (Identity code):           │ │
│  │  ┌──────┐  ┌──────┐  ┌──────┐                   │ │
│  │  │ XXXX │─ │ XXXX │─ │ XXXX │                   │ │
│  │  └──────┘  └──────┘  └──────┘                   │ │
│  │                                                  │ │
│  │  ┌─────────────────────────────────┐             │ │
│  │  │      ВХОД  /  SIGN IN           │             │ │
│  │  └─────────────────────────────────┘             │ │
│  └─────────────────────────────────────────────────┘ │
│                                                       │
│  Build hash: a3f7c9b2... [Verify]                    │
│                                                       │
└─────────────────────────────────────────────────────┘
```

### Ballot Selection (`BallotForm.tsx`)

```
┌─────────────────────────────────────────────────────┐
│  МИР 23 — София                         16 мандата  │
│                                                       │
│  Изберете партия / коалиция:                         │
│                                                       │
│  ┌─────────────────────────────────────────────────┐ │
│  │  № 1  МЕЧтА                                ▶   │ │
│  ├─────────────────────────────────────────────────┤ │
│  │  № 2  АБВ                                  ▶   │ │
│  ├─────────────────────────────────────────────────┤ │
│  │  ★ № 3  ГЕРБ-СДС                           ▼   │ │
│  │  ┌─────────────────────────────────────────┐    │ │
│  │  │  ○ Без преференция (No preference)      │    │ │
│  │  │  ○  1. Бойко Борисов                    │    │ │
│  │  │  ○  2. Даниел Митов                     │    │ │
│  │  │  ●  3. Деница Сачева                    │    │ │
│  │  │  ○  4. ...                              │    │ │
│  │  │  ○  5. ...                              │    │ │
│  │  │  ...                                    │    │ │
│  │  └─────────────────────────────────────────┘    │ │
│  ├─────────────────────────────────────────────────┤ │
│  │  № 4  ДПС – НОВО НАЧАЛО                   ▶   │ │
│  ├─────────────────────────────────────────────────┤ │
│  │  № 5  БСП – ОБЕДИНЕНА ЛЕВИЦА               ▶   │ │
│  ├─────────────────────────────────────────────────┤ │
│  │  ...                                            │ │
│  └─────────────────────────────────────────────────┘ │
│                                                       │
│  ┌───────────────────────┐                           │
│  │  ПРЕГЛЕД  /  REVIEW   │ (enabled when choice made)│
│  └───────────────────────┘                           │
│                                                       │
└─────────────────────────────────────────────────────┘
```

### Review & Cast (`BallotReview.tsx`)

```
┌─────────────────────────────────────────────────────┐
│                                                       │
│  Вашият вот / Your vote:                             │
│                                                       │
│  Партия: № 3 ГЕРБ-СДС                               │
│  Преференция: 3. Деница Сачева                       │
│                                                       │
│  ┌───────────────────────────────────────────┐       │
│  │                                            │       │
│  │   🔒 Шифроване на бюлетината...            │       │
│  │   ████████████████████░░░ 80%              │       │
│  │   Генериране на криптографски доказателства│       │
│  │                                            │       │
│  └───────────────────────────────────────────┘       │
│                                                       │
│  ┌─────────────────────┐  ┌──────────────────────┐  │
│  │  ГЛАСУВАЙ / CAST    │  │ ПРОВЕРИ / AUDIT      │  │
│  │  VOTE               │  │ (Benaloh challenge)  │  │
│  └─────────────────────┘  └──────────────────────┘  │
│                                                       │
│  [← Назад / Back]                                    │
│                                                       │
└─────────────────────────────────────────────────────┘
```

### Receipt Screen (`Receipt.tsx`)

```
┌─────────────────────────────────────────────────────┐
│                                                       │
│  ✓ Вашият вот е записан успешно!                     │
│    Your vote was cast successfully!                   │
│                                                       │
│  ┌───────────────────────────────────────────┐       │
│  │  Разписка / Receipt:                       │       │
│  │                                            │       │
│  │    A3F7-C9B2-E1D4-8A6F                   │       │
│  │                                            │       │
│  │  [📋 Копирай / Copy]  [💾 Запази / Save]  │       │
│  └───────────────────────────────────────────┘       │
│                                                       │
│  Можете да проверите дали гласът ви е записан        │
│  на публичната дъска по всяко време.                 │
│  You can verify your vote was recorded on the        │
│  public bulletin board at any time.                   │
│                                                       │
│  [🔍 Провери сега / Verify now]                      │
│                                                       │
│  Можете да гласувате отново до 20:00 ч.              │
│  You may re-vote until 20:00.                        │
│                                                       │
│  Можете да анулирате онлайн гласа си, като           │
│  гласувате лично в секцията в деня на изборите.     │
│                                                       │
└─────────────────────────────────────────────────────┘
```

## End-to-End Demo Script

### Demo Setup (`packages/admin/src/main.rs` — `setup` subcommand)

```rust
// glasuvai-admin setup prepares the full demo environment
fn setup(mir_id: u32, num_voters: u32, num_trustees: u32, threshold: u32) {
    // 1. Generate election keypair (5-of-9 DKG ceremony)
    //    - 9 trustees, threshold 5
    //    - Save shares to trustee-shares/ directory
    //    - Publish combined PK and Feldman commitments

    // 2. Generate per-MIR RSA keys (IdP blind signature keys)
    //    - One key per MIR (at least MIR 23 for demo)
    //    - Publish public keys

    // 3. Generate 50 demo voters for MIR 23
    //    - Demo EGNs ("0000000001", "0000000002", ...)
    //    - Identity codes: random 12-char Base32 (seeded PRNG for demo reproducibility)
    //    - Save voters.json (voter roll for IdP, contains code_hash only)
    //    - Save voter_codes.json (cleartext codes, for demo client)

    // 4. Load ballot spec for MIR 23 (real party and candidate data)

    // 5. Write election config file (used by all components)

    // 6. Initialize Bulletin Board database

    // 7. Publish election setup event on BB
}
```

### Demo Run (`packages/admin/src/main.rs` — `run` subcommand)

```rust
// glasuvai-admin run executes the complete election scenario
async fn run(idp_url: &str, vs_url: &str, voter_codes_path: &str) {
    // PHASE 1: VOTING (simulate 50 voters)
    for voter in &demo_voters {
        // a. Authenticate with IdP (EGN + code)
        // b. Get blind-signed token
        // c. Randomly select a party + optional preference
        //    (weighted to produce realistic distribution)
        // d. Encrypt ballot with ZKPs (using glasuvai-crypto directly)
        // e. Submit to Voting Server
        // f. Store receipt
    }

    // PHASE 2: RE-VOTES (5 voters change their mind)
    for revoter in &revoters {
        // Same flow as above, new choice
    }

    // PHASE 3: CLOSE VOTING
    // Publish "election close" event on BB

    // PHASE 4: TALLY
    // a. Export BB data
    // b. De-duplicate ballots
    // c. Aggregate homomorphically
    // d. Run threshold decryption ceremony (5 of 9 trustees)
    // e. Solve DLOGs → get counts
    // f. Apply Hare-Niemeyer seat allocation
    // g. Apply 7% preference threshold
    // h. Publish results

    // PHASE 5: VERIFY
    // Run the independent verifier — all 10 checks must pass

    // PHASE 6: REPORT
    // Generate and display results
}
```

### Demo Vote Distribution

Approximate vote distribution for MIR 23 demo (50 voters), roughly proportional to 51st NA actual results:

```rust
const DEMO_DISTRIBUTION: &[(u32, u32)] = &[
    (3,  14), // ГЕРБ-СДС: ~28%
    (8,  10), // ПП-ДБ: ~20%
    (9,   7), // Възраждане: ~14%
    (4,   5), // ДПС-НН: ~10%
    (5,   4), // БСП-ОЛ: ~8%
    (7,   3), // Величие: ~6%
    (6,   2), // ИТН: ~4%
    (1,   2), // МЕЧтА: ~4%
    (14,  2), // ДПС: ~4%
    (10,  1), // Others: ~2%
];
// 50 preference votes distributed randomly among candidates
```

## Implementation Steps

### Step 1: WASM Module

Implement `packages/crypto-wasm/src/lib.rs` — the thin `wasm-bindgen` wrapper around `glasuvai-crypto`. Build with `wasm-pack build --target web`.

Since `glasuvai-crypto` is already `#[no_std]`-compatible, this is mostly glue code: parse JSON inputs, call crypto functions, serialize JSON outputs.

**Test**: Load WASM in browser, call `generateToken`, `encryptBallot`. Verify outputs are valid by submitting to the Rust server (same crypto code, guaranteed compatibility).

### Step 2: TypeScript Bridge

Implement `src/crypto/wasm_bridge.ts`. Wraps the `wasm-bindgen` generated JS glue with typed interfaces. Ensure all functions handle errors and return typed results.

### Step 3: Authentication UI

Implement `AuthPage.tsx`. Connect to IdP API for auth + token.

### Step 4: Ballot UI

Implement `BallotForm.tsx`, `PartyList.tsx`, `CandidateList.tsx` with real MIR 23 data.

**Design notes**:
- Party list ordered by ballot number (as on real paper ballot)
- Clicking a party expands its candidate list
- "No preference" is the first option in each list
- Only one party can be selected at a time
- Only one candidate (or no preference) per party
- Accessibility: full keyboard navigation, ARIA labels

### Step 5: Encryption & Submission

Implement `BallotReview.tsx`. Encryption runs in the WASM module (asynchronous — show progress). On "Cast", submit to Voting Server API.

### Step 6: Benaloh Challenge

Implement `BenalohChallenge.tsx`. When voter clicks "Audit":
1. WASM returns challenge response (plaintext + randomness)
2. UI displays the revealed data
3. An external tool (or the UI itself) can re-encrypt and verify
4. Ballot is spoiled — voter returns to ballot selection

### Step 7: Receipt Display and Verification

Implement `Receipt.tsx` and `ReceiptVerify.tsx`. Verification queries the BB API with the receipt hash.

### Step 8: Build Hash Display

Implement `BuildHash.tsx`. Compute SHA-256 of all loaded assets and display for transparency.

### Step 9: Demo Setup

Implement `glasuvai-admin setup` subcommand. Creates the entire demo environment.

### Step 10: Demo Run

Implement `glasuvai-admin run` subcommand. Executes the simulated election.

### Step 11: E2E Test

```rust
#[test]
fn test_full_election() {
    // 1. Run setup (DKG, voter generation, BB init)
    // 2. Start BB, IdP, Voting Server
    // 3. Run 50 voters + 5 re-votes
    // 4. Run tally
    // 5. Run verifier — ALL 10 CHECKS MUST PASS
    // 6. Verify results match expected
    // 7. Clean up
}
```

## Languages

The UI supports Bulgarian (primary), English, and Turkish. Translations are stored in `src/i18n/`:

```json
// bg.json
{
    "app.title": "Гласувай",
    "app.subtitle": "Избори за 51-о Народно събрание",
    "auth.egn_label": "ЕГН (Единен граждански номер)",
    "auth.code_label": "Код за идентификация",
    "auth.submit": "ВХОД",
    "ballot.select_party": "Изберете партия / коалиция",
    "ballot.no_preference": "Без преференция",
    "ballot.review": "ПРЕГЛЕД",
    "review.your_vote": "Вашият вот",
    "review.party": "Партия",
    "review.preference": "Преференция",
    "review.cast": "ГЛАСУВАЙ",
    "review.audit": "ПРОВЕРИ ШИФЪРА",
    "review.encrypting": "Шифроване на бюлетината...",
    "review.generating_proofs": "Генериране на доказателства...",
    "receipt.success": "Вашият вот е записан успешно!",
    "receipt.label": "Разписка",
    "receipt.copy": "Копирай",
    "receipt.save": "Запази",
    "receipt.verify_now": "Провери сега",
    "receipt.revote_info": "Можете да гласувате отново до 20:00 ч.",
    "receipt.override_info": "Можете да анулирате онлайн гласа си в секцията."
}
```

## Running the Demo

```bash
# Terminal 1: Build everything
cd /home/martin/Projects/glasuvai

# Build all Rust packages
cargo build --release

# Build WASM module
cd packages/crypto-wasm && wasm-pack build --target web --release
cp pkg/glasuvai_crypto_wasm_bg.wasm ../web-client/public/
cp pkg/glasuvai_crypto_wasm.js ../web-client/src/crypto/

# Build web client
cd ../web-client && npm install && npm run build

# Terminal 2: Setup demo
cargo run -p glasuvai-admin -- setup \
    --mir 23 --voters 50 --trustees 9 --threshold 5 \
    --output demo-data/

# Terminal 3: Start services
cargo run -p glasuvai-bulletin-board --release -- \
    --port 8081 --db demo-data/bb.db &

cargo run -p glasuvai-identity-provider --release -- \
    --port 8082 --voters demo-data/voters.json --keys demo-data/mir-keys/ &

cargo run -p glasuvai-voting-server --release -- \
    --port 8083 --bb-url http://localhost:8081 --pk demo-data/election-pk.json &

# Terminal 4: Serve web client
cd packages/web-client && npm run preview -- --port 3000

# Terminal 5: Run demo scenario (or use the web UI)
cargo run -p glasuvai-admin -- run \
    --idp-url http://localhost:8082 \
    --vs-url http://localhost:8083 \
    --voters demo-data/voter_codes.json

# Terminal 6: Tally
cargo run -p glasuvai-tally -- \
    --bb-url http://localhost:8081 \
    --shares demo-data/trustee-shares/ \
    --config demo-data/election-config.json \
    --output demo-data/results/

# Terminal 7: Verify
cargo run -p glasuvai-verifier -- verify \
    --bb-export demo-data/results/bb-export.json \
    --config demo-data/election-config.json \
    --results demo-data/results/declared-results.json
```

## Acceptance Criteria

- [ ] WASM module loads in Chrome, Firefox, Safari, Edge (~200-500 KB binary)
- [ ] WASM encryption produces ballots that the server validates (same crypto crate)
- [ ] No cross-implementation test vectors needed — single codebase, two compile targets
- [ ] Web UI displays all real parties from 51st NA registration for MIR 23
- [ ] Web UI displays real candidate names in correct list order
- [ ] Ballot encryption + ZKP generation completes in < 30s in browser
- [ ] Full auth → token → encrypt → submit → receipt flow works in browser
- [ ] Benaloh challenge correctly reveals and verifies ballot
- [ ] Receipt verification against BB works
- [ ] Demo setup creates a complete election environment
- [ ] Demo run: 50 voters + 5 re-votes complete without errors
- [ ] Tally produces correct vote counts matching demo input
- [ ] Hare-Niemeyer seat allocation matches expected result
- [ ] 7% preference threshold correctly identifies winning candidates
- [ ] **Independent verifier passes ALL 10 checks on the demo election**
- [ ] UI works in Bulgarian and English
- [ ] Keyboard navigation works for all ballot operations
- [ ] E2E test passes in CI
- [ ] Total demo runtime (setup + vote + tally + verify) < 5 minutes
- [ ] `cargo tree -p glasuvai-crypto-wasm` shows only glasuvai-crypto + wasm-bindgen
