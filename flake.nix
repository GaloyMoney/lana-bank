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
  }: let
    releaseVersion = builtins.getEnv "RELEASE_BUILD_VERSION";
  in
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
          python313 = super.python313.override {
            packageOverrides = pySelf: pySuper: let
              disableTests = pkg:
                pkg.overrideAttrs (old: {
                  doCheck = false;
                  doInstallCheck = false;
                });
              # Only disable tests for specific packages that need it
              packagesToDisableTests = [
                "black"
                "isort"
                "sqlfmt"
                "weasyprint"
              ];
            in
              builtins.listToAttrs (
                map (name: {
                  inherit name;
                  value = disableTests pySuper.${name};
                })
                (builtins.filter (name: builtins.hasAttr name pySuper) packagesToDisableTests)
              );
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
          || pkgs.lib.hasInfix "/lib/gotenberg/config/" path
          || pkgs.lib.hasInfix "/lana/admin-server/src/graphql/schema.graphql" path
          || pkgs.lib.hasInfix "/lana/customer-server/src/graphql/schema.graphql" path
          || pkgs.lib.hasInfix "/apps/admin-panel/messages/generated/en.json" path
          || pkgs.lib.hasInfix "/dev/lanacli/src/graphql/" path;
      };

      commonArgs = {
        src = rustSource;
        strictDeps = true;
        SQLX_OFFLINE = true;
        # clang and lld for faster linking (configured in .cargo/config.toml)
        nativeBuildInputs =
          pkgs.lib.optionals pkgs.stdenv.isLinux [pkgs.clang pkgs.lld]
          ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [pkgs.llvmPackages.lld];
      };

      cargoArtifacts = craneLib.buildDepsOnly (commonArgs
        // {
          cargoExtraArgs = "--features mock-custodian,sumsub-testing";
        });

      # Get the default version from Cargo.toml
      defaultVersion = (craneLib.crateNameFromCargoToml {src = rustSource;}).version;

      # Use release version if set, otherwise fall back to Cargo.toml version
      cliVersion =
        if releaseVersion != ""
        then releaseVersion
        else defaultVersion;

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
          cargoExtraArgs = "-p lana-cli --features mock-custodian,sumsub-testing";
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

      lana-cli-release = let
        rustTarget = "x86_64-unknown-linux-musl";
        muslCC = pkgs.pkgsCross.musl64.stdenv.cc;
      in
        craneLibMusl.buildPackage {
          version = cliVersion; # Use the conditional version
          src = rustSource;
          strictDeps = true;
          cargoToml = ./lana/cli/Cargo.toml;
          doCheck = false;
          pname = "lana-cli-release";
          CARGO_PROFILE = "release";
          SQLX_OFFLINE = true;
          CARGO_BUILD_TARGET = rustTarget;
          cargoExtraArgs = "-p lana-cli --features sim-bootstrap --target ${rustTarget}";

          RELEASE_BUILD_VERSION = cliVersion;

          # clang + lld for linking (handles response files, avoiding ARG_MAX)
          nativeBuildInputs = [pkgs.clang pkgs.lld];

          # Add musl target for static linking
          depsBuildBuild = [muslCC];

          # Use clang as linker driver with lld backend
          # clang handles response files properly, avoiding ARG_MAX issues
          CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER = "clang";
          CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_RUSTFLAGS = "-C link-arg=--target=x86_64-unknown-linux-musl -C link-arg=-fuse-ld=lld -C link-arg=--sysroot=${muslCC.libc}";
          CC_x86_64_unknown_linux_musl = "${muslCC}/bin/x86_64-unknown-linux-musl-gcc";
          TARGET_CC = "${muslCC}/bin/x86_64-unknown-linux-musl-gcc";
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

          nativeBuildInputs = commonArgs.nativeBuildInputs ++ [pkgs.cargo-nextest];
        }
      );

      rustToolchainMusl = rustVersion.override {
        extensions = ["rust-src"];
        targets = ["x86_64-unknown-linux-musl"];
      };

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
          cocogitto
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
          python313Packages.black
          python313Packages.isort
          python313Packages.sqlfmt
          podman
          podman-compose
          cachix
          ps
          curl
          fly
          tilt
          procps
          poppler-utils
          keycloak
          tokio-console
          # Documentation tools
          # Font packages for PDF generation
          fontconfig
          dejavu_fonts # Provides serif, sans-serif, and monospace
        ]
        ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
          clang
          lld
          xvfb-run
          cypress
          python313Packages.weasyprint

          slirp4netns
          fuse-overlayfs

          util-linux
          psmisc
          iptables
        ]
        ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
          llvmPackages.lld
        ];

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
          lana-cli-release = lana-cli-release;

          lana-deps = cargoArtifacts;

          lana-test-archive = lana-test-archive;

          write_sdl = craneLib.buildPackage (
            individualCrateArgs
            // {
              pname = "write_sdl";
              cargoExtraArgs = "-p codegen --bin write_sdl";
            }
          );

          write_customer_sdl = craneLib.buildPackage (
            individualCrateArgs
            // {
              pname = "write_customer_sdl";
              cargoExtraArgs = "-p codegen --bin write_customer_sdl";
            }
          );

          write_translation_labels = craneLib.buildPackage (
            individualCrateArgs
            // {
              pname = "write_translation_labels";
              cargoExtraArgs = "-p codegen --bin write_translation_labels";
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
              pkgs.podman
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
              pkgs.poppler-utils
              pkgs.libuuid
            ];
          in
            pkgs.symlinkJoin {
              name = "bats-runner";
              paths = [
                podman-runner.podman-compose-runner
                pkgs.podman
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
                pkgs.poppler-utils
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
                export DAGSTER="''${DAGSTER:-"true"}"
                export GOTENBERG="''${GOTENBERG:-"false"}"
                export MAILCRAB="''${MAILCRAB:-"false"}"

                # Build compose file arguments
                COMPOSE_FILES=(-f docker-compose.yml)
                if [[ "''${DAGSTER:-false}" == "true" ]]; then
                  COMPOSE_FILES+=(-f docker-compose.dagster.yml)
                fi
                if [[ "''${GOTENBERG:-false}" == "true" ]]; then
                  COMPOSE_FILES+=(-f docker-compose.gotenberg.yml)
                fi
                if [[ "''${MAILCRAB:-false}" == "true" ]]; then
                  COMPOSE_FILES+=(-f docker-compose.mailcrab.yml)
                fi

                # Function to cleanup on exit
                cleanup() {
                  if [[ -n "''${KEEP_PODMAN_UP:-}" ]]; then
                    echo "KEEP_PODMAN_UP set — skipping podman-compose cleanup."
                    return 0
                  fi
                  echo "Stopping podman-compose..."
                  podman-compose-runner ''${COMPOSE_FILES[@]} down || true
                }

                # Register cleanup function
                trap cleanup EXIT

                echo "Starting podman-compose in detached mode..."
                # Start databases first
                podman-compose-runner ''${COMPOSE_FILES[@]} up -d core-pg keycloak-pg

                # Wait for both databases to be ready
                echo "Waiting for PostgreSQL to be ready..."
                wait4x postgresql "${devEnvVars.PG_CON}" --timeout 120s
                echo "Waiting for Keycloak PostgreSQL to be ready..."
                wait4x postgresql "postgresql://dbuser:secret@localhost:5437/default?sslmode=disable" --timeout 120s

                # Now start remaining services after databases are confirmed ready
                echo "Starting all other services..."
                podman-compose-runner ''${COMPOSE_FILES[@]} up -d

                # Wait for Keycloak with retry logic
                echo "Waiting for Keycloak to be ready..."
                for attempt in {1..3}; do
                  echo "Attempt $attempt/3: Checking Keycloak health..."
                  if wait4x http http://localhost:8081 --timeout 60s; then
                    echo "SUCCESS: Keycloak is ready"
                    break
                  fi

                  if [ $attempt -eq 3 ]; then
                    echo "ERROR: Keycloak failed to start after 3 attempts. Dumping logs..."
                    podman-compose-runner ''${COMPOSE_FILES[@]} logs keycloak || true
                    exit 1
                  fi

                  echo "Keycloak not ready, restarting (attempt $attempt/3)..."
                  podman-compose-runner ''${COMPOSE_FILES[@]} restart keycloak
                  sleep 5
                done

                if [[ "''${DAGSTER}" == "true" ]]; then
                  echo "Waiting for Dagster GraphQL endpoint to be ready..."
                  wait4x http http://localhost:3000/graphql --timeout 180s
                fi

                if [[ "''${GOTENBERG}" == "true" ]]; then
                  echo "Waiting for Gotenberg to be ready..."
                  wait4x http http://localhost:3030/health --timeout 60s
                fi

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
              pkgs.coreutils
            ];
          in
            pkgs.symlinkJoin {
              name = "simulation-runner";
              paths = [
                podman-runner.podman-compose-runner
                pkgs.wait4x
                pkgs.gnugrep
                pkgs.coreutils
                lana-cli-bootstrap
              ];
              postBuild = ''
                mkdir -p $out/bin
                cat > $out/bin/simulation-runner << 'EOF'
                #!${pkgs.bash}/bin/bash
                set -e

                # Add all tools to PATH
                export PATH="${binPath}:$PATH"

                # Set environment variables needed by simulations
                export PG_CON="${devEnvVars.PG_CON}"
                export DATABASE_URL="${devEnvVars.DATABASE_URL}"
                export ENCRYPTION_KEY="${devEnvVars.ENCRYPTION_KEY}"
                # Use info level to capture both success and error logs
                export RUST_LOG="info"

                # Function to cleanup on exit
                cleanup() {
                  echo "Stopping podman-compose..."
                  podman-compose-runner down || true
                  if [ -f .server.pid ]; then
                    kill "$(cat .server.pid)" 2>/dev/null || true
                  fi
                }

                # Register cleanup function
                trap cleanup EXIT

                echo "Starting podman-compose in detached mode..."
                podman-compose-runner up -d

                # Wait for PostgreSQL to be ready
                echo "Waiting for PostgreSQL to be ready..."
                wait4x postgresql "${devEnvVars.PG_CON}" --timeout 120s

                echo "Running cli"
                export LANA_CONFIG="./bats/lana-bootstrap.yml"
                ${lana-cli-bootstrap}/bin/lana-cli 2>&1 | tee server.log &
                echo "$!" > .server.pid

                # Wait for simulation to complete by polling logs
                # "transitioning to realtime" is logged when sim_bootstrap completes successfully
                echo "Waiting for simulation to complete..."
                MAX_WAIT=1800  # 30 minutes max for simulation to complete
                ELAPSED=0
                INTERVAL=5

                while [ $ELAPSED -lt $MAX_WAIT ]; do
                  # Check for simulation errors first
                  if grep -q "ERROR sim_bootstrap" server.log; then
                    echo "❌ Simulation failed with error; dumping last 200 lines of logs:"
                    tail -n 200 server.log
                    exit 1
                  fi

                  # Check for successful completion
                  if grep -q "transitioning to realtime" server.log; then
                    echo "✅ Simulation completed successfully!"
                    # Verify health check is now available
                    if wait4x http http://localhost:5253/health --timeout 30s; then
                      echo "✅ Health check passed!"
                      exit 0
                    else
                      echo "⚠️ Simulation completed but health check failed"
                      tail -n 200 server.log
                      exit 1
                    fi
                  fi

                  sleep $INTERVAL
                  ELAPSED=$((ELAPSED + INTERVAL))
                  echo "Still waiting... ($ELAPSED seconds elapsed)"
                done

                echo "❌ Simulation timed out after $MAX_WAIT seconds"
                echo "Dumping last 200 lines of logs:"
                tail -n 200 server.log
                exit 1
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
              # Start databases first
              ${podman-runner.podman-compose-runner}/bin/podman-compose-runner up -d core-pg keycloak-pg

              # Wait for both databases to be ready
              echo "Waiting for PostgreSQL to be ready..."
              ${pkgs.wait4x}/bin/wait4x postgresql "$DATABASE_URL" --timeout 120s
              echo "Waiting for Keycloak PostgreSQL to be ready..."
              ${pkgs.wait4x}/bin/wait4x postgresql "postgresql://dbuser:secret@localhost:5437/default?sslmode=disable" --timeout 120s

              # Now start remaining services after databases are confirmed ready
              echo "Starting all other services..."
              ${podman-runner.podman-compose-runner}/bin/podman-compose-runner up -d

              # Wait for Keycloak with retry logic
              echo "Waiting for Keycloak to be ready..."
              for attempt in {1..3}; do
                echo "Attempt $attempt/3: Checking Keycloak health..."
                if ${pkgs.wait4x}/bin/wait4x http http://localhost:8081 --timeout 60s; then
                  echo "SUCCESS: Keycloak is ready"
                  break
                fi

                if [ $attempt -eq 3 ]; then
                  echo "ERROR: Keycloak failed to start after 3 attempts. Dumping logs..."
                  ${podman-runner.podman-compose-runner}/bin/podman-compose-runner logs keycloak || true
                  exit 1
                fi

                echo "Keycloak not ready, restarting (attempt $attempt/3)..."
                ${podman-runner.podman-compose-runner}/bin/podman-compose-runner restart keycloak
                sleep 5
              done

              echo "Running database migrations..."
              ${pkgs.sqlx-cli}/bin/sqlx migrate run --source lana/app/migrations

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

          dagster-format = pkgs.stdenv.mkDerivation {
            name = "dagster-format-check";
            src = ./.;

            nativeBuildInputs = [
              pkgs.python313Packages.black
              pkgs.python313Packages.isort
              pkgs.python313Packages.sqlfmt
            ];

            buildPhase = ''
              # Nix builders default HOME to /homeless-shelter (not writable). sqlfmt writes a cache.
              export HOME="${TMPDIR:-/tmp}/home"
              export XDG_CACHE_HOME="$HOME/.cache"
              mkdir -p "$XDG_CACHE_HOME"

              cd dagster
              black --check --diff src
              isort --check-only src
              sqlfmt --check src/dbt_lana_dw/models
            '';

            installPhase = ''
              mkdir -p $out
              echo "Dagster formatting clean" > $out/result.txt
            '';
          };

          workspace-deny = craneLib.cargoDeny {
            src = rustSource;
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
                echo "Run 'make sdl-rust' to update the schema"
                exit 1
              fi

              echo "Comparing customer SDL..."
              if ! diff -u lana/customer-server/src/graphql/schema.graphql customer-schema-generated.graphql; then
                echo "ERROR: Customer GraphQL schema is out of date!"
                echo "Run 'make sdl-rust' to update the schema"
                exit 1
              fi

              echo "SDL schemas are up to date ✓"
            '';

            installPhase = ''
              mkdir -p $out
              echo "SDL check passed" > $out/result.txt
            '';
          };

          check-translation-labels = pkgs.stdenv.mkDerivation {
            name = "check-translation-labels";
            src = rustSource;

            nativeBuildInputs = with pkgs; [
              diffutils
              coreutils
            ];

            buildInputs = [
              self.packages.${system}.write_translation_labels
            ];

            buildPhase = ''
              # Copy source to writable location so the binary can write output files
              cp -r . $TMPDIR/workdir
              cd $TMPDIR/workdir

              echo "Generating translation labels..."
              ${self.packages.${system}.write_translation_labels}/bin/write_translation_labels

              # Switch back to read-only source for diffing committed versions
              cd $NIX_BUILD_TOP/$sourceRoot

              echo "Comparing generated translation labels..."
              if ! diff -u apps/admin-panel/messages/generated/en.json $TMPDIR/workdir/apps/admin-panel/messages/generated/en.json; then
                echo "ERROR: Generated translation labels are out of date!"
                echo "Run 'make generate-translation-labels' to update"
                exit 1
              fi

              echo "Translation labels are up to date"
            '';

            installPhase = ''
              mkdir -p $out
              echo "Translation labels check passed" > $out/result.txt
            '';
          };

          check-custom-lints = craneLib.mkCargoDerivation (
            commonArgs
            // {
              inherit cargoArtifacts;
              pname = "check-custom-lints";
              doInstallCargoArtifacts = false;

              buildPhaseCargoCommand = ''
                echo "Running custom lints..."
                cargo run --package custom-lints --offline --quiet
                echo "Custom lints check passed ✓"
              '';

              installPhase = ''
                mkdir -p $out
                echo "Custom lints check passed" > $out/result.txt
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

          check-default-config = pkgs.stdenv.mkDerivation {
            name = "check-default-config";
            src = pkgs.lib.cleanSourceWith {
              src = ./.;
              filter = path: type:
                type == "directory" || pkgs.lib.hasInfix "/dev/lana.default.yml" path;
            };

            nativeBuildInputs = with pkgs; [
              diffutils
            ];

            buildInputs = [
              lana-cli-bootstrap
            ];

            buildPhase = ''
              echo "Generating default config..."
              ${lana-cli-bootstrap}/bin/lana-cli dump-default-config > default-config-generated.yml

              echo "Comparing default config..."
              if ! diff -u dev/lana.default.yml default-config-generated.yml; then
                echo "ERROR: Default config is out of date!"
                echo "Run 'make generate-default-config' to update the config"
                exit 1
              fi

              echo "Default config is up to date ✓"
            '';

            installPhase = ''
              mkdir -p $out
              echo "Default config check passed" > $out/result.txt
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
