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

