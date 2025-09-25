{
  description = "Lana";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
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
    advisory-db,
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

      commonArgs = {
        src = rustSource;
        strictDeps = true;
        SQLX_OFFLINE = true;
      };

      cargoArtifacts = craneLib.buildDepsOnly (commonArgs
        // {
          cargoExtraArgs = "--features sim-time,mock-custodian,sumsub-testing";
        });

      individualCrateArgs =
        commonArgs
        // {
          inherit cargoArtifacts;
          inherit (craneLib.crateNameFromCargoToml {src = rustSource;}) version;
          # NB: we disable tests since we'll run them all via cargo-nextest
          doCheck = false;
        };

      lana-cli-debug = craneLib.buildPackage (
        individualCrateArgs
        // {
          pname = "lana-cli-debug";
          cargoExtraArgs = "-p lana-cli --features sim-time,mock-custodian,sumsub-testing";
          src = rustSource;
        }
      );

      # Pre-built test binaries
      lana-test-binaries = craneLib.buildPackage (
        individualCrateArgs
        // {
          pname = "lana-test-binaries";
          cargoExtraArgs = "--tests --all-features";
          doCheck = false;
        }
      );

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
          iptables
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
          default = lana-cli-debug;

          lana-cli-debug = lana-cli-debug;

          lana-deps = cargoArtifacts;

          lana-test-binaries = lana-test-binaries;

          podman-up = let
            podman-runner = pkgs.callPackage ./nix/podman-runner.nix {};
          in
            pkgs.writeShellScriptBin "podman-up" ''
              exec ${podman-runner.podman-compose-runner}/bin/podman-compose-runner up "$@"
            '';

          bats-runner = let
            podman-runner = pkgs.callPackage ./nix/podman-runner.nix {};
          in pkgs.symlinkJoin {
            name = "bats-runner";
            paths = [
              podman-runner.podman-compose-runner
              pkgs.wait4x
              pkgs.bats
              pkgs.gnugrep
              pkgs.procps
              pkgs.coreutils
              pkgs.findutils
              pkgs.jq
              pkgs.curl
              pkgs.gnused
              pkgs.gawk
              lana-cli-debug
            ];
            postBuild = ''
              mkdir -p $out/bin
              cat > $out/bin/bats-runner << 'EOF'
              #!${pkgs.bash}/bin/bash
              set -e

              # Set environment variables needed by bats tests
              export LANA_BIN="${lana-cli-debug}/bin/lana-cli"
              export PG_CON="${devEnvVars.PG_CON}"
              export DATABASE_URL="${devEnvVars.DATABASE_URL}"
              export ENCRYPTION_KEY="${devEnvVars.ENCRYPTION_KEY}"

              # Function to cleanup on exit
              cleanup() {
                echo "Stopping podman-compose..."
                podman-compose-runner down || true
              }

              # Register cleanup function
              trap cleanup EXIT

              echo "Starting podman-compose in detached mode..."
              podman-compose-runner up -d

              # Wait for PostgreSQL to be ready
              echo "Waiting for PostgreSQL to be ready..."
              wait4x postgresql "${devEnvVars.PG_CON}" --timeout 120s

              echo "Running bats tests with LANA_BIN=$LANA_BIN..."
              bats bats/*.bats

              echo "Tests completed successfully!"
              EOF
              chmod +x $out/bin/bats-runner
            '';
          };

          # Legacy wrapper for backward compatibility
          bats = pkgs.writeShellScriptBin "bats" ''
            exec ${self.packages.${system}.bats-runner}/bin/bats-runner "$@"
          '';

          nextest-runner = let
            podman-runner = pkgs.callPackage ./nix/podman-runner.nix {};
          in pkgs.symlinkJoin {
            name = "nextest-runner";
            paths = [
              podman-runner.podman-compose-runner
              pkgs.wait4x
              pkgs.sqlx-cli
              pkgs.cargo-nextest
              pkgs.coreutils
              lana-test-binaries
            ];
            postBuild = ''
              mkdir -p $out/bin
              cat > $out/bin/nextest-runner << 'EOF'
              #!${pkgs.bash}/bin/bash
              set -e

              # Set environment variables needed by tests
              export DATABASE_URL="${devEnvVars.DATABASE_URL}"
              export PG_CON="${devEnvVars.PG_CON}"

              # Function to cleanup on exit
              cleanup() {
                echo "Stopping core-pg..."
                podman-compose-runner stop core-pg || true
                podman-compose-runner rm -f core-pg || true
              }

              # Register cleanup function
              trap cleanup EXIT

              echo "Starting core-pg database..."
              podman-compose-runner up -d core-pg

              # Wait for PostgreSQL to be ready
              echo "Waiting for PostgreSQL to be ready..."
              wait4x postgresql "$DATABASE_URL" --timeout 120s

              # Run migrations
              echo "Running database migrations..."
              sqlx migrate run --source lana/app/migrations

              # Run nextest
              echo "Running cargo nextest..."
              cargo-nextest nextest run --workspace --all-features

              echo "Tests completed successfully!"
              EOF
              chmod +x $out/bin/nextest-runner
            '';
          };

          # Legacy wrapper for backward compatibility
          nextest = pkgs.writeShellScriptBin "nextest" ''
            exec ${self.packages.${system}.nextest-runner}/bin/nextest-runner "$@"
          '';
        };

        checks = {
          workspace-clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );
          workspace-fmt = craneLib.cargoFmt {
            src = rustSource;
          };

          workspace-audit = craneLib.cargoAudit {
            inherit advisory-db;
            src = rustSource;
          };

          workspace-deny = craneLib.cargoDeny {
            src = rustSource;
          };

          workspace-hakari = craneLib.mkCargoDerivation {
            src = rustSource;
            pname = "workspace-hakari";
            cargoArtifacts = null;
            doInstallCargoArtifacts = false;

            buildPhaseCargoCommand = ''
              cargo hakari generate --diff
              cargo hakari manage-deps --dry-run
              cargo hakari verify
            '';

            nativeBuildInputs = [
              pkgs.cargo-hakari
            ];
          };

          workspace-machete = craneLib.mkCargoDerivation {
            src = rustSource;
            pname = "lana-bank-machete";
            cargoArtifacts = null;
            doInstallCargoArtifacts = false;

            buildPhaseCargoCommand = ''
              cargo machete
            '';

            nativeBuildInputs = [
              pkgs.cargo-machete
            ];
          };
        };

        apps.default = flake-utils.lib.mkApp {
          drv = lana-cli-debug;
          name = "lana-cli";
        };

        apps.podman-up = flake-utils.lib.mkApp {
          drv = self.packages.${system}.podman-up;
          name = "podman-up";
        };

        apps.bats = flake-utils.lib.mkApp {
          drv = self.packages.${system}.bats-runner;
          name = "bats-runner";
        };

        apps.nextest = flake-utils.lib.mkApp {
          drv = self.packages.${system}.nextest-runner;
          name = "nextest-runner";
        };

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
