{
  description = "Lana";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
  };
  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
  }:
    flake-utils.lib.eachDefaultSystem
    (system: let
      overlays = [
        (self: super: {
          nodejs = super.nodejs_20;
        })
        (import rust-overlay)
      ];
      pkgs = import nixpkgs {
        inherit system overlays;
      };
      rustVersion = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      rustToolchain = rustVersion.override {
        extensions = ["rust-analyzer" "rust-src"];
      };
      mkAlias = alias: command: pkgs.writeShellScriptBin alias command;
      aliases = [
        (mkAlias "meltano" ''docker compose run --rm meltano -- "$@"'')
      ];
      nativeBuildInputs = with pkgs;
        [
          rustToolchain
          opentofu
          alejandra
          ytt
          sqlx-cli
          cargo-nextest
          cargo-audit
          cargo-watch
          bacon
          typos
          postgresql
          docker-compose
          bats
          jq
          napi-rs-cli
          yarn
          nodejs
          typescript
          google-cloud-sdk
          ytt
          pnpm
          tilt
          pandoc
          nodePackages.vercel
        ]
        ++ lib.optionals pkgs.stdenv.isLinux [
          xvfb-run
          cypress
          wkhtmltopdf
        ]
        ++ lib.optionals pkgs.stdenv.isDarwin [
          darwin.apple_sdk.frameworks.SystemConfiguration
        ]
        ++ aliases;
      devEnvVars = rec {
        OTEL_EXPORTER_OTLP_ENDPOINT = http://localhost:4317;
        PGDATABASE = "pg";
        PGUSER = "user";
        PGPASSWORD = "password";
        PGHOST = "127.0.0.1";
        DATABASE_URL = "postgres://${PGUSER}:${PGPASSWORD}@${PGHOST}:5433/pg";
        PG_CON = "${DATABASE_URL}";
      };
    in
      with pkgs; {
        devShells.default = mkShell (devEnvVars
          // {
            inherit nativeBuildInputs;
          });

        formatter = alejandra;
      });
}
