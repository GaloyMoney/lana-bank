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

      craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
      #craneLib = crane.mkLib pkgs;
      # craneLib = craneLib.crateNameFromCargoToml {cargoToml = "./path/to/Cargo.toml";};

      rustSource = pkgs.lib.cleanSourceWith {
        src = ./.;
        filter = path: type:
          craneLib.filterCargoSources path type
          || pkgs.lib.hasInfix "/lib/authz/src/rbac.conf" path
          || pkgs.lib.hasInfix "/.sqlx/" path
          || pkgs.lib.hasInfix "/lana/app/migrations/" path
          || pkgs.lib.hasInfix "/lana/notification/src/email/templates/" path;
      };

      # Source filter for entity-rollups that includes templates and workspace dependencies
      entityRollupsSource = pkgs.lib.cleanSourceWith {
        src = ./.;
        filter = path: type:
          craneLib.filterCargoSources path type
          || pkgs.lib.hasInfix "/lib/authz/src/rbac.conf" path
          || pkgs.lib.hasInfix "/.sqlx/" path
          || pkgs.lib.hasInfix "/lana/entity-rollups/src/templates/" path;
      };

      # Function to build cargo artifacts for a specific profile
      mkCargoArtifacts = profile:
        craneLib.buildDepsOnly {
          src = rustSource;
          strictDeps = true;
          cargoToml = ./Cargo.toml;
          pname = "lana-workspace-deps-${profile}";
          version = "0.0.0";
          CARGO_PROFILE = profile;
          SQLX_OFFLINE = true;
          cargoExtraArgs = "--features sim-time";
        };

      # Function to build lana-cli for a specific profile
      mkLanaCli = profile: let
        cargoArtifacts = mkCargoArtifacts profile;
      in
        craneLib.buildPackage {
          src = rustSource;
          strictDeps = true;
          cargoToml = ./lana/cli/Cargo.toml;
          inherit cargoArtifacts;
          doCheck = false;
          pname = "lana-cli";
          CARGO_PROFILE = profile;
          SQLX_OFFLINE = true;
          cargoExtraArgs = "-p lana-cli --features sim-time";
        };

      # Function to build static lana-cli (musl target for containers)
      mkLanaCliStatic = profile: let
        rustTarget = "x86_64-unknown-linux-musl";
        # Build dependencies specifically for the musl target
        cargoArtifactsStatic = craneLibMusl.buildDepsOnly {
          src = rustSource;
          strictDeps = true;
          cargoToml = ./Cargo.toml;
          pname = "lana-workspace-deps-${profile}-musl";
          version = "0.0.0";
          CARGO_PROFILE = profile;
          SQLX_OFFLINE = true;
          CARGO_BUILD_TARGET = rustTarget;
          cargoExtraArgs = "--features sim-time,sim-bootstrap --target ${rustTarget}";

          # Add musl target dependencies
          depsBuildBuild = with pkgs; [
            pkgsCross.musl64.stdenv.cc
          ];

          # Environment variables for static linking
          CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER = "${pkgs.pkgsCross.musl64.stdenv.cc}/bin/x86_64-unknown-linux-musl-gcc";
          CC_x86_64_unknown_linux_musl = "${pkgs.pkgsCross.musl64.stdenv.cc}/bin/x86_64-unknown-linux-musl-gcc";
          TARGET_CC = "${pkgs.pkgsCross.musl64.stdenv.cc}/bin/x86_64-unknown-linux-musl-gcc";
        };
      in
        craneLibMusl.buildPackage {
          src = rustSource;
          strictDeps = true;
          cargoToml = ./lana/cli/Cargo.toml;
          cargoArtifacts = cargoArtifactsStatic;
          doCheck = false;
          pname = "lana-cli-static";
          CARGO_PROFILE = profile;
          SQLX_OFFLINE = true;
          CARGO_BUILD_TARGET = rustTarget;
          cargoExtraArgs = "-p lana-cli --features sim-time,sim-bootstrap --target ${rustTarget}";

          # Add musl target for static linking
          depsBuildBuild = with pkgs; [
            pkgsCross.musl64.stdenv.cc
          ];

          # Environment variables for static linking
          CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER = "${pkgs.pkgsCross.musl64.stdenv.cc}/bin/x86_64-unknown-linux-musl-gcc";
          CC_x86_64_unknown_linux_musl = "${pkgs.pkgsCross.musl64.stdenv.cc}/bin/x86_64-unknown-linux-musl-gcc";
          TARGET_CC = "${pkgs.pkgsCross.musl64.stdenv.cc}/bin/x86_64-unknown-linux-musl-gcc";
        };

      # Build artifacts and packages for both profiles
      debugCargoArtifacts = mkCargoArtifacts "dev";
      releaseCargoArtifacts = mkCargoArtifacts "release";

      lana-cli-debug = mkLanaCli "dev";
      lana-cli-release = mkLanaCli "release";
      lana-cli-static = mkLanaCliStatic "release";

      checkCode = craneLib.mkCargoDerivation {
        pname = "check-code";
        version = "0.1.0";
        src = rustSource;
        cargoToml = ./Cargo.toml;
        cargoLock = ./Cargo.lock;
        cargoArtifacts = debugCargoArtifacts;
        SQLX_OFFLINE = true;
        cargoExtraArgs = "--all-targets --all-features";

        nativeBuildInputs = [
          pkgs.protobuf
          pkgs.cacert
          pkgs.cargo-audit
          pkgs.cargo-deny
          pkgs.cargo-machete
        ];

        configurePhase = ''
          export CARGO_NET_GIT_FETCH_WITH_CLI=true
          export PROTOC="${pkgs.protobuf}/bin/protoc"
          export PATH="${pkgs.protobuf}/bin:${pkgs.gitMinimal}/bin:${pkgs.coreutils}/bin:$PATH"
          export SSL_CERT_FILE="${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
          export CARGO_HTTP_CAINFO="$SSL_CERT_FILE"
        '';

        buildPhaseCargoCommand = "check";
        buildPhase = ''
          cargo fmt --check
          cargo clippy --all-targets --all-features || true
          cargo audit
          cargo deny check
          cargo machete
        '';
        installPhase = "touch $out";
      };

      testInCi = craneLib.mkCargoDerivation {
        pname = "test-in-ci";
        version = "0.1.0";
        src = rustSource;
        cargoToml = ./Cargo.toml;
        cargoLock = ./Cargo.lock;
        cargoArtifacts = debugCargoArtifacts;
        SQLX_OFFLINE = true;

        nativeBuildInputs = [
          pkgs.cacert
          pkgs.cargo-nextest
          pkgs.protobuf
          pkgs.gitMinimal
        ];

        configurePhase = ''
          export CARGO_NET_GIT_FETCH_WITH_CLI=true
          export PROTOC="${pkgs.protobuf}/bin/protoc"
          export PATH="${pkgs.protobuf}/bin:$PATH"
          export SSL_CERT_FILE="${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
          export CARGO_HTTP_CAINFO="$SSL_CERT_FILE"
        '';

        buildPhaseCargoCommand = "nextest run";
        buildPhase = ''
          # run whole workspace tests, verbose+locked to mirror Makefile
          cargo nextest run --workspace --locked --verbose
        '';

        installPhase = "touch $out";
      };

      entity-rollups = craneLib.buildPackage {
        src = entityRollupsSource;
        cargoToml = ./Cargo.toml;
        cargoArtifacts = debugCargoArtifacts;
        pname = "entity-rollups";
        version = "0.1.0";
        doCheck = false;
        SQLX_OFFLINE = true;
        cargoExtraArgs = "-p entity-rollups --all-features";
      };

      write_sdl = craneLib.buildPackage {
        src = rustSource;
        cargoToml = ./Cargo.toml;
        cargoArtifacts = debugCargoArtifacts;
        pname = "write_sdl";
        version = "0.1.0";
        doCheck = false;
        SQLX_OFFLINE = true;
        cargoExtraArgs = "--bin write_sdl";
      };

      write_customer_sdl = craneLib.buildPackage {
        src = rustSource;
        cargoToml = ./Cargo.toml;
        cargoArtifacts = debugCargoArtifacts;
        pname = "write_customer_sdl";
        version = "0.1.0";
        doCheck = false;
        SQLX_OFFLINE = true;
        cargoExtraArgs = "--bin write_customer_sdl";
      };

      meltano = pkgs.callPackage ./meltano.nix {};

      mkAlias = alias: command: pkgs.writeShellScriptBin alias command;

      rustVersion = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      rustToolchain = rustVersion.override {
        extensions = [
          "rust-analyzer"
          "rust-src"
          "rustfmt"
          "clippy"
        ];
        targets = ["x86_64-unknown-linux-musl"];
      };

      # Separate toolchain for musl cross-compilation
      rustToolchainMusl = rustVersion.override {
        extensions = ["rust-src"];
        targets = ["x86_64-unknown-linux-musl"];
      };

      # Create a separate Crane lib for musl builds
      craneLibMusl = (crane.mkLib pkgs).overrideToolchain rustToolchainMusl;

      nativeBuildInputs = with pkgs;
        [
          wait4x
          rustToolchain
          opentofu
          alejandra
          ytt
          sqlx-cli
          cargo-nextest
          cargo-audit
          cargo-watch
          cargo-deny
          cargo-machete
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
          procps
          meltano
        ]
        ++ lib.optionals pkgs.stdenv.isLinux [
          xvfb-run
          cypress
          python313Packages.weasyprint

          slirp4netns
          fuse-overlayfs

          util-linux
          psmisc
        ]
        ++ lib.optionals pkgs.stdenv.isDarwin [
          darwin.apple_sdk.frameworks.SystemConfiguration
        ];
      devEnvVars = rec {
        OTEL_EXPORTER_OTLP_ENDPOINT = http://localhost:4317;
        PGDATABASE = "pg";
        PGUSER = "user";
        PGPASSWORD = "password";
        PGHOST = "127.0.0.1";
        DATABASE_URL = "postgres://${PGUSER}:${PGPASSWORD}@${PGHOST}:5433/pg?sslmode=disable";
        PG_CON = "${DATABASE_URL}";
        CUSTODIAN_ENCRYPTION_KEY = "0000000000000000000000000000000000000000000000000000000000000000";
      };
    in
      with pkgs; {
        packages = {
          default = lana-cli-debug; # Debug as default
          debug = lana-cli-debug;
          release = lana-cli-release;
          static = lana-cli-static;
          check-code = checkCode;
          test-in-ci = testInCi;
          entity-rollups = entity-rollups;
          write_sdl = write_sdl;
          write_customer_sdl = write_customer_sdl;
          inherit meltano;
        };

        apps.default = flake-utils.lib.mkApp {drv = lana-cli-debug;};

        devShells.default = mkShell (devEnvVars
          // {
            inherit nativeBuildInputs;
            shellHook = ''
              export LANA_CONFIG="$(pwd)/bats/lana.yml"
              export MELTANO_PROJECT_ROOT="$(pwd)/meltano"
            '';
          });

        formatter = alejandra;
      });
}
