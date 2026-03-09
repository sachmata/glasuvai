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
rust-toolchain.toml      Pins Rust 1.85.0 + WASM target
flake.nix                Nix flake — pins all build inputs
flake.lock               Exact commit hashes for all Nix inputs
.cargo/config.toml       Cargo workspace settings
Cargo.toml               Workspace root
Cargo.lock               Pinned Cargo dependency versions

packages/
  crypto/                Rust #[no_std] — core cryptographic library
  crypto-wasm/           Rust → WASM — client-side crypto bridge
  identity-provider/     Rust — authentication + blind signatures
  voting-server/         Rust — ballot acceptance
  bulletin-board/        Rust — append-only public ledger
  tally/                 Rust — homomorphic aggregation + threshold decryption
  verifier/              Rust — independent election verification CLI
  trustee-tool/          Rust — key ceremony + partial decryption CLI
  web-client/            TypeScript + React — voter UI
  mobile-client/         React Native — mobile voter UI
  admin/                 Rust — election setup tools
nix/                     Additional Nix build definitions
docs/                    Protocol specification
test/                    End-to-end test suite
```

See [plan.md](plan.md) for the full architecture design.

## Reproducible Builds

All build inputs are pinned for verifiability:

- **Rust compiler**: `rust-toolchain.toml` → Rust 1.85.0
- **System dependencies**: `flake.lock` → nixpkgs 24.11 LTS (exact commit)
- **Cargo crates**: `Cargo.lock` → exact versions + checksums
- **Node.js**: Nix flake → Node.js 22 LTS

```bash
# Enter reproducible dev shell (all tools pinned)
nix develop

# Build any package — identical output for same commit
nix build .#glasuvai-verifier
```

## License

[Apache License 2.0](LICENSE)
