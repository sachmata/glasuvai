## 17. SUMMARY OF KEY DESIGN DECISIONS

| Decision | Choice | Rationale |
|---|---|---|
| Encryption | Exponential ElGamal on P-256 | Additively homomorphic, well-studied, government-approved curve |
| Tallying | Homomorphic (not mix-net) | Simpler, sufficient for party-list + preference |
| Verifiability | Benaloh challenge + receipt hash + public BB | Full E2E verifiability |
| Coercion resistance | Re-voting + in-person override | Re-voting for online correction; paper ballot at polling station cancels online vote (Estonian model). Strongest practical measure for Bulgaria's vote-buying context. |
| Identity unlinking | RSA blind signatures | Proven, simple, well-understood |
| Token derivation | H(EGN \|\| identity_code \|\| election_id \|\| "token") | Deterministic (enables re-vote overwrite) but includes voter-held secret (identity_code) preventing brute-force reversal from public voter roll |
| Trust model | 5-of-9 multi-stakeholder threshold | No single faction controls decryption |
| Bulletin board | Hash-chained log + Merkle tree + mirrors | Hash chain for append-only integrity; Merkle tree for efficient inclusion proofs; mirrors for decentralized verification |
| Build verification | Nix reproducible builds + runtime hash | Any voter can verify code integrity |
| Client crypto | Rust→WASM (from shared `packages/crypto` crate) | ~500 KB binary. Same code as server — one implementation, no cross-language sync needed. Crypto crate is `#[no_std]`, zero external crates. |
| Election data | `packages/election` crate with TOML data under `data/` | Election domain types + real ballot data separated from crypto crate. TOML files are human-readable, git-diffable. Feature flags select which election is embedded (e.g., `bg-na51-2024`). SHA-256 integrity digest computed at build time for anti-tampering. |
| Hosting | Bulgarian sovereign infrastructure | No foreign entity in the critical path. State data centers + EU PoPs for diaspora. |
| Seat allocation | Hare-Niemeyer + 7% preference threshold | Matches Bulgarian Electoral Code |
| Legal framework | Electoral Code amendments + Constitutional Court review | Online voting requires explicit legal authorization and constitutional compatibility ruling |
| Diaspora | Full MIR 32 support | Consular code distribution, EU edge nodes, consular override stations |
| Fallback | Graceful degradation to paper voting | Every failure mode has a defined response; paper voting is always available as override |
| Timeline | 36 months with two pilot rounds | Non-binding parallel pilot → binding small-scale pilot → national deployment |
---
Do you want me to proceed with implementation? If so, I'd suggest starting with the crypto core library (`/packages/crypto`) since everything else depends on it, followed by the election domain crate (`/packages/election`) which provides the types and real ballot data that all services consume. The crypto crate is `#[no_std]`-compatible with zero external dependencies; the election crate embeds real ballot data (TOML files under `data/`) with feature flags per election and a SHA-256 anti-tampering digest. Or if you'd like to refine any part of this architecture first — the identity code distribution, the ballot encoding, the re-voting mechanism, or anything else — let me know.
