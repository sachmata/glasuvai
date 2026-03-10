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
