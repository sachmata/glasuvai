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
