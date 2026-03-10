# Task Completion Checklist

After completing any code change:

1. **Format**: `cargo fmt --all`
2. **Lint**: `cargo clippy --workspace --all-targets -- -D warnings`
3. **Test**: `cargo test --workspace` (or `-p <package>` for single package)
4. **For WASM changes**: `wasm-pack build packages/crypto-wasm --target web`
5. **For Nix changes**: `nix flake check`

If updating dependencies:
- Always commit `flake.lock` and `Cargo.lock` together
- Run `cargo audit` to check for vulnerabilities

All checks should pass with zero errors and zero warnings before committing.
