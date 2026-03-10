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
