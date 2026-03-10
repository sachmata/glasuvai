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
