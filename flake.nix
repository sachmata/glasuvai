{
  description = "Glasuvai — Verifiable online voting system for Bulgarian elections";

  inputs = {
    # Pin to NixOS 24.11 LTS (stable, long-term support)
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";

    # Rust toolchain overlay — provides exact rustc versions
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        # Pin Rust toolchain to the exact version from rust-toolchain.toml
        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

        # Common native build inputs for all packages
        nativeBuildInputs = with pkgs; [
          rustToolchain
          pkg-config
        ];

        # System libraries needed at build/runtime
        buildInputs = with pkgs; [
          sqlite         # for rusqlite (bulletin-board, voting-server)
          openssl        # for potential TLS needs
        ];
      in
      {
        # ── Development shell ──────────────────────────────────────────
        devShells.default = pkgs.mkShell {
          inherit buildInputs;
          nativeBuildInputs = nativeBuildInputs ++ (with pkgs; [
            # Rust tooling
            cargo-audit    # Dependency vulnerability scanner
            cargo-deny     # License and advisory checker
            wasm-pack      # Build Rust → WASM packages

            # Node.js for web client (pinned LTS)
            nodejs_22

            # Nix tooling
            nil            # Nix LSP
            nixpkgs-fmt    # Nix formatter
          ]);

          shellHook = ''
            echo "glasuvai dev shell"
            echo "  rustc: $(rustc --version)"
            echo "  cargo: $(cargo --version)"
            echo "  node:  $(node --version)"
            echo "  wasm-pack: $(wasm-pack --version)"
          '';

          # Ensure rusqlite can find SQLite
          SQLITE3_LIB_DIR = "${pkgs.sqlite.out}/lib";
        };

        # ── Packages ──────────────────────────────────────────────────
        # Each package builds a specific binary from the workspace.
        # All share the same pinned Rust toolchain and dependencies.
        #
        # Usage:
        #   nix build .#glasuvai-admin
        #   nix build .#glasuvai-verifier
        #   nix build .#bulletin-board
        #   nix build .#voting-server
        #
        # Reproducibility: anyone building from the same commit with
        # the same flake.lock gets bit-for-bit identical outputs.

        packages = {
          # TODO: Uncomment as packages are implemented in each milestone
          #
          # glasuvai-admin = pkgs.rustPlatform.buildRustPackage {
          #   pname = "glasuvai-admin";
          #   version = "0.1.0";
          #   src = ./.;
          #   cargoLock.lockFile = ./Cargo.lock;
          #   cargoBuildFlags = [ "-p" "glasuvai-admin" ];
          #   inherit buildInputs nativeBuildInputs;
          # };
          #
          # glasuvai-verifier = pkgs.rustPlatform.buildRustPackage {
          #   pname = "glasuvai-verifier";
          #   version = "0.1.0";
          #   src = ./.;
          #   cargoLock.lockFile = ./Cargo.lock;
          #   cargoBuildFlags = [ "-p" "glasuvai-verifier" ];
          #   inherit buildInputs nativeBuildInputs;
          # };
        };

        # ── Checks (CI) ──────────────────────────────────────────────
        checks = {
          # TODO: Add clippy, fmt, test checks
          # clippy = ...
          # fmt = ...
          # test = ...
        };
      }
    );
}
