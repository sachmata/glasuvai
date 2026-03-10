# Suggested Commands

## Development Environment

### Preferred: Nix dev shell (fully pinned, reproducible)
```bash
nix develop                          # Enter reproducible dev shell
```

### Alternative: Docker
```bash
docker compose build dev             # Build dev image
./scripts/docker-dev.sh shell        # Start interactive dev shell
./scripts/docker-dev.sh test         # cargo test
./scripts/docker-dev.sh fmt          # cargo fmt
./scripts/docker-dev.sh clippy       # cargo clippy
./scripts/docker-dev.sh wasm         # wasm-pack build
./scripts/docker-dev.sh clean        # stop & remove volumes
```

## Build
```bash
cargo build --workspace              # Build all Rust crates
cargo build -p <package>             # Build specific package

nix build .#glasuvai-verifier        # Nix reproducible build
nix build .#glasuvai-admin
nix build .#bulletin-board
nix build .#voting-server
nix build .                          # Build all packages

wasm-pack build packages/crypto-wasm --target web   # Build WASM
```

## Test
```bash
cargo test --workspace               # Run all tests
cargo test -p <package>              # Test specific package
```

## Lint & Format
```bash
cargo clippy --workspace --all-targets -- -D warnings   # Lint (warnings = errors)
cargo fmt --all                                         # Format all
cargo fmt --all -- --check                              # Check formatting (CI)
```

## CI
```bash
nix flake check                      # Run all CI checks (clippy, fmt, test)
```

## Dependency Management
```bash
nix flake update                     # Update all Nix inputs
nix flake update nixpkgs             # Update only nixpkgs
cargo update                         # Update Cargo dependencies
cargo audit                          # Scan for vulnerabilities
cargo deny check                     # License + advisory check

# Always commit lock files after updating:
git add flake.lock Cargo.lock && git commit -m "chore: update dependencies"
```

## Verification
```bash
sha256sum ./result/bin/<binary>      # Compare binary hash
nix hash path ./result               # Get Nix output hash
nix derivation show .#<package>      # Show full derivation (for audit)
```
