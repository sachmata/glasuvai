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
