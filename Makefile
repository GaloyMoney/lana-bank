# Docker, Podman and Tilt
dev-up:
	cd dev && tilt up

dev-down:
	cd dev && tilt down

# ── Podman Setup ──────────────────────────────────────────────────────────────────
# These targets handle podman setup in an OS-aware manner:
# - Linux: Configures /etc/containers policy and registries
# - macOS: Uses default podman configuration (no additional setup needed)
podman-setup: podman-check podman-configure podman-service-start

podman-check:
	@echo "--- Checking for Podman ---"
	@command -v podman >/dev/null 2>&1 || { echo "Error: podman not found. Please install podman first."; exit 1; }
	@command -v podman-compose >/dev/null 2>&1 || { echo "Error: podman-compose not found. Please install podman-compose first."; exit 1; }
	@echo "--- Podman binaries found ---"

podman-configure:
	@./dev/bin/podman-configure.sh

podman-service-start:
	@./dev/bin/podman-service-start.sh

podman-service-stop:
	@echo "--- Stopping Podman service ---"
	@pkill -f "podman system service" || echo "No podman service to stop"
	@echo "--- Podman service stopped ---"

podman-debug:
	@echo "--- Podman Debug Information ---"
	@echo "OS: $$(uname)"
	@echo "Podman version:"
	@podman version || echo "Podman not found"
	@echo "Docker version:"
	@docker version || echo "Docker not found"
	@echo "Podman info:"
	@podman info || echo "Podman info failed"
	@echo "Socket status:"
	@ls -la /run/podman/podman.sock 2>/dev/null || echo "System socket not found at /run/podman/podman.sock"
	@ls -la $${XDG_RUNTIME_DIR:-/run/user/$$(id -u)}/podman/podman.sock 2>/dev/null || echo "User socket not found"
	@echo "Dynamic socket detection result:"
	@./dev/bin/podman-get-socket.sh || echo "Socket detection failed"
	@echo "Running podman processes:"
	@ps aux | grep podman || echo "No podman processes found"
	@echo "DOCKER_HOST: $${DOCKER_HOST:-not set}"
	@echo "--- End Debug Information ---"

# ── Container Management ──────────────────────────────────────────────────────────
start-deps-podman: podman-setup
	@DOCKER_HOST=$$(./dev/bin/podman-get-socket.sh) ENGINE_DEFAULT=podman ./dev/bin/docker-compose-up.sh
	wait4x postgresql $${PG_CON}

clean-deps-podman: 
	@DOCKER_HOST=$$(./dev/bin/podman-get-socket.sh) ENGINE_DEFAULT=podman ./dev/bin/clean-deps.sh

reset-deps-podman: clean-deps-podman start-deps-podman setup-db

# ── Test Targets ───────────────────────────────────────────────────────────────────
test-integration-podman: start-deps-podman
	@echo "--- Running Integration Tests with Podman ---"
	@$(MAKE) setup-db
	@cargo nextest run --verbose --locked
	@$(MAKE) clean-deps-podman

test-bats-podman: start-deps-podman
	@echo "--- Running BATS Tests with Podman ---"
	@$(MAKE) setup-db
	@nix build . -L
	@./dev/bin/run-bats-with-server.sh
	@$(MAKE) clean-deps-podman

next-watch:
	cargo watch -s 'cargo nextest run'

clean-deps:
	./dev/bin/clean-deps.sh

start-deps:
	./dev/bin/docker-compose-up.sh
	wait4x postgresql $${PG_CON}

# Rust backend
setup-db:
	cd lana/app && cargo sqlx migrate run

sqlx-prepare:
	cargo sqlx prepare --workspace

reset-deps: clean-deps start-deps setup-db

run-server:
	cargo run --features sim-time --bin lana-cli -- --config ./bats/lana-sim-time.yml | tee .e2e-logs

run-server-with-bootstrap:
	cargo run --all-features --bin lana-cli -- --config ./bats/lana-sim-time.yml | tee .e2e-logs

check-code: check-code-rust check-code-apps check-code-tf

check-code-tf:
	tofu fmt -recursive .
	git diff --exit-code *.tf

# Default (nix-based) code checking
check-code-rust: sdl-rust update-schemas
	git diff --exit-code lana/customer-server/src/graphql/schema.graphql
	git diff --exit-code lana/admin-server/src/graphql/schema.graphql
	git diff --exit-code lana/entity-rollups/schemas
	test -z "$$(git ls-files --others --exclude-standard lana/entity-rollups/schemas)"
	nix build .#check-code -L --option sandbox false

# Cargo alternative for faster compilation during development
check-code-rust-cargo: sdl-rust-cargo update-schemas-cargo
	git diff --exit-code lana/customer-server/src/graphql/schema.graphql
	git diff --exit-code lana/admin-server/src/graphql/schema.graphql
	git diff --exit-code lana/entity-rollups/schemas
	test -z "$$(git ls-files --others --exclude-standard lana/entity-rollups/schemas)"
	SQLX_OFFLINE=true cargo fmt --check --all
	SQLX_OFFLINE=true cargo check
	SQLX_OFFLINE=true cargo clippy --all-features
	SQLX_OFFLINE=true cargo audit
	cargo deny check
	cargo machete

# Default (nix-based) schema update
update-schemas:
	SQLX_OFFLINE=true nix run .#entity-rollups -- update-schemas

# Cargo alternative for faster compilation during development
update-schemas-cargo:
	SQLX_OFFLINE=true cargo run --bin entity-rollups --all-features -- update-schemas

clippy:
	SQLX_OFFLINE=true cargo clippy --all-features

build:
	SQLX_OFFLINE=true cargo build --locked

build-for-tests:
	nix build .

e2e: clean-deps start-deps build-for-tests
	bats -t bats

# Default (nix-based) SDL generation
sdl-rust:
	SQLX_OFFLINE=true nix run .#write_sdl -- > lana/admin-server/src/graphql/schema.graphql
	SQLX_OFFLINE=true nix run .#write_customer_sdl -- > lana/customer-server/src/graphql/schema.graphql

# Cargo alternative for faster compilation during development
sdl-rust-cargo:
	SQLX_OFFLINE=true cargo run --bin write_sdl > lana/admin-server/src/graphql/schema.graphql
	SQLX_OFFLINE=true cargo run --bin write_customer_sdl > lana/customer-server/src/graphql/schema.graphql

sdl-js:
	cd apps/admin-panel && pnpm install && pnpm codegen
	cd apps/customer-portal && pnpm install && pnpm codegen

full-sdl: sdl-rust sdl-js

# Cargo alternative for full SDL generation
full-sdl-cargo: sdl-rust-cargo sdl-js

# Frontend Apps
check-code-apps: sdl-js check-code-apps-admin-panel check-code-apps-customer-portal
	git diff --exit-code apps/admin-panel/lib/graphql/generated/
	git diff --exit-code apps/customer-portal/lib/graphql/generated/

start-admin:
	cd apps/admin-panel && pnpm install --frozen-lockfile && pnpm dev

start-customer-portal:
	cd apps/customer-portal && pnpm install --frozen-lockfile && pnpm dev

check-code-apps-admin-panel:
	cd apps/admin-panel && pnpm install --frozen-lockfile && pnpm lint && pnpm tsc-check && pnpm build

check-code-apps-customer-portal:
	cd apps/customer-portal && pnpm install --frozen-lockfile && pnpm lint && pnpm tsc-check && pnpm build

build-storybook-admin-panel:
	cd apps/admin-panel && pnpm install --frozen-lockfile && pnpm run build-storybook

test-cypress-in-ci:
	@echo "--- Starting Cypress Tests ---"
	@echo "Working directory: $(shell pwd)"
	@echo "Node version: $(shell node --version 2>/dev/null || echo 'Node not found')"
	@echo "Pnpm version: $(shell pnpm --version 2>/dev/null || echo 'Pnpm not found')"
	@echo "Checking if services are running..."
	@echo "--- Service Health Checks ---"
	@echo "Core server status:"
	@curl -s -o /dev/null -w "Response code: %{response_code}\n" http://localhost:5253/health || echo "Core server health check failed"
	@echo "GraphQL endpoint status:" 
	@curl -s -o /dev/null -w "Response code: %{response_code}\n" http://localhost:5253/graphql || echo "GraphQL endpoint check failed"
	@echo "Admin panel status:"
	@curl -s -o /dev/null -w "Response code: %{response_code}\n" http://localhost:3001 || echo "Admin panel direct check failed"
	@curl -s -o /dev/null -w "Response code: %{response_code}\n" http://localhost:4455/admin || echo "Admin panel via proxy failed"
	@echo "Database connectivity check:"
	@podman exec lana-bank-kratos-admin-pg-1 pg_isready -U dbuser || echo "Kratos admin DB not ready"
	@podman exec lana-bank-core-pg-1 pg_isready -U user || echo "Core DB not ready"
	@echo "Container status:"
	@podman ps --filter "label=com.docker.compose.project=lana-bank" --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}" || echo "Failed to check container status"
	@echo "--- End Health Checks ---"
	@echo "--- Running Cypress Tests ---"
	cd apps/admin-panel && CI=true pnpm cypress:run headless

# Meltano
bitfinex-run:
	meltano run tap-bitfinexapi target-bigquery

sumsub-run:
	meltano run tap-sumsubapi target-bigquery

pg2bq-run:
	meltano run tap-postgres target-bigquery

bq-pipeline-run:
	meltano run dbt-bigquery:run

check-code-pipeline:
	meltano invoke sqlfluff:lint

lint-code-pipeline:
	meltano invoke sqlfluff:fix

bq-drop-old-run:
	meltano run drop-old-relations

bq-drop-all-run:
	meltano run drop-all-relations

# misc
sumsub-webhook-test: # add https://xxx.ngrok-free.app/sumsub/callback to test integration with sumsub
	ngrok http 5253

tilt-in-ci:
	./dev/bin/tilt-ci.sh

start-cypress-stack:
	./dev/bin/start-cypress-stack.sh

# Default (nix-based) test in CI
test-in-ci: start-deps setup-db
	nix build .#test-in-ci -L --option sandbox false

# Cargo alternative for faster compilation during development
test-in-ci-cargo: start-deps setup-db
	cargo nextest run --verbose --locked

build-x86_64-unknown-linux-musl-release:
	SQLX_OFFLINE=true cargo build --release --all-features --locked --bin lana-cli --target x86_64-unknown-linux-musl

# Login code retrieval
get-admin-login-code:
	@podman exec lana-bank-kratos-admin-pg-1 psql -U dbuser -d default -t -c "SELECT body FROM courier_messages WHERE recipient='$(EMAIL)' ORDER BY created_at DESC LIMIT 1;" | grep -Eo '[0-9]{6}' | head -n1

get-customer-login-code:
	@podman exec lana-bank-kratos-customer-pg-1 psql -U dbuser -d default -t -c "SELECT body FROM courier_messages WHERE recipient='$(EMAIL)' ORDER BY created_at DESC LIMIT 1;" | grep -Eo '[0-9]{6}' | head -n1

get-superadmin-login-code:
	@podman exec lana-bank-kratos-admin-pg-1 psql -U dbuser -d default -t -c "SELECT body FROM courier_messages WHERE recipient='admin@galoy.io' ORDER BY created_at DESC LIMIT 1;" | grep -Eo '[0-9]{6}' | head -n1
