# Code Style and Conventions

## Rust
- **Edition**: Infer from workspace (Rust 1.85.0)
- `#[no_std]` compatible for the `crypto` crate — zero external crypto crates
- Standard `rustfmt` formatting (enforced via `cargo fmt`)
- `clippy` with `-D warnings` — all warnings are errors
- `snake_case` for functions/variables, `PascalCase` for types/traits, `SCREAMING_SNAKE_CASE` for constants
- Modules mirror filesystem (e.g., `src/elgamal.rs`, `src/zkp.rs`)
- No external crates for crypto — every operation traceable to textbook definitions

## TypeScript / React (web-client, mobile-client)
- TypeScript strict mode
- React functional components
- `wasm_bridge.ts` for WASM interop (bridges to Rust crypto)
- Component files: PascalCase (e.g., `BallotForm.tsx`, `Verify.tsx`)

## General
- All cryptographic code lives in `packages/crypto` (Rust `#[no_std]`)
- WASM wrapper `packages/crypto-wasm` is a thin bindgen shim only
- `flake.lock` and `Cargo.lock` are ALWAYS committed
- Conventional commits style: `feat:`, `fix:`, `chore:`, `docs:`, `refactor:`, `test:`
