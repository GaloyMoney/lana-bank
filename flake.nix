{
  description = "Lana";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = { nixpkgs.follows = "nixpkgs"; };
    };
    crane.url = "github:ipetkov/crane";
  };
  outputs = { self, nixpkgs, flake-utils, rust-overlay, crane, }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [
          (self: super: { nodejs = super.nodejs_20; })
          (import rust-overlay)
        ];
        pkgs = import nixpkgs { inherit system overlays; };
        rustVersion = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile
          ./rust-toolchain.toml;
        rustToolchain =
          rustVersion.override { extensions = [ "rust-analyzer" "rust-src" ]; };
        mkAlias = alias: command: pkgs.writeShellScriptBin alias command;

        craneLib = crane.mkLib pkgs;

        cleaned = pkgs.lib.cleanSourceWith {
          src = ./.;
          filter = path: type:
            craneLib.filterCargoSources path type
            || pkgs.lib.hasInfix "/lib/authz/src/rbac.conf" path
            || pkgs.lib.hasInfix "/.sqlx/" path;
        };

        commonArgs = {
          src = cleaned;
          strictDeps = true;

          buildInputs = [
            # Add additional build inputs here
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            # Additional darwin specific inputs can be set here
            pkgs.libiconv
          ];

          SQLX_OFFLINE = true;
          name = "lana123";
          version = "0.2.0";
        };

        # Build only the Cargo dependencies (for caching)
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        # my-crate = craneLib.buildPackage (commonArgs // {
        #   cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        #   # Additional environment variables or build phases/hooks can be set
        #   # here *without* rebuilding all dependency crates
        #   # MY_CUSTOM_VAR = "some value";
        # });

        # Build the Lana CLI crate using the cached deps
        my-crate = craneLib.buildPackage (commonArgs // {
          cargoArtifacts = cargoArtifacts;
          pname = "lana-cli"; # Package name for the derivation
          cargoExtraArgs =
            "-p lana-cli"; # Build only the `lana/cli` workspace member&#8203;:contentReference[oaicite:9]{index=9}
          # (Add `nativeBuildInputs` or `buildInputs` here too if the crate itself needs them)
        });

        aliases =
          [ (mkAlias "meltano" ''docker compose run --rm meltano -- "$@"'') ];
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
            pnpm
            vendir
            netlify-cli
            tilt
            pandoc
            skopeo
          ]
          ++ lib.optionals pkgs.stdenv.isLinux [ xvfb-run cypress wkhtmltopdf ]
          ++ lib.optionals pkgs.stdenv.isDarwin
          [ darwin.apple_sdk.frameworks.SystemConfiguration ] ++ aliases;
        devEnvVars = rec {
          OTEL_EXPORTER_OTLP_ENDPOINT = "http://localhost:4317";
          PGDATABASE = "pg";
          PGUSER = "user";
          PGPASSWORD = "password";
          PGHOST = "127.0.0.1";
          DATABASE_URL = "postgres://${PGUSER}:${PGPASSWORD}@${PGHOST}:5433/pg";
          PG_CON = "${DATABASE_URL}";
        };
      in with pkgs; {
        packages.default = my-crate;
        packages.deps = cargoArtifacts;

        apps.default = flake-utils.lib.mkApp { drv = my-crate; };

        devShells.default =
          mkShell (devEnvVars // { inherit nativeBuildInputs; });

        formatter = alejandra;
      });
}
