# SPDX-FileCopyrightText: 2026 The Anytag Backend Authors
# SPDX-License-Identifier: AGPL-3.0-only

{
  description = "anytag-backend development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
          ];
        };

      in
      {
        devShells.default = pkgs.mkShell {
          name = "anytag-backend-dev";

          buildInputs = with pkgs; [
            rustToolchain
            diesel-cli
            postgresql_18
            nil
            git
            just
            nixfmt-rfc-style
            openssl
            pkg-config
            nixd
            reuse
            cargo-watch
          ];

          env = {
            # DATABASE_URL will be constructed from .env variables in shellHook
            # or will be set by .env file loaded via direnv
            RUST_BACKTRACE = "1";
            CARGO_TERM_COLOR = "always";
          };

          LD_LIBRARY_PATH = "${pkgs.openssl.out}/lib";

          shellHook = ''
            # Construct DATABASE_URL from individual components if not already set
            if [ -z "$DATABASE_URL" ] && [ -n "$DB_TYPE" ] && [ -n "$DB_HOST" ] && [ -n "$DB_PORT" ] && [ -n "$DB_USER" ] && [ -n "$DB_PASS" ] && [ -n "$DB_NAME" ]; then
              export DATABASE_URL="$DB_TYPE://$DB_USER:$DB_PASS@$DB_HOST:$DB_PORT/$DB_NAME"
              echo "🔗 Constructed DATABASE_URL from environment variables"
            elif [ -z "$DATABASE_URL" ]; then
              echo "⚠️  DATABASE_URL is not set and required components are missing"
              echo "   Set DATABASE_URL or DB_TYPE, DB_HOST, DB_PORT, DB_USER, DB_PASS, DB_NAME in .env"
            fi

            echo "========================================"
            echo "🎯 anytag-backend Development Environment"
            echo "========================================"
            echo ""
            echo "📦 Tools:"
            echo "  Rust: $(rustc --version | cut -d' ' -f2)"
            echo "  Cargo: $(cargo --version | cut -d' ' -f2)"
            echo "  Diesel: $(diesel -V | cut -d' ' -f2)"
            echo ""
            echo "🚀 Quick start:"
            echo "  1. docker compose up -d postgres"
            echo "  2. diesel migration run"
            echo "  3. cargo watch -x run   (hot-reload dev server)"
            echo "========================================"
          '';
        };

        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "anytag-backend";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

          buildInputs = with pkgs; [
            openssl
            postgresql_18
          ];

          cargoBuildFlags = [ "--release" ];
        };
      }
    );
}
