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
      };
      craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

      rustSource = pkgs.lib.cleanSourceWith {
        src = ./.;
        filter = path: type:
          craneLib.filterCargoSources path type
          || pkgs.lib.hasInfix "/lib/authz/src/rbac.conf" path
          || pkgs.lib.hasInfix "/.sqlx/" path
          || pkgs.lib.hasInfix "/dev/entity-rollups/src/templates" path
          || pkgs.lib.hasInfix "/dev/entity-rollups/schemas" path
          || pkgs.lib.hasInfix "/lana/app/migrations/" path
          || pkgs.lib.hasInfix "/lana/notification/src/email/templates/" path
          || pkgs.lib.hasInfix "/lana/contract-creation/src/templates/" path
          || pkgs.lib.hasInfix "/lib/rendering/config/" path
          || pkgs.lib.hasInfix "/lana/admin-server/src/graphql/schema.graphql" path
          || pkgs.lib.hasInfix "/lana/customer-server/src/graphql/schema.graphql" path;
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

      lana-cli-bootstrap = craneLib.buildPackage (
        individualCrateArgs
        // {
          pname = "lana-cli-bootstrap";
          cargoExtraArgs = "-p lana-cli --all-features";
          src = rustSource;
        }
      );

      lana-cli-static = let
        rustTarget = "x86_64-unknown-linux-musl";
      in
        craneLibMusl.buildPackage {
          inherit (craneLib.crateNameFromCargoToml {src = rustSource;}) version;
          src = rustSource;
          strictDeps = true;
          cargoToml = ./lana/cli/Cargo.toml;
          doCheck = false;
          pname = "lana-cli-static";
          CARGO_PROFILE = "release";
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

      # Pre-built test binaries with nextest archive
      lana-test-archive = craneLib.mkCargoDerivation (
        commonArgs
        // {
          inherit cargoArtifacts;
          pname = "lana-test-archive";

          buildPhaseCargoCommand = ''
            # Build all test binaries
            cargo test --workspace --all-features --no-run

            # Create nextest archive
            cargo nextest archive \
              --workspace \
              --all-features \
              --archive-file test-archive.tar.zst
          '';

          installPhase = ''
            mkdir -p $out
            cp test-archive.tar.zst $out/

            # Also save the binaries list for reference
            cargo nextest list --workspace --all-features > $out/test-list.txt
          '';

          nativeBuildInputs = [pkgs.cargo-nextest];
        }
      );

      rustToolchainMusl = rustVersion.override {
        extensions = ["rust-src"];
        targets = ["x86_64-unknown-linux-musl"];
      };

      craneLibMusl = (crane.mkLib pkgs).overrideToolchain rustToolchainMusl;

      meltanoPkgs = pkgs.callPackage ./nix/meltano.nix {};

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
          meltanoPkgs.meltano
          poppler_utils
          cocogitto
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
          meltano = meltanoPkgs.meltano;
          meltano-image = meltanoPkgs.meltano-image;
          default = lana-cli-debug;
          lana-cli-debug = lana-cli-debug;
          lana-cli-static = lana-cli-static;

          lana-deps = cargoArtifacts;

          lana-test-archive = lana-test-archive;

          write_sdl = craneLib.buildPackage (
            individualCrateArgs
            // {
              pname = "write_sdl";
              cargoExtraArgs = "-p admin-server --bin write_sdl";
            }
          );

          write_customer_sdl = craneLib.buildPackage (
            individualCrateArgs
            // {
              pname = "write_customer_sdl";
              cargoExtraArgs = "-p customer-server --bin write_customer_sdl";
            }
          );

          entity-rollups = craneLib.buildPackage (
            individualCrateArgs
            // {
              pname = "entity-rollups";
              cargoExtraArgs = "-p entity-rollups --all-features";
            }
          );

          podman-up = let
            podman-runner = pkgs.callPackage ./nix/podman-runner.nix {};
          in
            pkgs.writeShellScriptBin "podman-up" ''
              exec ${podman-runner.podman-compose-runner}/bin/podman-compose-runner up "$@"
            '';

          bats-runner = let
            podman-runner = pkgs.callPackage ./nix/podman-runner.nix {};
            binPath = pkgs.lib.makeBinPath [
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
              pkgs.poppler_utils
              pkgs.libuuid
            ];
          in
            pkgs.symlinkJoin {
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
                pkgs.poppler_utils
                pkgs.libuuid
                lana-cli-debug
              ];
              postBuild = ''
                mkdir -p $out/bin
                cat > $out/bin/bats-runner << 'EOF'
                #!${pkgs.bash}/bin/bash
                set -e

                # Add all tools to PATH
                export PATH="${binPath}:$PATH"

                # Set environment variables needed by bats tests
                export LANA_BIN="${lana-cli-debug}/bin/lana-cli"
                export PG_CON="${devEnvVars.PG_CON}"
                export DATABASE_URL="${devEnvVars.DATABASE_URL}"
                export ENCRYPTION_KEY="${devEnvVars.ENCRYPTION_KEY}"

                # Function to cleanup on exit
                cleanup() {
                  if [[ -n "''${KEEP_PODMAN_UP:-}" ]]; then
                    echo "KEEP_PODMAN_UP set — skipping podman-compose cleanup."
                    return 0
                  fi
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
                echo "Waiting for Keycloak..."
                ${pkgs.wait4x}/bin/wait4x http http://localhost:8081 --timeout 120s

                # Set TERM for CI environments
                export TERM="''${TERM:-dumb}"
                echo "Running bats tests with LANA_BIN=$LANA_BIN..."
                bats bats/*.bats

                echo "Tests completed successfully!"
                EOF
                chmod +x $out/bin/bats-runner
              '';
            };

          simulation-runner = let
            podman-runner = pkgs.callPackage ./nix/podman-runner.nix {};
            binPath = pkgs.lib.makeBinPath [
              podman-runner.podman-compose-runner
              pkgs.wait4x
              pkgs.gnugrep
            ];
          in
            pkgs.symlinkJoin {
              name = "bats-runner";
              paths = [
                podman-runner.podman-compose-runner
                pkgs.wait4x
                pkgs.gnugrep
                lana-cli-bootstrap
              ];
              postBuild = ''
                mkdir -p $out/bin
                cat > $out/bin/simulation-runner << 'EOF'
                #!${pkgs.bash}/bin/bash
                set -e

                # Add all tools to PATH
                export PATH="${binPath}:$PATH"

                # Set environment variables needed by bats tests
                export PG_CON="${devEnvVars.PG_CON}"
                export DATABASE_URL="${devEnvVars.DATABASE_URL}"
                export ENCRYPTION_KEY="${devEnvVars.ENCRYPTION_KEY}"

                # Function to cleanup on exit
                cleanup() {
                  echo "Stopping podman-compose..."
                  podman-compose-runner down || true
                  cat .server.pid | xargs kill || true
                }

                # Register cleanup function
                trap cleanup EXIT

                echo "Starting podman-compose in detached mode..."
                podman-compose-runner up -d

                # Wait for PostgreSQL to be ready
                echo "Waiting for PostgreSQL to be ready..."
                wait4x postgresql "${devEnvVars.PG_CON}" --timeout 120s

                echo "Running cli"
                export LANA_CONFIG="./bats/lana.yml"
                ${lana-cli-bootstrap}/bin/lana-cli 2>&1 | tee server.log &
                echo "$!" > .server.pid

                wait4x http http://localhost:5253/health --timeout 30m

                if grep -q "panicked" server.log; then
                  echo "❌ Server panicked; dumping last 200 lines of logs:"
                  tail -n 200 server.log
                  cat .server.pid | xargs kill || true
                  exit 1
                fi
                EOF
                chmod +x $out/bin/simulation-runner
              '';
            };

          # Legacy wrapper for backward compatibility
          bats = pkgs.writeShellScriptBin "bats" ''
            exec ${self.packages.${system}.bats-runner}/bin/bats-runner "$@"
          '';

          simulation = pkgs.writeShellScriptBin "simulation" ''
            exec ${self.packages.${system}.simulation-runner}/bin/simulation-runner "$@"
          '';

          # Simple nextest runner that runs pre-built test archive
          nextest-runner = let
            podman-runner = pkgs.callPackage ./nix/podman-runner.nix {};
          in
            pkgs.writeShellScriptBin "nextest-runner" ''
              set -e

              # Add all tools to PATH
              export PATH="${pkgs.lib.makeBinPath [
                podman-runner.podman-compose-runner
                pkgs.wait4x
                pkgs.sqlx-cli
                pkgs.cargo-nextest
                pkgs.coreutils
                pkgs.fontconfig
              ]}:$PATH"

              export FONTCONFIG_FILE=${pkgs.fontconfig.out}/etc/fonts/fonts.conf
              export FONTCONFIG_PATH=${pkgs.fontconfig.out}/etc/fonts

              # Set environment variables needed by tests
              export DATABASE_URL="${devEnvVars.DATABASE_URL}"
              export PG_CON="${devEnvVars.PG_CON}"

              # Function to cleanup on exit
              cleanup() {
                echo "Stopping deps..."
                ${podman-runner.podman-compose-runner}/bin/podman-compose-runner down || true
              }

              # Register cleanup function
              trap cleanup EXIT

              echo "Starting deps..."
              ${podman-runner.podman-compose-runner}/bin/podman-compose-runner up -d core-pg keycloak

              # Wait for PostgreSQL to be ready
              echo "Waiting for PostgreSQL to be ready..."
              ${pkgs.wait4x}/bin/wait4x postgresql "$DATABASE_URL" --timeout 120s

              echo "Running database migrations..."
              ${pkgs.sqlx-cli}/bin/sqlx migrate run --source lana/app/migrations
              echo "Waiting for Keycloak..."
              ${pkgs.wait4x}/bin/wait4x http http://localhost:8081 --timeout 120s

              # Run nextest using pre-built archive
              echo "Running cargo nextest from pre-built archive..."
              ${pkgs.cargo-nextest}/bin/cargo-nextest nextest run \
                --archive-file ${lana-test-archive}/test-archive.tar.zst \
                --workspace-remap .

              echo "Tests completed successfully!"
            '';

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

          check-sdl = pkgs.stdenv.mkDerivation {
            name = "check-sdl";
            src = rustSource;

            nativeBuildInputs = with pkgs; [
              diffutils
            ];

            buildInputs = [
              self.packages.${system}.write_sdl
              self.packages.${system}.write_customer_sdl
            ];

            buildPhase = ''
              # Generate SDL schemas using the pre-built binaries
              echo "Generating admin SDL..."
              ${self.packages.${system}.write_sdl}/bin/write_sdl > admin-schema-generated.graphql

              echo "Generating customer SDL..."
              ${self.packages.${system}.write_customer_sdl}/bin/write_customer_sdl > customer-schema-generated.graphql

              # Compare with committed schemas
              echo "Comparing admin SDL..."
              if ! diff -u lana/admin-server/src/graphql/schema.graphql admin-schema-generated.graphql; then
                echo "ERROR: Admin GraphQL schema is out of date!"
                echo "Run 'make sdl-rust-cargo' to update the schema"
                exit 1
              fi

              echo "Comparing customer SDL..."
              if ! diff -u lana/customer-server/src/graphql/schema.graphql customer-schema-generated.graphql; then
                echo "ERROR: Customer GraphQL schema is out of date!"
                echo "Run 'make sdl-rust-cargo' to update the schema"
                exit 1
              fi

              echo "SDL schemas are up to date ✓"
            '';

            installPhase = ''
              mkdir -p $out
              echo "SDL check passed" > $out/result.txt
            '';
          };

          check-dependency-dag = craneLib.mkCargoDerivation (
            commonArgs
            // {
              inherit cargoArtifacts;
              pname = "check-dependency-dag";
              doInstallCargoArtifacts = false;

              buildPhaseCargoCommand = ''
                echo "Checking dependency DAG..."
                cargo run --package check-dependency-dag --offline --quiet
                echo "Dependency DAG check passed ✓"
              '';

              installPhase = ''
                mkdir -p $out
                echo "Dependency DAG check passed" > $out/result.txt
              '';
            }
          );

          check-entity-rollups = pkgs.stdenv.mkDerivation {
            name = "check-entity-rollups";
            src = rustSource;

            nativeBuildInputs = with pkgs; [
              diffutils
              findutils
              coreutils
            ];

            buildInputs = [
              self.packages.${system}.entity-rollups
            ];

            buildPhase = ''
              # Create a temporary directory for generated schemas
              TEMP_SCHEMAS=$(mktemp -d)

              echo "Generating entity rollup schemas..."
              SQLX_OFFLINE=true ${self.packages.${system}.entity-rollups}/bin/entity-rollups update-schemas --force-recreate --schemas-out-dir "$TEMP_SCHEMAS"

              # Compare with committed schemas
              echo "Comparing entity rollup schemas..."

              # Check for differences
              if ! diff -r dev/entity-rollups/schemas "$TEMP_SCHEMAS"; then
                echo "ERROR: Entity rollup schemas are out of date!"
                echo "Run 'make update-schemas' to update the schemas"
                exit 1
              fi

              # Check for extra files in committed schemas
              committed_files=$(find dev/entity-rollups/schemas -type f -name "*.json" | sort)
              generated_files=$(find "$TEMP_SCHEMAS" -type f -name "*.json" | sort)

              committed_count=$(echo "$committed_files" | wc -l)
              generated_count=$(echo "$generated_files" | wc -l)

              if [ "$committed_count" -ne "$generated_count" ]; then
                echo "ERROR: Schema file count mismatch!"
                echo "Committed schemas: $committed_count files"
                echo "Generated schemas: $generated_count files"
                echo "Run 'make update-schemas' to update the schemas"
                exit 1
              fi

              echo "Entity rollup schemas are up to date ✓"
            '';

            installPhase = ''
              mkdir -p $out
              echo "Entity rollup schemas check passed" > $out/result.txt
            '';
          };

          check-fmt = pkgs.stdenv.mkDerivation {
            name = "check-fmt";
            src = ./.;

            nativeBuildInputs = with pkgs; [
              alejandra
              opentofu
              git
              findutils
            ];

            buildPhase = ''
              # Create a temporary directory and copy all files
              export HOME=$(mktemp -d)
              cp -r . $HOME/repo
              cd $HOME/repo

              # Initialize git repo for diff checking
              git init
              git config user.email "test@example.com"
              git config user.name "Test"
              git add -A
              git commit -m "Initial commit" > /dev/null 2>&1

              # Check Nix formatting
              echo "Checking Nix file formatting..."
              alejandra .

              # Check for any Nix files and verify formatting
              if find . -name "*.nix" -type f | head -1 | grep -q .; then
                if ! git diff --exit-code '*.nix' 2>/dev/null; then
                  echo "ERROR: Nix files are not formatted!"
                  echo "Run 'nix fmt .' to format all Nix files"
                  exit 1
                fi
                echo "✓ Nix files are properly formatted"
              else
                echo "✓ No Nix files found to check"
              fi

              # Reset for next check
              git add -A
              git commit -m "After nix format" > /dev/null 2>&1 || true

              # Check Terraform/OpenTofu formatting
              echo "Checking Terraform file formatting..."

              # Check if there are any .tf files
              if find . -name "*.tf" -type f | head -1 | grep -q .; then
                tofu fmt -recursive .

                if ! git diff --exit-code '*.tf' 2>/dev/null; then
                  echo "ERROR: Terraform files are not formatted!"
                  echo "Run 'tofu fmt -recursive .' to format all Terraform files"
                  exit 1
                fi
                echo "✓ Terraform files are properly formatted"
              else
                echo "✓ No Terraform files found to check"
              fi

              echo ""
              echo "All formatting checks passed ✓"
            '';

            installPhase = ''
              mkdir -p $out
              echo "Formatting checks passed" > $out/result.txt
            '';
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

        apps.simulation = flake-utils.lib.mkApp {
          drv = self.packages.${system}.simulation-runner;
          name = "simulation-runner";
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
