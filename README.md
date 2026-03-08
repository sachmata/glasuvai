# Glasuvai

**Verifiable online voting system for Bulgarian elections.**

> **This is an experimental research repository.** The code and architecture
> are under active development and have not been audited, certified, or
> approved for use in real elections. Do not deploy in production.

## Overview

Glasuvai explores a cryptographically verifiable online voting system designed
for Bulgarian parliamentary elections. Core properties:

- **Ballot secrecy** — homomorphic encryption (ElGamal on Ristretto255); votes are never decrypted individually
- **E2E verifiability** — public bulletin board, Benaloh challenges, receipt hashes, Merkle inclusion proofs
- **No single point of trust** — 5-of-9 threshold decryption across independent stakeholders
- **Coercion resistance** — online re-voting + in-person paper ballot override
- **Identity unlinking** — RSA blind signatures separate identity from vote
- **Open source** — reproducible Nix builds, runtime attestation

## Repository Structure

```
packages/
  crypto/                Go — core cryptographic library
  crypto-wasm/           Rust → WASM — client-side crypto
  identity-provider/     Go — authentication + blind signatures
  voting-server/         Go — ballot acceptance
  bulletin-board/        Go — append-only public ledger
  tally/                 Go — homomorphic aggregation + threshold decryption
  verifier/              Go — independent election verification CLI
  trustee-tool/          Go — key ceremony + partial decryption CLI
  web-client/            TypeScript + React — voter UI
  mobile-client/         React Native — mobile voter UI
  admin/                 TypeScript — election setup tools
nix/                     Reproducible build definitions
docs/                    Protocol specification
test/                    End-to-end test suite
```

See [plan.md](plan.md) for the full architecture design.

## License

[Apache License 2.0](LICENSE)
