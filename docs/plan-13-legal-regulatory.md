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
