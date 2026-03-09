# Glasuvai: Verifiable Online Voting System for Bulgaria

## Complete Architecture Design

---

## 1. SYSTEM OVERVIEW

### 1.1 Design Principles

| Principle | How it's achieved |
|---|---|
| No single point of trust | Threshold cryptography — 5-of-9 trustees from different stakeholders |
| Individual verifiability | Voter receives a receipt; Merkle inclusion proof against public BB |
| Universal verifiability | Public bulletin board + open-source verifier anyone can run |
| Ballot secrecy | Homomorphic encryption — votes never decrypted individually; token derivation includes voter-held secret preventing brute-force deanonymization |
| Coercion resistance | Online re-voting + in-person paper ballot override (Estonian model) |
| Transparency | 100% open source, reproducible builds, runtime attestation |
| Simplicity | Minimal moving parts; Rust + TypeScript; no blockchain. One crypto implementation shared between server and client (via WASM). |
| Sovereignty | All infrastructure on Bulgarian state systems; no foreign CDN |
| Accessibility | WCAG 2.1 AA, multilingual (BG/TR/EN), assisted voting support |
| Graceful degradation | Every failure mode documented; paper voting always available as fallback |

### 1.2 Trust Model — Five Independent Roles
No two roles should be controlled by the same entity:

```
┌─────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│  REGISTRAR   │  │   IDENTITY   │  │   VOTING     │  │  BULLETIN    │  │   TRUSTEES   │
│  (ЦИК/CEC)  │  │   PROVIDER   │  │   SERVER     │  │   BOARD      │  │  (9 parties) │
│              │  │              │  │              │  │              │  │              │
│ Manages      │  │ Validates    │  │ Accepts      │  │ Append-only  │  │ Hold key     │
│ voter roll   │  │ identity,    │  │ encrypted    │  │ public       │  │ shares for   │
│ per MIR      │  │ issues blind │  │ ballots,     │  │ ledger of    │  │ threshold    │
│              │  │ signatures   │  │ checks       │  │ all ballots  │  │ decryption   │
│              │  │              │  │ signatures   │  │              │  │              │
└─────────────┘  └──────────────┘  └──────────────┘  └──────────────┘  └──────────────┘
```

Key separation: The Identity Provider knows WHO voted but not HOW. The Voting Server sees encrypted ballots but not WHO cast them. Neither alone can break secrecy.

### 1.3 Trustee Composition (5-of-9 threshold)

| Seat | Holder |
|---|---|
| 1 | ЦИК (Central Election Commission) |
| 2-4 | Three largest parliamentary parties/coalitions |
| 5-6 | Two opposition parties (next largest) |
| 7 | Bulgarian Academy of Sciences (technical auditor) |
| 8 | Registered civil society / NGO observer |
| 9 | International technical auditor (e.g., OSCE-nominated) |
Any 5 of 9 can perform decryption. No political faction holds 5 seats.

---

## 2. IDENTITY & AUTHENTICATION

### 2.1 Two Authentication Paths

```
                          ┌─────────────────────────────┐
                          │         VOTER                │
                          └──────────┬──────────────────┘
                                     │
                    ┌────────────────┴────────────────┐
                    ▼                                  ▼
          ┌─────────────────┐               ┌──────────────────┐
          │  PATH A: eID    │               │  PATH B: Offline │
          │  (QES Certificate)│              │  Identity Code   │
          │                 │               │                  │
          │ User signs auth │               │ User enters EGN  │
          │ challenge with  │               │ + 12-char code   │
          │ personal cert   │               │ received at      │
          │ (B-Trust, etc.) │               │ община office    │
          └────────┬────────┘               └────────┬─────────┘
                   │                                  │
                   ▼                                  ▼
          ┌──────────────────────────────────────────────────┐
          │          IDENTITY PROVIDER SERVICE                │
          │                                                  │
          │  1. Validates identity (cert or code+EGN)        │
          │  2. Checks voter roll: eligible + correct MIR    │
          │  3. Issues BLIND SIGNATURE on voter's token      │
          │  4. Records: "EGN X has authenticated" (but NOT  │
          │     what they voted or their token value)         │
          └──────────────────────────────────────────────────┘
```

### 2.2 Offline Identity Code System
Before election day (T-30 to T-7 days):

1. ЦИК generates the voter roll from ГРАО (civil registration) database
2. For each voter: generate a 12-character alphanumeric code (72 bits entropy)
   - Format: XXXX-XXXX-XXXX (Base32 encoded, excluding ambiguous chars I/O/L/1/0)
   - Example: A3F7-KM9B-XWDQ
3. Code is stored as H(EGN || code) — the system only stores the hash, not the plaintext code
4. Distribution options:
   - Municipal office (община): Citizen presents лична карта (ID card), clerk verifies identity → code is generated on demand and displayed on a **voter-facing device** (small screen or receipt printer, like a bank PIN pad). The clerk's terminal shows only "Code issued successfully" — the clerk never sees the code.
   - Online via eID: Citizens with valid QES can request code delivery to their authenticated session (skip office visit entirely)
5. Each code is single-use for authentication but allows re-voting (re-authenticate, re-vote)

Security properties:
- The code alone is useless without the matching EGN
- The EGN alone is useless without the code
- The clerk sees the voter's identity but never sees the code (displayed only on the voter-facing device)
- The backend generates and hashes the code in a single atomic transaction — the plaintext code is transmitted only to the voter-facing device and never stored
- No persistent secret exists that could be used to reconstruct codes — ballot secrecy is a cryptographic guarantee, not a trust assumption

### 2.3 Blind Signature Protocol (Identity Unlinking)
This is the critical mechanism that prevents linking identity to vote:

```
VOTER                          IDENTITY PROVIDER
  │                                    │
  │  1. Authenticate (cert or code)    │
  │ ──────────────────────────────────>│
  │                                    │  Verify identity
  │                                    │  Check voter roll
  │                                    │  Check MIR assignment
  │                                    │
  │  2. Generate random token T        │
  │     Blind it: T' = T · r^e mod n  │
  │     (r = random blinding factor)   │
  │                                    │
  │  3. Send T' + MIR number           │
  │ ──────────────────────────────────>│
  │                                    │  Sign with MIR-specific key:
  │                                    │  S' = (T')^d_MIR mod n
  │                                    │  Mark voter as "token issued"
  │  4. Receive S'                     │
  │ <──────────────────────────────────│
  │                                    │
  │  5. Unblind: S = S' · r^-1 mod n  │
  │     Now (T, S) is a valid signed   │
  │     token for this MIR — but the   │
  │     IdP cannot link T to identity  │
  │                                    │
```

For re-voting: The voter re-authenticates and gets a NEW blind-signed token. The old token is NOT revoked at this stage — revocation happens at tally time (see Section 5).
MIR-specific keys: The Identity Provider uses a different signing key per MIR (32 keys — 31 domestic + 1 diaspora). This ensures a voter can only submit a ballot to their assigned MIR without the voting server knowing their identity.
---

## 3. BALLOT STRUCTURE

### 3.1 Bulgarian Ballot Encoding
A Bulgarian parliamentary ballot consists of:
- Party choice: One party/coalition from the registered list for this MIR
- Preference choice (optional): One candidate number from that party's list
We encode this as a matrix of ElGamal ciphertexts:

For MIR with P registered parties, and max C candidates per list:

```
Ballot = P × (C + 1) matrix of ciphertexts
         Party1    Party2    ...    PartyP
NoPreference  E(0/1)    E(0/1)   ...    E(0/1)     ← "vote for party, no preference"
Candidate 1   E(0/1)    E(0/1)   ...    E(0/1)
Candidate 2   E(0/1)    E(0/1)   ...    E(0/1)
  ...
Candidate C   E(0/1)    E(0/1)   ...    E(0/1)
```

Constraints (enforced by ZKP):
1. Exactly ONE cell in the entire matrix is E(1); all others are E(0)
2. This is proven with a disjunctive Chaum-Pedersen proof
Example: Voter in MIR 23 (Sofia) votes for Party 3, preference for Candidate 5:

```
         P1     P2     P3     P4   ...
NoPref   E(0)   E(0)   E(0)   E(0)
Cand1    E(0)   E(0)   E(0)   E(0)
...
Cand5    E(0)   E(0)   E(1)   E(0)  ← this cell
...
```

### 3.2 Homomorphic Tallying by Column
After multiplying all ballots element-wise:

```
         P1      P2      P3      P4   ...
NoPref   E(45)   E(120)  E(89)   E(33)
Cand1    E(12)   E(55)   E(41)   E(8)
Cand2    E(19)   E(38)   E(27)   E(11)
...
```

- Party vote total = sum of entire column = 45+12+19+... 
- Preference count per candidate = individual cell value (excluding NoPref row)
- 7% preference threshold checked against party total per MIR
This encoding supports homomorphic tallying without ever decrypting individual ballots.

### 3.3 Ballot Size Estimation
For a typical MIR with ~20 parties and max ~30 candidates per list:
- Matrix: 20 × 31 = 620 ElGamal ciphertexts
- Each ciphertext (P-256): ~64 bytes
- ZKP: ~equal size to ciphertexts
- Total ballot: ~80 KB
This is large but manageable. Optimization: only include rows for parties that actually have candidates in this MIR, and only up to the actual list length per party. Typical ballot would be ~20-40 KB.
---

## 4. CRYPTOGRAPHIC PROTOCOL

### 4.1 Election Setup Phase (weeks before election)

```
┌──────────────────────────────────────────────────────────┐
│                    KEY CEREMONY                           │
│                                                           │
│  1. All 9 trustees gather (public event, livestreamed)    │
│  2. Pedersen Distributed Key Generation:                  │
│     - Each trustee generates secret share sᵢ              │
│     - Each publishes commitment Cᵢ = g^sᵢ                │
│     - Cross-verification of Feldman VSS                   │
│  3. Combined election public key: PK = Π g^sᵢ            │
│     No single trustee knows the full secret key           │
│  4. PK published on bulletin board                        │
│  5. Per-MIR Identity Provider signing keys generated      │
│     (standard RSA, public keys published)                 │
│                                                           │
│  All artifacts: signed, timestamped, hash-chained         │
└──────────────────────────────────────────────────────────┘
```

### 4.2 Voting Phase

```
VOTER'S DEVICE (browser/app)              VOTING SERVER         BULLETIN BOARD
        │                                       │                      │
        │  [Already has blind-signed token       │                      │
        │   (T, S) from Identity Provider]       │                      │
        │                                       │                      │
   ┌────┴────────────────────────┐              │                      │
   │ 1. BALLOT CONSTRUCTION      │              │                      │
   │    (runs entirely in        │              │                      │
   │     browser/app — client-   │              │                      │
   │     side only)              │              │                      │
   │                             │              │                      │
   │  a. Voter selects party     │              │                      │
   │     + optional preference   │              │                      │
   │  b. Construct plaintext     │              │                      │
   │     matrix (0s and 1s)      │              │                      │
   │  c. Encrypt each cell with  │              │                      │
   │     election PK using       │              │                      │
   │     ElGamal + fresh random  │              │                      │
   │  d. Generate ZKP that       │              │                      │
   │     exactly one cell = 1    │              │                      │
   │  e. Generate ballot hash:   │              │                      │
   │     receipt = H(encrypted   │              │                      │
   │     ballot)                 │              │                      │
   └────┬────────────────────────┘              │                      │
        │                                       │                      │
        │  2. BENALOH CHALLENGE (optional)       │                      │
        │     Voter can challenge: "reveal       │                      │
        │     randomness for this ballot"        │                      │
        │     → verifies encryption correct      │                      │
        │     → this ballot is SPOILED           │                      │
        │     → voter constructs new ballot      │                      │
        │     (repeat as many times as desired)  │                      │
        │                                       │                      │
        │  3. CAST: Send (T, S, encrypted       │                      │
        │     ballot, ZKP, MIR_id)              │                      │
        │ ─────────────────────────────────────>│                      │
        │                                       │ 4. VALIDATE:          │
        │                                       │  a. Verify S is valid │
        │                                       │     sig on T under    │
        │                                       │     MIR key           │
        │                                       │  b. Verify ZKP        │
        │                                       │  c. Store ballot      │
        │                                       │     keyed by T        │
        │                                       │  d. If T already      │
        │                                       │     exists: REPLACE   │
        │                                       │     (re-vote)         │
        │                                       │                      │
        │                                       │  5. PUBLISH to BB ───>│  Append entry:
        │                                       │                      │  {T, encrypted
        │  6. Receive confirmation              │                      │   ballot, ZKP,
        │     + receipt hash                    │                      │   MIR, timestamp}
        │ <─────────────────────────────────────│                      │
        │                                       │                      │
   ┌────┴────────────────────────┐              │                      │
   │ VOTER STORES RECEIPT        │              │                      │
   │ (hash of their last ballot) │              │                      │
   │ Can verify on BB anytime    │              │                      │
   └─────────────────────────────┘              │                      │
```

### 4.3 Re-voting Mechanism (Online)

When a voter re-votes online:

1. They re-authenticate with the Identity Provider, receiving a NEW blind-signed token T₂
2. The Identity Provider records that this EGN has received a 2nd token (but doesn't know T₁ or T₂)
3. The voter submits their new ballot with T₂
4. At tally time: The Identity Provider publishes a list of {voter_sequence_number, count_of_tokens_issued} (no EGNs, no tokens)
5. For voters who received multiple tokens, a private set intersection protocol or linkable ring signatures determine which token was LAST, without revealing identity

Deterministic token derivation (recommended for V1):
- Token = H(EGN || identity_code || election_id || "token")
  CRITICAL: The identity_code (12-char code from Section 2.2) is included as a voter-held secret.
  Without it, T = H(EGN || election_id || "token") would be trivially reversible — anyone
  with the public voter roll could compute the hash for all ~7M EGNs and match against the
  bulletin board, completely destroying ballot secrecy.
- The voter blinds T, sends the blinded value to the Identity Provider for signing
- Same voter always produces the same token T (deterministic from their secrets)
- Re-voting simply overwrites the ballot for token T on the voting server
- The Identity Provider sees the same blinded value each time (cannot link to vote, but can verify it's the same voter requesting again)
- The identity_code has 72 bits of entropy (Section 2.2), making brute-force infeasible even for a single known EGN
This is simpler: no multi-token resolution needed. The voting server sees T and replaces the old ballot. The bulletin board shows version history (old ballot marked as superseded).

**Important distinction — re-voting vs. in-person override:**
- **Re-voting is exclusively an online mechanism.** A voter can authenticate and submit a new encrypted ballot as many times as they want before 20:00. Each submission overwrites the previous one for the same token T. The voter stays in the online channel throughout.
- **In-person override (Section 4.4) is NOT re-voting.** It is a one-time, irreversible cancellation of the online ballot. The voter physically goes to their polling station, the online ballot is permanently marked as overridden, and the voter casts a paper/machine ballot instead. There is no way to return to the online channel after an in-person override.

### 4.4 In-Person Override (Coercion Countermeasure — NOT Re-voting)
A voter who has cast an online ballot may override it by voting in person at their assigned polling station on election day. This is the strongest practical countermeasure against vote buying and coercion in Bulgaria's context.

**This is NOT re-voting.** The in-person override is a one-way, irreversible operation:
1. The voter's online ballot is permanently marked as overridden (excluded from tally)
2. The voter casts a paper/machine ballot at the station, which is final
3. The Identity Provider blocks all further online token issuance for this EGN
4. The voter cannot return to the online channel — the paper ballot is their definitive vote

Re-voting (casting multiple online ballots, with only the last counting) exists only in the online channel (Section 4.3). At the polling station, there is exactly one action: cancel online ballot → vote on paper → done.
#### 4.4.1 Infrastructure: Existing Machine Voting Devices
Bulgaria already deploys machine voting devices ("Информационно обслужване" tablets) at
virtually all ~12,000 polling stations. These devices have:
- Touch screens for voter interaction
- Network connectivity (mobile data, used for results transmission)
- Secure boot and certified software images
- Physical presence at stations — no new hardware procurement needed
The override module is added as a software extension to the existing machine voting
application. This requires:
- A firmware/software update to the certified machine voting image (must pass the
  same ЦИК certification process as the machine voting software itself)
- The override module runs as a separate, isolated process from the machine voting
  software (shared device, separate trust domain)
- Commission members authenticate to the override module with their commission
  credentials (separate from voter-facing machine voting mode)

For the ~1-3% of stations that lack machine voting devices (very small/remote stations,
some abroad): a fallback paper-based override protocol is used (see 4.4.4 below).

#### 4.4.2 Override Protocol (Standard — Machine-Assisted)

```
VOTER                  COMMISSION MEMBER              MACHINE VOTING DEVICE
  │                         │                                │
  │  1. Present лична       │                                │
  │     карта (ID card)     │                                │
  │ ───────────────────────>│                                │
  │                         │  2. Switch to override          │
  │                         │     module, scan/enter EGN      │
  │                         │ ──────────────────────────────>│
  │                         │                                │  3. Device queries
  │                         │                                │     Identity Provider:
  │                         │                                │     "Has EGN X cast
  │                         │                                │      an online vote?"
  │                         │                                │
  │                         │                                │  4a. Response: NO
  │                         │  "No online vote found.        │      → normal voting
  │                         │   Proceed with standard        │      proceeds, no
  │                         │   paper/machine voting."       │      override needed
  │                         │ <──────────────────────────────│
  │                         │                                │
  │                         │                                │  4b. Response: YES
  │                         │  Screen shows:                 │      (token_hash
  │                         │  "Voter has active online      │       returned)
  │                         │   ballot. Override?"           │
  │                         │ <──────────────────────────────│
  │                         │                                │
  │  5. Voter verbally      │  6. Commission member confirms │
  │     confirms: "I want   │     + second member witnesses  │
  │     to cancel my        │     (two-member rule)          │
  │     online vote and     │ ──────────────────────────────>│
  │     vote on paper."     │                                │
  │                         │                                │  7. Device sends signed
  │                         │                                │     override message:
  │                         │                                │     {token_hash,
  │                         │                                │      station_id,
  │                         │                                │      timestamp,
  │                         │                                │      commission_sig}
  │                         │                                │     → Voting Server
  │                         │                                │     → BB appends entry
  │                         │                                │
  │                         │                                │  8. Confirmation:
  │                         │  "Online vote cancelled.       │     "Override recorded
  │  9. Voter receives      │   Issue paper ballot."         │      successfully"
  │     paper ballot,       │ <──────────────────────────────│
  │     votes normally      │                                │
  │                         │                                │
```

#### 4.4.3 Privacy Flow
The override must not reveal to the Voting Server WHO the voter is:
- The commission device sends the EGN to the Identity Provider (encrypted, over TLS)
- The Identity Provider checks its records: "Has this EGN been issued a blind-signed token?"
- If yes: IdP returns the token_hash H(T) — NOT the EGN, NOT the token T
- The commission device forwards token_hash to the Voting Server as part of the
  override message. The Voting Server marks that token as overridden.
- The Voting Server never receives the EGN. It only knows "token H(T) was overridden
  at station S at time X."
- The Identity Provider knows "EGN X requested override" but does not learn the vote content
  (same separation as during authentication)
Key property: This is the same identity/vote separation as the rest of the system.
The IdP knows WHO, the Voting Server knows WHICH token — neither knows both.

#### 4.4.4 Fallback: Stations Without Machine Voting Devices
For the small number of stations without networked devices:
1. At station opening (07:00), commission receives a printed list of voter numbers
   (NOT EGNs) who have active online ballots, sourced from Identity Provider
2. If a voter on the list requests override:
   a. Commission records the override on a numbered paper form (triplicate)
   b. Form contains: voter number, station ID, timestamp, two commission signatures
   c. Override forms are collected by ЦИК courier at polls close (20:00)
   d. Batch-processed: ЦИК looks up token_hash for each voter number, sends override
      messages to Voting Server before tallying begins
3. Risk: A voter who cast an online ballot AFTER the morning list was printed would
   not appear on the list. Mitigation: the list is regenerated at 12:00 and delivered
   to offline stations. Remaining gap (12:00-20:00 online votes) is accepted as a
   known limitation for the small number of affected stations.
4. For diaspora: consular stations use the machine-assisted protocol (4.4.2) since
   consulates have reliable network connectivity.

#### 4.4.5 Timing and Anti-Abuse
- Override is available from station opening (07:00) until close (20:00)
- After 20:00, no further changes are possible (online or in-person)
- A voter can only override once (device enforces: EGN flagged after override)
- The override transaction requires physical presence of the voter + two commission members
- The override module logs all transactions locally (tamper-evident log on device)
  in addition to sending to the Voting Server — post-election audit can cross-check
- Online re-voting is blocked for overridden voters: once the Identity Provider processes
  the override, it refuses to issue new blind-signed tokens for that EGN
- The override is **irreversible** — there is no mechanism to "undo" an override and return
  to online voting. The paper ballot is final.

**Summary — re-voting vs. in-person override:**

| Property | Online re-voting (Sec 4.3) | In-person override (Sec 4.4) |
|---|---|---|
| Where | From any device (browser/app) | Physical polling station only |
| How many times | Unlimited until 20:00 | Exactly once |
| Reversible | Yes (next re-vote overwrites) | No — permanent and final |
| What happens | New encrypted ballot replaces old for same token T | Online ballot cancelled; voter casts paper/machine ballot |
| Can return to online | Yes | No — IdP blocks further token issuance |
| Final vote | Last online ballot for token T | Paper/machine ballot at station |
| Software | Web/mobile client (`web-client`, `mobile-client`) | Station override module (`station-override`) |

This mechanism means a coercer can never be certain the voter didn't override their
coerced online vote by visiting the polling station privately on election day.
---

## 5. BULLETIN BOARD (BB) — DECENTRALIZED VERIFICATION

### 5.1 Architecture

```
                    ┌─────────────────────────────────┐
                    │     PRIMARY BULLETIN BOARD       │
                    │     (operated by ЦИК)            │
                    │                                   │
                    │  Append-only log:                 │
                    │  Entry N: {hash_prev, T, ballot,  │
                    │           ZKP, MIR, timestamp,    │
                    │           server_signature}        │
                    │                                   │
                    │  Hash chain: H_N = H(H_{N-1}||    │
                    │              entry_N)              │
                    └───────────────┬───────────────────┘
                                    │
                 ┌──────────────────┼──────────────────┐
                 │                  │                    │
                 ▼                  ▼                    ▼
    ┌────────────────┐  ┌────────────────┐  ┌────────────────┐
    │  MIRROR (Party │  │  MIRROR (NGO   │  │  MIRROR        │
    │  ГЕРБ)          │  │  "Transparency │  │  (Journalist   │
    │                │  │   Bulgaria")   │  │   consortium)  │
    └────────────────┘  └────────────────┘  └────────────────┘
```

### 5.2 Mirror Protocol

Anyone can run a mirror. Each mirror:

1. Subscribes to the primary BB via WebSocket or polling
2. Receives each new entry in real-time
3. Independently verifies:
   - Hash chain integrity (H_N = H(H_{N-1} || entry_N))
   - ZKP validity for each ballot
   - Server signature validity
   - Token signature validity (MIR-specific)
4. Stores a complete copy
5. Publishes its own signed attestation of the current chain head hash at regular intervals (e.g., every 5 minutes)
Consistency check: If the primary BB shows different data to different mirrors (equivocation), the divergent chain heads will be detected instantly by comparing attestations. Any single honest mirror detects tampering.

### 5.3 Merkle Tree for Efficient Inclusion Proofs
In addition to the linear hash chain (which provides append-only integrity), the BB maintains a Merkle tree over all entries:
- Leaf nodes: H(entry_N) for each BB entry
- Tree is updated incrementally as entries are appended
- The Merkle root is published alongside the chain head hash in mirror attestations
This enables efficient inclusion proofs:
- A voter can verify their ballot is in the BB by requesting a Merkle proof (O(log N) hashes)
  instead of downloading the entire chain (O(N) entries, potentially millions)
- Third-party verifiers can spot-check random entries without downloading the full BB
- Merkle root is signed by the server and included in mirror attestations — any
  inconsistency between chain head and Merkle root is detectable
The Merkle tree is a verification optimization. The hash chain remains the primary
integrity mechanism and is fully verified by mirrors.

### 5.4 BB Data Format

```json
{
  election_id: bg-parliament-2026,
  entry_index: 847293,
  prev_hash: a3f7c9...,
  timestamp: 2026-10-27T14:23:17.442Z,
  mir_id: 23,
  token_hash: H(T),
  encrypted_ballot: base64...,
  ballot_zkp: base64...,
  token_signature: base64...,
  supersedes: null,          // or entry_index of previous vote by same token
  merkle_path: [...],        // Merkle inclusion proof for this entry
  server_signature: base64...,
  entry_hash: 7b2e4f...
}
```

---

## 6. TALLYING & RESULTS

### 6.1 End of Voting Day

```
20:00 (polls close)
        │
        ▼
┌───────────────────────────────────────────────────┐
│  1. VOTING SERVER STOPS ACCEPTING NEW BALLOTS      │
│     (enforced by timestamp + trustee co-signed     │
│      "election close" message)                     │
│                                                    │
│  2. PUBLISH FINAL BB STATE                         │
│     - Final hash chain head                        │
│     - Total ballot count per MIR                   │
│     - All mirrors confirm consistency              │
│                                                    │
│  3. RESOLVE RE-VOTES AND IN-PERSON OVERRIDES         │
│     - For each token T with multiple ballots:        │
│       keep only the LAST (by timestamp)              │
│     - Remove all ballots for tokens marked as        │
│       "overridden" (in-person override, Sec 4.4)     │
│     - Publish the de-duplicated ballot set           │
│     - Publish the override list (token hashes only)  │
│     - Anyone can verify: "final set" is a subset     │
│       of "full BB" with correct de-duplication       │
│       and correct override exclusions                │
│                                                    │
│  4. HOMOMORPHIC AGGREGATION (public, verifiable)   │
│     For each MIR:                                  │
│       For each cell [party][candidate]:            │
│         product_cell = Π encrypted_ballots[cell]   │
│     Result: one aggregated ciphertext per cell     │
│     Published on BB — anyone can verify the        │
│     multiplication                                 │
│                                                    │
│  5. THRESHOLD DECRYPTION CEREMONY                  │
│     (public event, livestreamed)                   │
│     - Each trustee computes partial decryption     │
│       of each aggregated cell                      │
│     - Each publishes Chaum-Pedersen proof of       │
│       correct partial decryption                   │
│     - 5 of 9 partial decryptions combined          │
│     - Result: plaintext tally per cell             │
│     - DLog recovery: g^count → count               │
│       (feasible since count < 7,000,000)           │
│                                                    │
│  6. SEAT ALLOCATION (deterministic, verifiable)    │
│     - Apply 4% national threshold                  │
│     - Hare-Niemeyer largest remainder method        │
│       per MIR for qualifying parties               │
│     - Apply 7% preference threshold per MIR        │
│     - Publish full seat allocation + winner list   │
└───────────────────────────────────────────────────┘
```

### 6.2 Result Reports Generated

| Report | Contents |
|---|---|
| National summary | Total votes per party, seats won, threshold calculations |
| Per-MIR breakdown | Votes per party per MIR, seats allocated per MIR, Hare quota calculations |
| Preference results | Per candidate: preference count, percentage of party total, whether threshold met |
| Final MP list | 240 elected MPs with party, MIR, and whether elected by list order or preference |
| Verification bundle | All ciphertexts, ZKPs, partial decryptions, proofs — everything needed for independent verification |
---

## 7. BUILD VERIFICATION & CODE TRANSPARENCY

### 7.1 Reproducible Builds

```
┌─────────────────────────────────────────────────────────────┐
│                   BUILD PIPELINE                              │
│                                                               │
│  Source code: GitHub public repo (glasuvai/glasuvai)           │
│  Commit: tagged release, signed by ≥3 maintainers            │
│                                                               │
│  Build: Nix-based reproducible build                         │
│  ┌─────────────────────────────────────────────┐             │
│  │  nix build .#voting-server                   │             │
│  │  nix build .#voting-client-web               │             │
│  │  nix build .#voting-client-mobile            │             │
│  │  nix build .#bulletin-board                  │             │
│  │  nix build .#verifier                        │             │
│  │                                               │             │
│  │  Output: deterministic hash per artifact      │             │
│  │  Anyone building from same commit gets        │             │
│  │  identical binary hash                        │             │
│  └─────────────────────────────────────────────┘             │
│                                                               │
│  Attestation: Multiple independent parties build             │
│  from source and publish their output hashes.                │
│  If all match → build is verified.                           │
└─────────────────────────────────────────────────────────────┘
```

### 7.2 Runtime Attestation (for voters)

Every voter visiting the voting website can verify they're running the correct code:

```
┌────────────────────────────────────────────────────────────────┐
│  VOTER'S BROWSER                                                │
│                                                                  │
│  The voting web app is a STATIC SITE (HTML + JS + WASM)         │
│  served from sovereign edge cache nodes. No server-side          │
│  rendering.                                                      │
│                                                                  │
│  Verification steps (automated, shown in UI):                   │
│                                                                  │
│  1. Browser fetches all static assets                            │
│  2. Computes SHA-256 of entire bundle:                           │
│     hash = H(index.html || app.js || crypto.wasm || ...)        │
│  3. Displays: "Build hash: a3f7c9..."                            │
│  4. Voter can compare against:                                   │
│     - Hash published by ЦИК                                     │
│     - Hash published by each party's mirror                      │
│     - Hash from building source code yourself                    │
│     - Hash reported by browser extensions (community tools)      │
│                                                                  │
│  Additionally: The app includes a Subresource Integrity          │
│  manifest — every JS/WASM file has an expected hash.             │
│                                                                  │
│  For mobile app: Reproducible builds + published APK/IPA hash   │
│  Same principle — build from source, compare hash                │
└────────────────────────────────────────────────────────────────┘
```

### 7.3 Server-Side Attestation
The voting server and bulletin board run in Confidential Computing enclaves (AMD SEV-SNP or Intel TDX) where possible, providing hardware attestation that the running binary matches the expected hash. This is a defense-in-depth measure, not a primary trust mechanism (the system is secure even without it, thanks to E2E verifiability).
---

## 8. SYSTEM COMPONENTS & TECH STACK

### 8.1 Component Map

```
┌─────────────────────────────────────────────────────────┐
│                    glasuvai MONOREPO                       │
│                                                           │
│  /packages                                                │
│  ├── /crypto          (Rust, #[no_std]-compatible)        │
│  │   ├── src/point.rs         P-256 point operations      │
│  │   ├── src/scalar.rs        Scalar arithmetic mod N     │
│  │   ├── src/field.rs         Prime field Fp arithmetic   │
│  │   ├── src/elgamal.rs       Exponential ElGamal         │
│  │   ├── src/zkp.rs           Chaum-Pedersen proofs       │
│  │   ├── src/threshold.rs     Pedersen DKG + threshold    │
│  │   ├── src/blind.rs         RSA blind signatures        │
│  │   ├── src/ballot/          Ballot encode/encrypt       │
│  │   └── Cargo.toml           ZERO external crates        │
│  │   Compiles to both native and WASM. One codebase for   │
│  │   all crypto — server and client use identical code.   │
│  │                                                        │
│  ├── /crypto-wasm     (Rust→WASM) Thin WASM wrapper       │
│  │   ├── src/lib.rs           wasm-bindgen JS glue        │
│  │   └── Cargo.toml           depends on crypto + wasm-   │
│  │   Output: ~200-500 KB .wasm                            │
│  │   Only dep: wasm-bindgen (JS interop, not crypto)      │
│  │                                                        │
│  ├── /identity-provider (Rust) Authentication service     │
│  │   ├── src/auth.rs          Code + cert authentication  │
│  │   ├── src/token.rs         Blind signature issuance    │
│  │   ├── src/voter.rs         ГРАО voter roll lookup      │
│  │   └── src/api.rs           HTTP handlers (axum)        │
│  │                                                        │
│  ├── /voting-server   (Rust)  Ballot acceptance service   │
│  │   ├── src/submit.rs        Accept + validate ballots   │
│  │   ├── src/revote.rs        Handle re-vote replacement  │
│  │   └── src/publish.rs       Push to bulletin board      │
│  │                                                        │
│  ├── /bulletin-board  (Rust)  Append-only public ledger   │
│  │   ├── src/chain.rs         Hash-chain management       │
│  │   ├── src/merkle.rs        Merkle tree                 │
│  │   ├── src/api.rs           Public read API (axum)      │
│  │   └── src/store.rs         Storage (rusqlite)          │
│  │                                                        │
│  ├── /tally           (Rust)  Tallying CLI tool            │
│  │   ├── src/aggregate.rs     Homomorphic aggregation     │
│  │   ├── src/decrypt.rs       Threshold decryption        │
│  │   ├── src/hare.rs          Hare-Niemeyer allocation    │
│  │   ├── src/preference.rs    Preference threshold check  │
│  │   └── src/report.rs        Results report generation   │
│  │                                                        │
│  ├── /verifier        (Rust)  Independent verifier tool   │
│  │   ├── src/checks/          All 10 verification checks  │
│  │   ├── src/pipeline.rs      Verification orchestrator   │
│  │   └── src/main.rs          CLI: "verify full election" │
│  │                                                        │
│  ├── /trustee-tool    (Rust)  Key ceremony + decryption   │
│  │   ├── src/keygen.rs        DKG participation           │
│  │   ├── src/decrypt.rs       Partial decryption          │
│  │   └── src/main.rs          CLI for trustees            │
│  │                                                        │
│  ├── /web-client      (TypeScript + React)                │
│  │   ├── /components                                      │
│  │   │   ├── BallotForm.tsx   Voter UI                    │
│  │   │   ├── Verify.tsx       Receipt verification        │
│  │   │   ├── Challenge.tsx    Benaloh challenge UI        │
│  │   │   └── BuildHash.tsx    Build attestation display   │
│  │   ├── /crypto                                          │
│  │   │   └── wasm_bridge.ts   Bridge to Rust WASM          │
│  │   └── /auth                                            │
│  │       ├── cert.ts          QES authentication          │
│  │       └── code.ts          Offline code auth           │
│  │                                                        │
│  ├── /mobile-client   (TypeScript + React Native)         │
│  │   └── (mirrors web-client structure)                   │
│  │                                                        │
│  ├── /admin           (Rust)  Election setup tools        │
│  │   ├── src/election_setup.rs Configure election params  │
│  │   ├── src/voter_roll.rs     Import voter roll          │
│  │   └── src/code_gen.rs       Generate identity codes    │
│  │                                                        │
│  ├── /station-override (Rust)  Polling station override   │
│  │   ├── src/auth/            Commission member auth      │
│  │   ├── src/query/           IdP query: "has EGN voted?" │
│  │   ├── src/override/        Override msg to Voting Srv  │
│  │   ├── src/ui/              Touch-screen UI (egui/iced) │
│  │   └── src/audit/           Local tamper-evident log    │
│  │   Runs on machine voting devices at ~12,000 stations.  │
│  │   Separate process from machine voting software.       │
│  │   Queries IdP by EGN, receives token_hash, sends       │
│  │   override to Voting Server (never sends EGN to VS).   │
│  │                                                        │
│  └── /code-kiosk      (Rust)  Municipal code distribution │
│      ├── src/generate/        CSPRNG code gen + hashing   │
│      ├── src/clerk/           Issuance workflow + session  │
│      ├── src/idp_client/      IdP eligibility + hash reg  │
│      ├── src/output/          Voter-facing output sinks   │
│      │   (stdout, serial, file/pipe)                      │
│      └── src/audit/           Hash-chained audit log      │
│                                                           │
│  /nix                 Reproducible build definitions       │
│  /docs                Protocol specification               │
│  /test                End-to-end test suite                │
└─────────────────────────────────────────────────────────┘
```

### 8.2 Technology Choices

| Component | Technology | Rationale |
|---|---|---|
| Crypto library | Rust `#[no_std]` crate, P-256 from first principles | One implementation for both server and client. Zero external crypto crates. Every operation traceable to textbook definitions. |
| Elliptic curve | P-256 (NIST secp256r1) | Well-studied, government-approved standard. Implemented from first principles in Rust. |
| Browser crypto | Rust→WASM (via `wasm-bindgen`) | ~200-500 KB binary. Same crypto crate compiled to WASM — guaranteed identical behavior to server. |
| Server framework | Rust + `axum` | Lightweight, well-audited async HTTP. Comparable simplicity to Go `net/http`. |
| BB storage | SQLite via `rusqlite` | Embedded, zero-config, sufficient for demo. |
| Web client | TypeScript + React (Vite) | Thin UI shell; all crypto in Rust→WASM |
| Mobile client | React Native (TypeScript) | Shared logic with web client |
| Build system | Nix + Cargo + `wasm-pack` | Reproducible builds, deterministic outputs |
| CI/CD | GitHub Actions | Public, auditable pipeline |

### 8.3 Dependency Policy

**Two-tier rule**: absolute zero external crates on the crypto path; minimal, well-audited crates for non-crypto server plumbing.

| Tier | Scope | External crates allowed |
|---|---|---|
| **Crypto path** (security-critical) | `packages/crypto` — ElGamal, ZKPs, DKG, blind sigs, ballot encoding | **ZERO**. Only `core`/`alloc`/`std`. P-256 field arithmetic, scalar arithmetic, point operations — all from first principles. `#[no_std]`-compatible. |
| **Server plumbing** (not security-critical) | HTTP handlers, JSON serialization, SQLite storage, async runtime | `axum`, `tokio`, `serde`/`serde_json`, `rusqlite`. Each is mature, widely audited, and replaceable. |
| **WASM bridge** | `packages/crypto-wasm` — JS interop only | `wasm-bindgen` (for JS glue). No crypto logic in this layer. |
| **Web client** | UI only | React (via npm). All crypto delegated to WASM module. |

**Why this is stricter than the original Go plan**: Go's `crypto/elliptic` and `math/big` are stdlib but still external code trusted implicitly. In the Rust version, the crypto path trusts **no code outside this repository** — P-256 is implemented from scratch.

**Single implementation advantage**: Since server and client share the same Rust crypto crate, there are no cross-language consistency issues. No shared test vectors needed for sync — `cargo test` covers everything. One bug fix applies everywhere.
---

## 9. VOTER EXPERIENCE FLOW

```
┌──────────────────────────────────────────────────────────────────┐
│                        VOTER JOURNEY                              │
│                                                                    │
│  ┌─────────────┐                                                  │
│  │ STEP 1      │  Voter opens glasuvai.bg in browser or mobile app  │
│  │ ARRIVE      │  → App loads, displays build hash for             │
│  │             │    transparency. Voter can verify (optional).     │
│  └──────┬──────┘                                                  │
│         │                                                          │
│  ┌──────▼──────┐                                                  │
│  │ STEP 2      │  Two options shown:                               │
│  │ IDENTIFY    │  A) "Sign in with electronic certificate"         │
│  │             │     → Browser prompts for QES cert                │
│  │             │  B) "Sign in with identity code"                  │
│  │             │     → Enter ЕГН + 12-char code                   │
│  └──────┬──────┘                                                  │
│         │                                                          │
│  ┌──────▼──────┐                                                  │
│  │ STEP 3      │  Identity Provider validates, returns             │
│  │ TOKEN       │  blind-signed token. Voter's device               │
│  │             │  stores it. Screen shows: "Authenticated          │
│  │             │  for MIR 23 - София 1"                            │
│  └──────┬──────┘                                                  │
│         │                                                          │
│  ┌──────▼──────┐                                                  │
│  │ STEP 4      │  Ballot screen shows:                             │
│  │ VOTE        │  - List of parties/coalitions for this MIR        │
│  │             │  - Voter taps a party                              │
│  │             │  - Party list expands showing candidates           │
│  │             │  - Voter optionally taps one candidate             │
│  │             │  - "Review" button                                 │
│  └──────┬──────┘                                                  │
│         │                                                          │
│  ┌──────▼──────┐                                                  │
│  │ STEP 5      │  Summary screen:                                  │
│  │ CONFIRM     │  "You are voting for: [Party] / [Candidate]"      │
│  │             │  Two buttons:                                      │
│  │             │  [CAST VOTE]  [VERIFY ENCRYPTION (optional)]      │
│  │             │                                                    │
│  │             │  If "Verify": Benaloh challenge — app reveals      │
│  │             │  randomness, voter (or external tool) checks.      │
│  │             │  This ballot is spoiled, voter returns to Step 4.  │
│  │             │                                                    │
│  │             │  If "Cast": ballot is encrypted, ZKP generated,   │
│  │             │  submitted to voting server.                       │
│  └──────┬──────┘                                                  │
│         │                                                          │
│  ┌──────▼──────┐                                                  │
│  │ STEP 6      │  Confirmation screen:                              │
│  │ RECEIPT     │  "Vote cast successfully!"                        │
│  │             │  "Your receipt: a3f7-c9b2-..."                    │
│  │             │  [COPY]  [SAVE TO DEVICE]                         │
│  │             │                                                    │
│  │             │  "You can verify your vote was recorded at         │
│  │             │   verify.glasuvai.bg anytime."                     │
│  │             │                                                    │
│  │             │  "You may re-vote until 20:00 to change            │
│  │             │   your choice."                                    │
│  │             │                                                    │
│  │             │  "You can also cancel your online vote by           │
│  │             │   voting in person at your polling station          │
│  │             │   on election day."                                 │
│  └─────────────┘                                                  │
└──────────────────────────────────────────────────────────────────┘
```

---

## 10. SECURITY ANALYSIS

### 10.1 Attack Scenarios & Mitigations

| Attack | Mitigation |
|---|---|
| Voting server manipulates ballots | Impossible — ballots are encrypted client-side. Server only sees ciphertexts + ZKPs. Cannot modify without breaking ZKP. |
| Voting server drops ballots | Detected — voter has receipt hash, checks BB via Merkle inclusion proof. If ballot missing, voter has cryptographic proof of suppression. Formal dispute procedure in Section 15.3. |
| Voting server adds fake ballots | Each ballot requires a blind-signed token from Identity Provider. Server cannot forge these. Number of tokens issued (published by IdP) must match ballots on BB. |
| Identity Provider links identity to vote | Impossible — blind signature protocol ensures IdP never sees the unblinded token. Cannot link identity to BB entry. Token derivation includes voter-held secret (identity code), preventing brute-force reversal from public voter roll. |
| IdP + Voting Server collude | This is the main threat. Mitigated by: (a) different organizations operate each, (b) voter can verify receipt independently, (c) token count auditing, (d) multiple IdP operators possible (federation), (e) token includes voter-held secret unknown to both IdP and Voting Server. |
| Trustees collude to decrypt individual votes | Need 5-of-9. Would require ЦИК + 3 parties + 1 other to collude. Politically extremely unlikely across rival parties. |
| Coercion / vote buying | Three-layer defense: (1) Online re-voting — voter can change vote after coercer leaves. (2) In-person override — voter can cancel online vote by voting at polling station on election day (Section 4.4). Coercer can never be certain the vote wasn't overridden. (3) Coercer cannot verify how voter voted (encrypted, unlinkable). |
| Man-in-the-middle on voter's device | Build hash verification + Benaloh challenge. Static client served from sovereign infrastructure with signed assets (Section 12). Modified client code would produce wrong hash and fail signature check. Challenged ballots would fail verification. |
| DDoS prevents voting | Sovereign edge-cached static client + multiple voting server replicas + extended voting window (Section 15.1). BB mirrors ensure data survives. Rate limiting per token. |
| Voter impersonation (stolen identity code) | Requires BOTH ЕГН + code. Re-voting allows real voter to override online. In-person voting at polling station provides ultimate override. Audit log shows multiple authentications (triggers investigation). |
| Token brute-force from voter roll | Token = H(EGN \|\| identity_code \|\| election_id \|\| "token"). Identity code has 72 bits of entropy. Even with a known EGN, 2^72 brute-force is computationally infeasible. Without the original V1-proposed deterministic token (which lacked the identity_code), this attack would have been trivial. |
| Edge node serves modified client code | Sovereign edge nodes serve pre-signed static assets. Signature verified client-side. Mirrors and community tools cross-check. A compromised edge node is detected immediately (Section 12). |
| Fewer than 5 trustees available | Decryption postponed up to 48 hours. If still unavailable, online component declared failed; paper/machine results certified independently (Section 15.1). |

### 10.2 What This System Does NOT Protect Against

Honesty requires acknowledging limitations:

1. **Persistent real-time coercion with physical presence at polling station** — If a coercer physically accompanies the voter to prevent in-person override AND watches their screen for all online re-voting, the three-layer defense is defeated. This requires continuous physical control of the voter from first online vote through polling station close. This is a fundamental limitation of ALL remote voting systems, though the in-person override raises the cost and risk for coercers significantly compared to online-only re-voting.

2. **Compromised voter device** — If the voter's phone/computer has malware, it could modify the ballot before encryption. Benaloh challenge helps detect this probabilistically, but a sophisticated rootkit could defeat it. Mitigations: allow voting from multiple devices; vote from a freshly-booted live USB; official kiosks at municipal offices (Section 16.4); ultimately, in-person override as last resort.

3. **Large-scale voter roll fraud** — If ГРАО data is corrupted (fake citizens), the system would issue valid tokens to non-existent people. Mitigation: publish voter roll for public audit at T-60 (with privacy protections). Identity code distribution requires physical presence, limiting the scale of ghost voter exploitation.

4. **Systemic state-level attack** — If the Bulgarian state itself (controlling ГРАО, ЦИК, and state infrastructure) acts as a unified adversary, no purely technical system can guarantee integrity. The multi-stakeholder trust model, international auditors, and open-source verifiability make such an attack detectable and publicly provable, but cannot prevent it. This is a political, not technical, boundary.
---

## 11. OFFLINE IDENTITY CODE DISTRIBUTION — DETAILED DESIGN

**T-60 days:** ЦИК publishes voter roll per MIR (name + voter number, no EGN). Public can challenge (ineligible voters, missing voters).

**T-30 days:** System provisioning:
- The Identity Provider's code-generation backend is deployed with a hardware random number generator (HRNG/TRNG) for high-quality entropy
- No bulk generation or printing occurs. No sealed cards, envelopes, or physical logistics.
- No HSM master secret — each code is generated from fresh randomness at issuance time and immediately discarded after hashing. There is no secret that could be used to reconstruct codes after the fact.

**T-30 to T-1:** On-demand distribution at municipal offices:
- Party observers may be present during distribution (same right as for paper ballot distribution under the Electoral Code)
- Citizen presents лична карта (ID card)
- Clerk scans ID → clerk's terminal sends the voter sequence number to the backend (the clerk's screen shows only the voter's name and municipality for identity verification)
- Backend verifies eligibility, generates a fresh random 12-character code (72 bits from CSPRNG), computes H(EGN || code || election_id || salt), stores only the hash, and sends the plaintext code to the voter-facing device
- The code is displayed on a **voter-facing device** at the clerk's desk — a small screen or thermal receipt printer oriented toward the voter (like a bank PIN pad). The clerk's terminal displays only "Code issued successfully."
- **CRITICAL SEPARATION:** The clerk sees the voter's identity (name, ID card) but never sees the code. The backend generates and hashes the code in a single atomic transaction — the plaintext is transmitted only to the voter-facing device and never retained.
- Citizen reads the code from the voter-facing device (or takes the printed receipt)
- Clerk confirms handover in system; citizen signs receipt
- System records: "Voter #N received code at [timestamp]" (but not the code itself — only H(EGN || code || election_id || salt) is stored)
- Distribution log published daily (count per municipality, no voter names/EGNs)

**Hardware requirements per municipal desk:**
- One voter-facing display (small LCD, ~€30) OR one thermal receipt printer (standard POS printer, ~€50)
- Connected to the clerk's terminal via USB; displays/prints only the code
- No network access from the voter-facing device itself — it receives data only from the clerk's terminal
- Estimated national deployment: ~3,000 municipal desks × ~€50 = ~€150K total hardware cost

**Re-issuance:** If a voter loses their code, they return to the municipal office, re-verify identity, and receive a **new** random code. The old code hash is replaced with the new one. The system logs the re-issuance event. Since the old code is irrecoverable (never stored), any attacker who previously obtained it through social engineering is locked out once the voter re-issues. A voter who has already cast a ballot with a token derived from the old code must re-authenticate and re-vote with their new code (the re-voting mechanism in Section 4.3 handles this seamlessly).

Alternative: **Online via eID**
- Citizen authenticates with QES certificate
- Backend generates a fresh random code (same process as in-person)
- Code displayed on screen (once only) or sent to registered email
- If received online, no physical visit needed

**Election day:** Citizen uses ЕГН + code to authenticate

**Advantages over pre-printed sealed cards:**
- Eliminates printing, sorting, shipping, and storing 6.5M sealed cards
- No physical supply chain to attack (no interception, no tampering with envelopes)
- No waste from unclaimed cards
- No persistent secret material: unlike sealed cards (physical) or HSM-derived codes (master secret), random codes are never stored and cannot be reconstructed by anyone — including the system operator
- **Cryptographic ballot secrecy guarantee:** since the Identity Provider cannot recover any voter's code after issuance, it cannot compute voter tokens (T = H(EGN || code || election_id || "token")), and therefore cannot link identities to ballots on the bulletin board. Ballot secrecy depends on cryptography, not on trusting the IdP operator.
---

## 12. INFRASTRUCTURE TOPOLOGY

```
                            ┌──────────────┐
                            │   VOTERS     │
                            │  (browsers   │
                            │   + apps)    │
                            └──────┬───────┘
                                   │ HTTPS
                            ┌──────▼───────┐
                            │  SOVEREIGN   │  Static web app
                            │  EDGE CACHE  │  (HTML+JS+WASM)
                            │  (BG state   │
                            │   data ctrs  │  Domestic: Sofia, Plovdiv,
                            │   + EU PoPs  │  Varna, Burgas edge nodes
                            │   for dias-  │  Diaspora: EU PoPs (Frankfurt,
                            │   pora)      │  Amsterdam, London, Chicago)
                            └──────┬───────┘
                                   │
                    ┌──────────────┼──────────────┐
                    │              │               │
             ┌──────▼──────┐ ┌────▼─────┐  ┌──────▼──────┐
             │  IDENTITY   │ │ VOTING   │  │  BULLETIN   │
             │  PROVIDER   │ │ SERVER   │  │  BOARD      │
             │  CLUSTER    │ │ CLUSTER  │  │  PRIMARY    │
             │             │ │          │  │             │
             │ (3+ nodes   │ │(3+ nodes │  │(3+ nodes    │
             │  behind LB) │ │ behind LB│  │ + mirrors)  │
             └─────────────┘ └──────────┘  └──────┬──────┘
                                                   │ WebSocket
                                           ┌───────┴────────┐
                                           │   BB MIRRORS    │
                                           │   (parties,     │
                                           │    NGOs, press) │
                                           └────────────────┘
```

Hosting: All backend services hosted on Bulgarian sovereign infrastructure (state data
centers operated by "Информационно обслужване" or equivalent state IT entity).
Static client served from edge cache nodes in Bulgarian data centers for domestic voters
and EU points of presence for diaspora. NO foreign commercial CDN (Cloudflare, AWS
CloudFront, etc.) in the critical path — a foreign entity must never control what code
voters receive. Mirrors run on each observer's own infrastructure.
Edge cache integrity: Each edge node serves pre-signed static assets. The signing key
is held by ЦИК. Voters and mirrors independently verify asset signatures match the
published build hash (Section 7.2). A compromised edge node cannot serve modified
code without detection.
---

### 12.1 DIASPORA VOTING — MIR 32
Bulgarian citizens residing abroad vote in MIR 32 (Чужбина) under different rules than domestic MIRs. The online voting system must fully support this.
#### 12.1.1 Ballot Structure
MIR 32 has its own registered party lists and candidates, distinct from domestic MIRs.
The ballot matrix follows the same encoding (Section 3.1) but uses MIR 32 party/candidate lists.
The Identity Provider uses a dedicated MIR 32 signing key (the 32nd key).
#### 12.1.2 Eligibility & Identity Verification
Diaspora voters are registered in the voter roll for MIR 32 based on:
- Prior registration at a Bulgarian consulate/embassy, OR
- Registration via the ЦИК online portal (existing process, with QES or consular verification)
Identity code distribution for diaspora:
- Path A (eID): Diaspora voters with valid Bulgarian QES certificates authenticate
  online and receive their identity code electronically (same as domestic Path A)
- Path B (Consular): Voters registered at a consulate visit in person, present ID,
  and receive their code via a voter-facing device at the consular desk (same
  on-demand generation protocol as domestic municipal distribution, Section 11)
#### 12.1.3 Time Zone Handling
Online voting for MIR 32 follows Bulgarian time (EET/EEST):
- Voting opens and closes at the same absolute time as domestic voting
- The web client displays both local time and Bulgarian time for diaspora voters
- Voters in western time zones (Americas) effectively have a longer "evening" window;
  voters in eastern time zones (Asia/Australia) have an early-morning start
- In-person override at consular polling stations follows the local opening hours
  of each consular station (set by ЦИК per consulate, typically 07:00-20:00 local)
#### 12.1.4 In-Person Override for Diaspora
Diaspora voters who voted online can override at their registered consular polling station.
The override protocol is identical to domestic (Section 4.4), with the consular election
commission performing the same role as the domestic polling station commission.
The "online voter" register is synchronized to all consular polling stations.
---

## 13. LEGAL & REGULATORY FRAMEWORK

Deploying online voting in Bulgaria requires parallel legal and technical workstreams. The system cannot go to production without the following:

### 13.1 Electoral Code Amendments

The Bulgarian Electoral Code (Изборен кодекс) must be amended to:

| Area | Required Change |
|---|---|
| Voting channel | Explicitly authorize online voting as a legal voting channel alongside paper and machine voting |
| Legal standing | Define that online ballots have equal legal weight to paper ballots |
| In-person override | Codify the right of an online voter to cancel their online ballot by voting in person |
| Re-voting | Define the legal status of re-voting (currently no precedent in Bulgarian law) |
| Bulletin board | Give legal standing to the BB as the official record of online votes |
| Trustee obligations | Define the legal duties, liabilities, and appointment process for the 9 trustees |
| Certification | Establish a certification process for online voting systems (similar to machine voting certification under Art. 213a) |
| Dispute resolution | Define procedures for challenges based on cryptographic evidence (receipt mismatches, ZKP failures) |
| Diaspora | Amend MIR 32 regulations to cover online voting from abroad |

### 13.2 Constitutional Considerations
- Article 10 of the Bulgarian Constitution guarantees secret ballot — the system's cryptographic ballot secrecy must be argued as satisfying this requirement
- The Constitutional Court may need to rule on whether online voting is compatible with the Constitution before deployment
- Precedent: The Constitutional Court has previously ruled on machine voting (Decision 3/2017) — similar scrutiny is expected

### 13.3 Data Protection (GDPR / ЗЗЛД)
- EGN processing must comply with GDPR Article 9 (special categories — political opinions are inferred from voting)
- Data Protection Impact Assessment (DPIA) required before deployment
- The КЗЛД (Commission for Personal Data Protection) must approve the data flows
- Key principle: The system architecture already minimizes data exposure (blind signatures, no individual decryption) — this is a strong GDPR compliance argument
- Retention policy: BB data (encrypted, unlinkable) retained permanently as public record. Identity Provider authentication logs retained for the statutory challenge period (typically 1 year), then deleted.

### 13.4 Interaction with Existing Voting Infrastructure
- Online voting supplements (does not replace) paper and machine voting
- The voter roll must track which channel each voter used to prevent double-voting
- The in-person override mechanism (Section 4.4) ensures paper voting takes precedence
- Results from online and offline channels are tallied separately and then combined per MIR
- ЦИК must publish per-channel breakdowns for transparency
---

## 14. PROJECT PHASES & TIMELINE

| Phase | Duration | Deliverables |
|---|---|---|
| Phase 0: Legal workstream | Months 1-18 (parallel) | Electoral Code amendments drafted, parliamentary review, Constitutional Court consultation, GDPR DPIA, КЗЛД approval |
| Phase 1: Protocol spec | Months 1-3 | Formal cryptographic protocol document, threat model, external academic review |
| Phase 2: Crypto core | Months 3-7 | Rust crypto crate (`#[no_std]`, P-256 from first principles), compiles to both native and WASM, comprehensive test suite, formal verification of critical proofs |
| Phase 3: Backend services | Months 7-11 | Identity Provider, Voting Server, Bulletin Board, Tally service (all Rust + axum), in-person override protocol |
| Phase 4: Client apps | Months 11-14 | Web client (React + Rust→WASM from shared crypto crate), Mobile client (React Native), Trustee CLI tool (Rust), accessibility compliance (WCAG 2.1 AA) |
| Phase 5: Verifier & tools | Months 14-16 | Independent verifier, BB mirror software, admin tools, diaspora edge infrastructure |
| Phase 6: Security audit | Months 16-22 | External audit by 2+ independent firms (minimum 4 months active audit), public bug bounty (ongoing), penetration testing, formal verification review |
| Phase 7: Public trust campaign | Months 20-24 | Public education campaign, technical documentation in Bulgarian, demonstration events, media briefings, open-source community engagement |
| Phase 8: Pilot election (non-binding) | Months 22-26 | Parallel non-binding pilot alongside a real election (voters cast online AND paper, results compared but online results are advisory only). Public post-mortem. |
| Phase 9: Pilot election (binding, small-scale) | Months 26-30 | Binding pilot for a small election (e.g., municipal by-election). Full legal standing. Post-mortem and system refinement. |
| Phase 10: Certification & national deployment | Months 30-36 | ЦИК formal certification, load testing at national scale (7M+ voters), key ceremony rehearsal, disaster recovery drill, national deployment for parliamentary election |
Total: ~36 months to production-ready
Note: The legal workstream (Phase 0) runs in parallel with technical development. If legal authorization is delayed, technical phases proceed but deployment is gated on legal readiness. The two non-binding and binding pilot phases provide empirical evidence for both technical reliability and public trust before national-scale deployment.
---

## 15. FALLBACK & DEGRADATION PLAN

Online voting is supplementary to paper/machine voting. The system must degrade gracefully without disenfranchising voters.

### 15.1 Failure Scenarios

| Scenario | Detection | Response |
|---|---|---|
| Voting Server fully down | Health checks fail, voters see error | Display message: "Online voting temporarily unavailable. Please retry or vote in person." All voters retain in-person voting right. Extended online voting window (up to 2 hours post-close) if downtime exceeds 1 hour during voting day — requires ЦИК emergency decision. |
| Identity Provider down | Auth requests fail | Voters who already have tokens can still submit ballots. New authentications queued with retry. Voters without tokens directed to in-person voting. |
| Bulletin Board primary down | Mirrors detect missing heartbeat | Voting Server buffers entries locally (up to 1 hour). When BB recovers, buffered entries are published with original timestamps. If BB is down >1 hour, voting continues but individual verifiability is delayed (voters receive receipts but cannot check BB until recovery). Mirrors continue serving read requests from last-known state. |
| BB equivocation detected | Mirrors report divergent chain heads | CRITICAL: Voting is immediately halted. ЦИК convenes emergency session. Divergent chains are published for public audit. Voting resumes only after root cause is identified and the canonical chain is determined by trustee majority vote. |
| Fewer than 5 trustees available for decryption | Trustees fail to appear at ceremony | Decryption ceremony postponed (legal deadline: 48 hours post-election close). ЦИК contacts missing trustees. If 5 cannot be assembled within 48 hours, the election's online component is declared failed and only paper/machine results are certified. |
| DDoS attack on infrastructure | Traffic anomaly detection | Sovereign edge nodes absorb load. Voting Server rate-limits by token (1 submission per 10 seconds per token). If sustained, extend voting window. BB mirrors are unaffected (distributed infrastructure). |
| Widespread client compromise (malware) | Benaloh challenge failures spike, community reports | ЦИК issues public advisory. Voters directed to: (a) use a different device, (b) boot from official live USB image, or (c) vote in person. If compromise is systemic, ЦИК may suspend online voting by emergency decision. |

### 15.2 Election Day Communication
- A public status dashboard (status.glasuvai.bg) shows real-time system health
- ЦИК maintains a communication channel (SMS, social media, TV) for emergency announcements
- All status changes are logged on the BB for post-election audit

### 15.3 Post-Election Disputes

If a voter's receipt does not match the BB:

1. Voter files a complaint with ЦИК, presenting their receipt hash
2. ЦИК verifies receipt against BB entries
3. If mismatch confirmed: this constitutes cryptographic proof of ballot suppression or tampering
4. Legal consequence: grounds for partial or full invalidation of the online vote in the affected MIR (per Electoral Code amendments, Section 13.1)
---

## 16. ACCESSIBILITY & INCLUSIVITY

### 16.1 Web Accessibility (WCAG 2.1 AA Compliance)
The web client must meet WCAG 2.1 Level AA as a minimum:
- Full keyboard navigation (no mouse required)
- Screen reader support (ARIA labels, semantic HTML) — tested with NVDA, JAWS, VoiceOver
- High-contrast mode and configurable font sizes
- No time limits on ballot construction (only the election window itself is timed)
- Clear error messages with recovery instructions
- No CAPTCHA (token-based authentication replaces bot prevention)

### 16.2 Language Support
- Primary: Bulgarian (български)
- Secondary: Turkish (türkçe) — significant minority, required under anti-discrimination law
- Tertiary: English — for diaspora voters
- All UI strings externalized; additional languages can be added without code changes

### 16.3 Assisted Voting
- Voters with disabilities may be assisted by a person of their choice (existing Electoral Code right)
- The system logs that assisted voting mode was used (for audit) but not the assistant's identity
- Audio ballot option: screen reader reads party names and candidates; voter confirms by keyboard

### 16.4 Low Digital Literacy Accommodation
- Municipal offices offer supervised kiosks where voters can cast online votes with staff assistance
  (staff assists with device operation but CANNOT see the ballot — privacy screen + voter confirms alone)
- Printed step-by-step guides available at municipal offices and distributed with code receipts
- Video tutorials on glasuvai.bg in Bulgarian and Turkish
- Phone helpline (non-technical) for authentication and navigation issues

### 16.5 Device Requirements
- Minimum: Any modern browser (Chrome/Firefox/Safari/Edge, last 2 major versions)
- Mobile: Android 8+ / iOS 14+
- No app installation required for web voting (progressive web app fallback)
- Estimated data usage per vote: ~500 KB (with Rust WASM crypto module)
---

## 17. SUMMARY OF KEY DESIGN DECISIONS

| Decision | Choice | Rationale |
|---|---|---|
| Encryption | Exponential ElGamal on P-256 | Additively homomorphic, well-studied, government-approved curve |
| Tallying | Homomorphic (not mix-net) | Simpler, sufficient for party-list + preference |
| Verifiability | Benaloh challenge + receipt hash + public BB | Full E2E verifiability |
| Coercion resistance | Re-voting + in-person override | Re-voting for online correction; paper ballot at polling station cancels online vote (Estonian model). Strongest practical measure for Bulgaria's vote-buying context. |
| Identity unlinking | RSA blind signatures | Proven, simple, well-understood |
| Token derivation | H(EGN \|\| identity_code \|\| election_id \|\| "token") | Deterministic (enables re-vote overwrite) but includes voter-held secret (identity_code) preventing brute-force reversal from public voter roll |
| Trust model | 5-of-9 multi-stakeholder threshold | No single faction controls decryption |
| Bulletin board | Hash-chained log + Merkle tree + mirrors | Hash chain for append-only integrity; Merkle tree for efficient inclusion proofs; mirrors for decentralized verification |
| Build verification | Nix reproducible builds + runtime hash | Any voter can verify code integrity |
| Client crypto | Rust→WASM (from shared `packages/crypto` crate) | ~500 KB binary. Same code as server — one implementation, no cross-language sync needed. Crypto crate is `#[no_std]`, zero external crates. |
| Hosting | Bulgarian sovereign infrastructure | No foreign entity in the critical path. State data centers + EU PoPs for diaspora. |
| Seat allocation | Hare-Niemeyer + 7% preference threshold | Matches Bulgarian Electoral Code |
| Legal framework | Electoral Code amendments + Constitutional Court review | Online voting requires explicit legal authorization and constitutional compatibility ruling |
| Diaspora | Full MIR 32 support | Consular code distribution, EU edge nodes, consular override stations |
| Fallback | Graceful degradation to paper voting | Every failure mode has a defined response; paper voting is always available as override |
| Timeline | 36 months with two pilot rounds | Non-binding parallel pilot → binding small-scale pilot → national deployment |
---
Do you want me to proceed with implementation? If so, I'd suggest starting with the crypto core library (/packages/crypto) since everything else depends on it. The Rust crypto crate is `#[no_std]`-compatible and shared between all server components and the WASM client. Or if you'd like to refine any part of this architecture first — the identity code distribution, the ballot encoding, the re-voting mechanism, or anything else — let me know.
