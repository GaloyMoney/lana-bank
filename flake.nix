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
        # Disable tests on libsecret due to missing DBUS on gh
        (self: super: {
          libsecret = super.libsecret.overrideAttrs (oldAttrs: {
            doCheck = false;
            doInstallCheck = false;
          });
        })
        (self: super: {
          python311 = super.python311.override {
            packageOverrides = pySelf: pySuper: let
              lib = super.lib;

              disableTests = pkg:
                pkg.overrideAttrs (_: {
                  doCheck = false;
                  doInstallCheck = false;
                });
            in
              lib.mapAttrs (
                name: pkg:
                  if lib.isDerivation pkg && builtins.hasAttr "overrideAttrs" pkg
                  then disableTests pkg
                  else pkg
              )
              pySuper;
          };
        })
      ];
      pkgs = import nixpkgs {
        inherit system overlays;
      };

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
          || pkgs.lib.hasInfix "/lana/notification/src/email/templates/" path
          || pkgs.lib.hasInfix "/lana/contract-creation/src/templates/" path
          || pkgs.lib.hasInfix "/lib/rendering/config/" path;
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
          cargo-hakari
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
          poppler_utils
          keycloak
          # Documentation tools
          mdbook
          mdbook-mermaid
          # Font packages for PDF generation
          fontconfig
          dejavu_fonts # Provides serif, sans-serif, and monospace
        ]
        ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
          xvfb-run
          cypress
          python313Packages.weasyprint

          slirp4netns
          fuse-overlayfs

          util-linux
          psmisc
        ]
        ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [];
      devEnvVars = rec {
        OTEL_EXPORTER_OTLP_ENDPOINT = http://localhost:4317;
        DATABASE_URL = "postgres://user:password@127.0.0.1:5433/pg?sslmode=disable";
        PG_CON = "${DATABASE_URL}";
        ENCRYPTION_KEY = "0000000000000000000000000000000000000000000000000000000000000000";
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
          write_sdl = write_sdl;
          write_customer_sdl = write_customer_sdl;
        };

        apps.default = flake-utils.lib.mkApp {drv = lana-cli-debug;};

        devShells.default = mkShell (devEnvVars
          // {
            inherit nativeBuildInputs;
            shellHook = ''
              export LANA_CONFIG="$(pwd)/bats/lana.yml"
              export MELTANO_PROJECT_ROOT="$(pwd)/meltano"

              # Font configuration for PDF generation
              export FONTCONFIG_FILE=${pkgs.fontconfig.out}/etc/fonts/fonts.conf
              export FONTCONFIG_PATH=${pkgs.fontconfig.out}/etc/fonts

              export KC_URL="http://localhost:8081"
              export REALM="master"
              export ADMIN_USER="admin"
              export ADMIN_PASS="admin"

              # Container engine setup
              # Clear DOCKER_HOST at the start to avoid stale values
              unset DOCKER_HOST

              # Use ENGINE_DEFAULT if already set, otherwise auto-detect
              if [[ -n "''${ENGINE_DEFAULT:-}" ]]; then
                echo "Using pre-configured engine: $ENGINE_DEFAULT"
              elif command -v podman &>/dev/null && ! command -v docker &>/dev/null; then
                export ENGINE_DEFAULT=podman
              else
                export ENGINE_DEFAULT=docker
              fi

              # Set up podman socket if using podman
              if [[ "$ENGINE_DEFAULT" == "podman" ]]; then
                # Let existing scripts handle podman setup
                if [[ "''${CI:-false}" == "true" ]] && [[ -f "$(pwd)/dev/bin/podman-service-start.sh" ]]; then
                  "$(pwd)/dev/bin/podman-service-start.sh" >/dev/null 2>&1 || true
                fi

                # Set socket if available (for both CI and local)
                socket="$($(pwd)/dev/bin/podman-get-socket.sh 2>/dev/null || echo NO_SOCKET)"
                [[ "$socket" != "NO_SOCKET" ]] && export DOCKER_HOST="$socket"
              fi
            '';
          });

        formatter = alejandra;
      });
}
