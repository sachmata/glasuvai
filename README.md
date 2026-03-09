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

See [PLAN.md](PLAN.md) for the full architecture design.

## Reproducible Builds with Nix

For a voting system, **reproducible builds are not optional** — any voter, auditor, or
observer must be able to independently verify that the deployed binaries match the
published source code. Glasuvai achieves this through Nix flakes.

### Why Nix

Every input to the build is content-addressed and pinned to an exact revision:

| Input | Pinned where | What it locks |
|---|---|---|
| **Rust compiler** | `rust-toolchain.toml` → `1.85.0` | Exact `rustc` + `cargo` version, WASM target, components |
| **System libraries** | `flake.lock` → nixpkgs `nixos-24.11` @ `50ab7937…` | `sqlite`, `openssl`, `pkg-config`, glibc — every `.so` |
| **Rust crates** | `Cargo.lock` | Every dependency version + integrity checksum |
| **Node.js** | `flake.nix` → `nodejs_22` from locked nixpkgs | Exact Node.js binary |
| **Rust overlay** | `flake.lock` → `oxalica/rust-overlay` @ pinned rev | Exact toolchain distribution |

Two people building the same Git commit with the same `flake.lock` get
**bit-for-bit identical outputs** — regardless of host OS, installed packages,
or environment variables.

### Prerequisites

Install Nix with flakes enabled:

```bash
# Install Nix (multi-user, recommended)
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh

# Or if you already have Nix, enable flakes:
# ~/.config/nix/nix.conf
#   experimental-features = nix-command flakes
```

No other dependencies are needed — Nix provides everything.

### Development Shell

```bash
# Enter the reproducible dev shell — all tools are pinned
nix develop

# What you get (exact versions from flake.lock):
#   rustc 1.85.0, cargo, clippy, rustfmt
#   wasm-pack, cargo-audit, cargo-deny
#   node 22, pkg-config, sqlite, openssl
#   nil (Nix LSP), nixpkgs-fmt

# Build, test, lint — all inside the pinned environment
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
wasm-pack build packages/crypto-wasm --target web
```

### Building Packages

```bash
# Build a specific binary — output goes to ./result/
nix build .#glasuvai-verifier
nix build .#glasuvai-admin
nix build .#bulletin-board
nix build .#voting-server

# Build all packages
nix build .

# Show the exact derivation (for audit)
nix derivation show .#glasuvai-verifier
```

### Verifying Reproducibility

Anyone can verify that a binary matches the source:

```bash
# Step 1: Check out the exact commit used to build the deployed binary
git checkout <release-tag>

# Step 2: Build locally — Nix guarantees deterministic output
nix build .#glasuvai-verifier

# Step 3: Compare the hash
sha256sum ./result/bin/glasuvai-verifier
# Must match the published hash for that release
```

For CI, this can be automated:

```bash
# In CI: build and assert the output hash matches the expected value
nix build .#glasuvai-verifier
OUT_HASH=$(nix hash path ./result)
[[ "$OUT_HASH" == "$EXPECTED_HASH" ]] || exit 1
```

### How the Pin Chain Works

```
rust-toolchain.toml          ← Rust version (single source of truth)
        │
        ▼
flake.nix                    ← References rust-toolchain.toml + nixpkgs
        │
        ▼
flake.lock                   ← Content hashes for ALL inputs
        │                       (nixpkgs, rust-overlay, flake-utils)
        ▼
Cargo.lock                   ← Exact crate versions + checksums
        │
        ▼
nix build .#<package>        ← Deterministic output
```

- `flake.lock` is auto-generated and committed. It records the exact Git revision
  and NAR hash of every flake input.
- `Cargo.lock` is committed and maps every crate to a version + checksum.
- `rust-toolchain.toml` is read by both `flake.nix` (via `fromRustupToolchainFile`)
  and `Dockerfile.dev` — one file controls the Rust version everywhere.

### Updating Dependencies

```bash
# Update all flake inputs to latest matching versions
nix flake update

# Update only nixpkgs (e.g., for security patches)
nix flake update nixpkgs

# Update Cargo dependencies
cargo update

# After updating, always commit the new lock files:
git add flake.lock Cargo.lock
git commit -m "chore: update dependencies"
```

> **Rule:** `flake.lock` and `Cargo.lock` are always committed.
> Never `.gitignore` them — they are the reproducibility guarantee.

### CI Checks

The flake defines a `checks` output for CI integration:

```bash
# Run all checks (clippy, fmt, tests) — in the exact pinned environment
nix flake check

# Equivalent to running in CI:
#   cargo clippy --workspace -- -D warnings
#   cargo fmt --all -- --check
#   cargo test --workspace
# But guaranteed to use the same toolchain as every other builder.
```

## Development Setup (Alternative: Docker)

For contributors who don't have Nix installed, a Docker-based dev environment
mirrors the Nix devShell tooling. It reads `rust-toolchain.toml` for the Rust
version but uses Debian packages for system libraries (close to, but not
bit-for-bit identical with Nix builds).

```bash
# Build the dev image
docker compose build dev

# Start interactive dev shell
./scripts/docker-dev.sh shell

# Inside the container — full toolchain available:
#   rustc 1.85.0, cargo, wasm-pack, node 22, clippy, rustfmt
cargo test --workspace
cargo clippy --workspace
wasm-pack build packages/crypto-wasm --target web

# Helper script shortcuts
./scripts/docker-dev.sh test     # cargo test
./scripts/docker-dev.sh fmt      # cargo fmt
./scripts/docker-dev.sh clippy   # clippy
./scripts/docker-dev.sh wasm     # build crypto-wasm
./scripts/docker-dev.sh clean    # stop & remove volumes
```

Docker volumes cache `cargo registry`, `target/`, and `sccache` across restarts
for fast rebuilds. Service containers (bulletin-board, voting-server, etc.) are
defined in `docker-compose.yml` and can be uncommented as packages are implemented.

> **Note:** Docker builds share `rust-toolchain.toml` with Nix for the Rust
> version, but official reproducibility verification should always use
> `nix build`.

## License

[Apache License 2.0](LICENSE)
