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
