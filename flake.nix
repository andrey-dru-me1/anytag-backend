# SPDX-FileCopyrightText: 2026 The Anytag Backend Authors
# SPDX-License-Identifier: AGPL-3.0-only

{
  description = "anytag-backend development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
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
            "llvm-tools-preview"
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
            nixfmt
            openssl
            pkg-config
            nixd
            reuse
            cargo-watch
            cargo-llvm-cov  # test coverage
          ] ++ lib.optionals stdenv.isDarwin [
            apple-sdk
          ];

          env = {
            # DATABASE_URL will be constructed from .env variables in shellHook
            # or will be set by .env file loaded via direnv
            RUST_BACKTRACE = "1";
            CARGO_TERM_COLOR = "always";
          };

          LD_LIBRARY_PATH = "${pkgs.openssl.out}/lib";

          shellHook = ''
            ${pkgs.lib.optionalString pkgs.stdenv.isDarwin ''
              export LLDB_DEBUGSERVER_PATH="/Library/Developer/CommandLineTools/Library/PrivateFrameworks/LLDB.framework/Versions/A/Resources/debugserver"
              export PATH="/Library/Developer/CommandLineTools/usr/bin:/usr/bin:$PATH"
            ''}

            # Construct DATABASE_URL from individual components if not already set
            if [ -z "$DATABASE_URL" ] && [ -n "$DB_TYPE" ] && [ -n "$DB_HOST" ] && [ -n "$DB_PORT" ] && [ -n "$DB_USER" ] && [ -n "$DB_PASS" ] && [ -n "$DB_NAME" ]; then
              export DATABASE_URL="$DB_TYPE://$DB_USER:$DB_PASS@$DB_HOST:$DB_PORT/$DB_NAME"
              echo "🔗 Constructed DATABASE_URL from environment variables"
            elif [ -z "$DATABASE_URL" ]; then
              echo "⚠️  DATABASE_URL is not set and required components are missing"
              echo "   Set DATABASE_URL or DB_TYPE, DB_HOST, DB_PORT, DB_USER, DB_PASS, DB_NAME in .env"
            fi
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
