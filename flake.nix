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

        buildInputs =
          []
          ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [];

        SQLX_OFFLINE = true;
      };

      cargoArtifacts = craneLib.buildDepsOnly (commonArgs
        // {
          cargoToml = ./Cargo.toml; # Explicitly point to the root Cargo.toml for workspace deps
          pname = "lana-workspace-deps"; # A distinct name for the deps build
          version = "0.0.0"; # A placeholder version for the deps build
          CARGO_PROFILE = "dev";
        });

      # Build dependencies in release mode
      cargoArtifacts-release = craneLib.buildDepsOnly (commonArgs-release
        // {
          cargoToml = ./Cargo.toml; # Explicitly point to the root Cargo.toml
          pname = "lana-workspace-deps-release"; # Distinct name for release deps
          version = "0.0.0"; # Placeholder version
          CARGO_PROFILE = "release";
        });

      lanaCliPname = "lana-cli";

      # Build the Lana CLI crate using the cached deps
      lana-cli = craneLib.buildPackage (commonArgs
        // {
          pname = "${lanaCliPname}-debug"; # Set pname for debug build
          CARGO_PROFILE = "dev"; # Explicitly set dev profile
          cargoToml = ./lana/cli/Cargo.toml; # Explicitly point to the CLI's Cargo.toml
          cargoArtifacts = cargoArtifacts;
          doCheck = false; # Disable tests for lana-cli
          # pname and version will now be taken from ./lana/cli/Cargo.toml by crane
          # pname = lanaCliPname; # Or keep explicitly if preferred
          # version = lanaCliVersion; # Or keep explicitly if preferred

          # FIXME: aiming at parity with older script for now
          cargoExtraArgs = "-p ${lanaCliPname} --features sim-time"; # Build only the specific package
        });

      # Build the Lana CLI crate in release mode
      lana-cli-release = craneLib.buildPackage (commonArgs
        // {
          pname = "${lanaCliPname}-release"; # Set pname for release build
          CARGO_PROFILE = "release"; # Explicitly set release profile
          cargoToml = ./lana/cli/Cargo.toml; # Explicitly point to the CLI's Cargo.toml
          cargoArtifacts = cargoArtifacts-release; # Use release deps
          doCheck = false; # Disable tests
          # pname and version will be taken from ./lana/cli/Cargo.toml
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
        packages.lana-cli-release = lana-cli-release;
        packages.deps = cargoArtifacts;
        packages.deps-release = cargoArtifacts-release; # Expose release deps

        apps.default = flake-utils.lib.mkApp {drv = lana-cli;};

        devShells.default =
          mkShell (devEnvVars // {inherit nativeBuildInputs;});

        formatter = alejandra;
      });
}
