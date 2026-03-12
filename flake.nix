{
  description = "anytag-backend development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
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
            docker-compose
            just
            nixfmt-rfc-style
            openssl
            pkg-config
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
            if [ -z "$DATABASE_URL" ] && [ -n "$POSTGRES_USER" ] && [ -n "$POSTGRES_PASSWORD" ] && [ -n "$POSTGRES_DB" ] && [ -n "$DB_PORT" ]; then
              export DATABASE_URL="postgres://$POSTGRES_USER:$POSTGRES_PASSWORD@localhost:$DB_PORT/$POSTGRES_DB"
              echo "🔗 Constructed DATABASE_URL from environment variables"
            elif [ -z "$DATABASE_URL" ]; then
              echo "⚠️  DATABASE_URL is not set and required components are missing"
              echo "   Set DATABASE_URL or POSTGRES_USER, POSTGRES_PASSWORD, POSTGRES_DB, DB_PORT in .env"
            fi

            echo "========================================"
            echo "🎯 anytag-backend Development Environment"
            echo "========================================"
            echo ""
            echo "📦 Tools:"
            echo "  Rust: $(rustc --version | cut -d' ' -f2)"
            echo "  Cargo: $(cargo --version | cut -d' ' -f2)"
            echo "  Diesel: $(diesel --version | cut -d' ' -f2)"
            echo ""
            echo "🚀 Quick start:"
            echo "  1. docker-compose up -d db"
            echo "  2. diesel migration run"
            echo "  3. cargo build"
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
