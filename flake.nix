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
    crane.url = "github:ipetkov/crane";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
    crane,
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

      craneLib = crane.mkLib pkgs;
      # craneLib = craneLib.crateNameFromCargoToml {cargoToml = "./path/to/Cargo.toml";};

      rustSource = pkgs.lib.cleanSourceWith {
        src = ./.;
        filter = path: type:
          craneLib.filterCargoSources path type
          || pkgs.lib.hasInfix "/lib/authz/src/rbac.conf" path
          || pkgs.lib.hasInfix "/.sqlx/" path
          || pkgs.lib.hasInfix "/lana/app/migrations/" path;
      };

      commonArgs = {
        src = rustSource;
        strictDeps = true;

        CARGO_PROFILE = "dev";
        SQLX_OFFLINE = true;
        # No specific package name for commonArgs, it's for general settings
        # version = lanaCliVersion; # Version will be set per-package
      };

      cargoArtifacts = craneLib.buildDepsOnly (commonArgs
        // {
          cargoToml = ./Cargo.toml; # Explicitly point to the root Cargo.toml for workspace deps
          pname = "lana-workspace-deps"; # A distinct name for the deps build
          version = "0.0.0"; # A placeholder version for the deps build
          CARGO_PROFILE = "dev"; # Explicitly set dev profile
          cargoExtraArgs = "--features sim-time"; # Build only the specific package
        });

      lanaCliPname = "lana-cli";

      # Build the Lana CLI crate using the cached deps
      lana-cli = craneLib.buildPackage (commonArgs
        // {
          cargoToml = ./lana/cli/Cargo.toml; # Explicitly point to the CLI's Cargo.toml
          cargoArtifacts = cargoArtifacts;
          doCheck = false; # Disable tests for lana-cli
          pname = lanaCliPname; # Use the original package name
          CARGO_PROFILE = "dev"; # Explicitly set dev profile

          # FIXME: aiming at parity with older script for now
          cargoExtraArgs = "-p ${lanaCliPname} --features sim-time"; # Build only the specific package
        });

      mkAlias = alias: command: pkgs.writeShellScriptBin alias command;

      rustVersion = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      rustToolchain = rustVersion.override {
        extensions = ["rust-analyzer" "rust-src"];
      };

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
          nodejs
          typescript
          google-cloud-sdk
          pnpm
          vendir
          netlify-cli
          pandoc
          nano
          podman
          podman-compose
          cachix
          ps
          curl
          tilt
        ]
        ++ lib.optionals pkgs.stdenv.isLinux [
          xvfb-run
          cypress
          wkhtmltopdf

          slirp4netns
          fuse-overlayfs

          util-linux
          psmisc
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
        packages.default = lana-cli;
        packages.deps = cargoArtifacts;

        apps.default = flake-utils.lib.mkApp {drv = lana-cli;};

        devShells.default =
          mkShell (devEnvVars // {inherit nativeBuildInputs;});

        formatter = alejandra;
      });
}
