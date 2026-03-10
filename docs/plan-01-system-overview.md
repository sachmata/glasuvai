## 1. SYSTEM OVERVIEW

### 1.1 Design Principles

| Principle | How it's achieved |
|---|---|
| No single point of trust | Threshold cryptography — 5-of-9 trustees from different stakeholders |
| Individual verifiability | Voter receives a receipt; Merkle inclusion proof against public BB |
| Universal verifiability | Public bulletin board + open-source verifier anyone can run |
| Ballot secrecy | Homomorphic encryption — votes never decrypted individually; token derivation includes voter-held secret preventing brute-force deanonymization |
| Coercion resistance | Online re-voting + in-person paper ballot override (Estonian model) |
| Transparency | 100% open source, reproducible builds, runtime attestation |
| Simplicity | Minimal moving parts; Rust + TypeScript; no blockchain. One crypto implementation shared between server and client (via WASM). |
| Sovereignty | All infrastructure on Bulgarian state systems; no foreign CDN |
| Accessibility | WCAG 2.1 AA, multilingual (BG/TR/EN), assisted voting support |
| Graceful degradation | Every failure mode documented; paper voting always available as fallback |

### 1.2 Trust Model — Five Independent Roles
No two roles should be controlled by the same entity:

```
┌─────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│  REGISTRAR   │  │   IDENTITY   │  │   VOTING     │  │  BULLETIN    │  │   TRUSTEES   │
│  (ЦИК/CEC)  │  │   PROVIDER   │  │   SERVER     │  │   BOARD      │  │  (9 parties) │
│              │  │              │  │              │  │              │  │              │
│ Manages      │  │ Validates    │  │ Accepts      │  │ Append-only  │  │ Hold key     │
│ voter roll   │  │ identity,    │  │ encrypted    │  │ public       │  │ shares for   │
│ per MIR      │  │ issues blind │  │ ballots,     │  │ ledger of    │  │ threshold    │
│              │  │ signatures   │  │ checks       │  │ all ballots  │  │ decryption   │
│              │  │              │  │ signatures   │  │              │  │              │
└─────────────┘  └──────────────┘  └──────────────┘  └──────────────┘  └──────────────┘
```

Key separation: The Identity Provider knows WHO voted but not HOW. The Voting Server sees encrypted ballots but not WHO cast them. Neither alone can break secrecy.

### 1.3 Trustee Composition (5-of-9 threshold)

| Seat | Holder |
|---|---|
| 1 | ЦИК (Central Election Commission) |
| 2-4 | Three largest parliamentary parties/coalitions |
| 5-6 | Two opposition parties (next largest) |
| 7 | Bulgarian Academy of Sciences (technical auditor) |
| 8 | Registered civil society / NGO observer |
| 9 | International technical auditor (e.g., OSCE-nominated) |
Any 5 of 9 can perform decryption. No political faction holds 5 seats.

