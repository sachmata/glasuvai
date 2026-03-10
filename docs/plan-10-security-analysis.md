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
