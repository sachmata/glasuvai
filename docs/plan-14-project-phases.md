## 14. PROJECT PHASES & TIMELINE

| Phase | Duration | Deliverables |
|---|---|---|
| Phase 0: Legal workstream | Months 1-18 (parallel) | Electoral Code amendments drafted, parliamentary review, Constitutional Court consultation, GDPR DPIA, КЗЛД approval |
| Phase 1: Protocol spec | Months 1-3 | Formal cryptographic protocol document, threat model, external academic review |
| Phase 2: Crypto core | Months 3-7 | Rust crypto crate (`#[no_std]`, P-256 from first principles), compiles to both native and WASM, comprehensive test suite, formal verification of critical proofs |
| Phase 3: Backend services | Months 7-11 | Identity Provider, Voting Server, Bulletin Board, Tally service (all Rust + axum), in-person override protocol |
| Phase 4: Client apps | Months 11-14 | Web client (React + Rust→WASM from shared crypto crate), Mobile client (React Native), Trustee CLI tool (Rust), accessibility compliance (WCAG 2.1 AA) |
| Phase 5: Verifier & tools | Months 14-16 | Independent verifier, BB mirror software, admin tools, diaspora edge infrastructure |
| Phase 6: Security audit | Months 16-22 | External audit by 2+ independent firms (minimum 4 months active audit), public bug bounty (ongoing), penetration testing, formal verification review |
| Phase 7: Public trust campaign | Months 20-24 | Public education campaign, technical documentation in Bulgarian, demonstration events, media briefings, open-source community engagement |
| Phase 8: Pilot election (non-binding) | Months 22-26 | Parallel non-binding pilot alongside a real election (voters cast online AND paper, results compared but online results are advisory only). Public post-mortem. |
| Phase 9: Pilot election (binding, small-scale) | Months 26-30 | Binding pilot for a small election (e.g., municipal by-election). Full legal standing. Post-mortem and system refinement. |
| Phase 10: Certification & national deployment | Months 30-36 | ЦИК formal certification, load testing at national scale (7M+ voters), key ceremony rehearsal, disaster recovery drill, national deployment for parliamentary election |
Total: ~36 months to production-ready
Note: The legal workstream (Phase 0) runs in parallel with technical development. If legal authorization is delayed, technical phases proceed but deployment is gated on legal readiness. The two non-binding and binding pilot phases provide empirical evidence for both technical reliability and public trust before national-scale deployment.
