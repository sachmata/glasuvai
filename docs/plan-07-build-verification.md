## 7. BUILD VERIFICATION & CODE TRANSPARENCY

### 7.1 Reproducible Builds

```
┌─────────────────────────────────────────────────────────────┐
│                   BUILD PIPELINE                              │
│                                                               │
│  Source code: GitHub public repo (glasuvai/glasuvai)           │
│  Commit: tagged release, signed by ≥3 maintainers            │
│                                                               │
│  Build: Nix-based reproducible build                         │
│  ┌─────────────────────────────────────────────┐             │
│  │  nix build .#voting-server                   │             │
│  │  nix build .#voting-client-web               │             │
│  │  nix build .#voting-client-mobile            │             │
│  │  nix build .#bulletin-board                  │             │
│  │  nix build .#verifier                        │             │
│  │                                               │             │
│  │  Output: deterministic hash per artifact      │             │
│  │  Anyone building from same commit gets        │             │
│  │  identical binary hash                        │             │
│  └─────────────────────────────────────────────┘             │
│                                                               │
│  Attestation: Multiple independent parties build             │
│  from source and publish their output hashes.                │
│  If all match → build is verified.                           │
└─────────────────────────────────────────────────────────────┘
```

### 7.2 Runtime Attestation (for voters)

Every voter visiting the voting website can verify they're running the correct code:

```
┌────────────────────────────────────────────────────────────────┐
│  VOTER'S BROWSER                                                │
│                                                                  │
│  The voting web app is a STATIC SITE (HTML + JS + WASM)         │
│  served from sovereign edge cache nodes. No server-side          │
│  rendering.                                                      │
│                                                                  │
│  Verification steps (automated, shown in UI):                   │
│                                                                  │
│  1. Browser fetches all static assets                            │
│  2. Computes SHA-256 of entire bundle:                           │
│     hash = H(index.html || app.js || crypto.wasm || ...)        │
│  3. Displays: "Build hash: a3f7c9..."                            │
│  4. Voter can compare against:                                   │
│     - Hash published by ЦИК                                     │
│     - Hash published by each party's mirror                      │
│     - Hash from building source code yourself                    │
│     - Hash reported by browser extensions (community tools)      │
│                                                                  │
│  Additionally: The app includes a Subresource Integrity          │
│  manifest — every JS/WASM file has an expected hash.             │
│                                                                  │
│  For mobile app: Reproducible builds + published APK/IPA hash   │
│  Same principle — build from source, compare hash                │
└────────────────────────────────────────────────────────────────┘
```

### 7.3 Server-Side Attestation
The voting server and bulletin board run in Confidential Computing enclaves (AMD SEV-SNP or Intel TDX) where possible, providing hardware attestation that the running binary matches the expected hash. This is a defense-in-depth measure, not a primary trust mechanism (the system is secure even without it, thanks to E2E verifiability).
