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

