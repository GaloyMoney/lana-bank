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
	@echo "--- Configuring Podman ---"
	@if [ "$$(uname)" = "Linux" ]; then \
		echo "Applying Linux-specific podman configuration..." && \
		mkdir -p /etc/containers && \
		echo '{ "default": [{"type": "insecureAcceptAnything"}]}' > /etc/containers/policy.json || true && \
		echo 'unqualified-search-registries = ["docker.io"]' > /etc/containers/registries.conf || true && \
		grep -q "host.containers.internal" /etc/hosts || echo "127.0.0.1 host.containers.internal" >> /etc/hosts || true; \
	else \
		echo "Non-Linux system detected, skipping container configuration"; \
	fi
	@echo "--- Podman configuration done ---"

podman-service-start:
	@echo "--- Starting Podman service ---"
	@if [ "$$(uname)" = "Linux" ]; then \
		echo "Checking if podman socket is working..." && \
		if [ -S /run/podman/podman.sock ] && CONTAINER_HOST=unix:///run/podman/podman.sock timeout 3s podman version >/dev/null 2>&1; then \
			echo "Podman socket already working!"; \
		else \
			echo "Starting podman system service..." && \
			mkdir -p /run/podman && \
			podman system service --time=0 unix:///run/podman/podman.sock & \
			echo "Waiting for socket to be created..." && \
			for i in 1 2 3 4 5; do \
				if [ -S /run/podman/podman.sock ] && CONTAINER_HOST=unix:///run/podman/podman.sock timeout 3s podman version >/dev/null 2>&1; then \
					echo "Socket created and working!"; \
					break; \
				fi; \
				echo "Waiting... ($$i/5)"; \
				sleep 2; \
			done; \
		fi; \
	fi
	@echo "--- Podman service ready ---"

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
	@ls -la /run/podman/podman.sock 2>/dev/null || echo "Socket not found at /run/podman/podman.sock"
	@echo "Running podman processes:"
	@ps aux | grep podman || echo "No podman processes found"
	@echo "DOCKER_HOST: $${DOCKER_HOST:-not set}"
	@echo "--- End Debug Information ---"

# ── Container Management ──────────────────────────────────────────────────────────
start-deps-podman: podman-setup
	@DOCKER_HOST=unix:///run/podman/podman.sock ENGINE_DEFAULT=podman ./bin/docker-compose-up.sh

clean-deps-podman: 
	@DOCKER_HOST=unix:///run/podman/podman.sock ENGINE_DEFAULT=podman ./bin/clean-deps.sh

reset-deps-podman: clean-deps-podman start-deps-podman setup-db

# ── Test Targets ───────────────────────────────────────────────────────────────────
test-integration-podman: start-deps-podman
	@echo "--- Running Integration Tests with Podman ---"
	@cargo nextest run --verbose --locked
	@$(MAKE) clean-deps-podman

test-bats-podman: start-deps-podman
	@echo "--- Running BATS Tests with Podman ---"
	@nix build . -L
	@$(MAKE) run-bats-with-server
	@$(MAKE) clean-deps-podman

run-bats-with-server:
	@echo "--- Starting Lana server for BATS tests ---"
	@export REPO_ROOT=$$(git rev-parse --show-toplevel) && \
	source "$$REPO_ROOT/bats/helpers.bash" && \
	start_server_nix && \
	trap 'echo "--- Stopping Lana server ---"; stop_server' EXIT && \
	echo "--- Running BATS tests ---" && \
	bats -t bats

next-watch:
	cargo watch -s 'cargo nextest run'

clean-deps:
	./bin/clean-deps.sh

start-deps:
	./bin/docker-compose-up.sh

# Rust backend
setup-db:
	cd lana/app && cargo sqlx migrate run

sqlx-prepare:
	cargo sqlx prepare --workspace

reset-deps: clean-deps start-deps setup-db

run-server:
	cargo run --bin lana-cli --features sim-time -- --config ./bats/lana-sim-time.yml | tee .e2e-logs

run-server-with-bootstrap:
	cargo run --bin lana-cli --all-features -- --config ./bats/lana-sim-time.yml | tee .e2e-logs

check-code: check-code-rust check-code-apps

check-code-rust: sdl-rust
	git diff --exit-code lana/customer-server/src/graphql/schema.graphql
	git diff --exit-code lana/admin-server/src/graphql/schema.graphql
	SQLX_OFFLINE=true cargo fmt --check --all
	SQLX_OFFLINE=true cargo check
	SQLX_OFFLINE=true cargo clippy --all-features
	SQLX_OFFLINE=true cargo audit
	cargo deny check
	cargo machete

clippy:
	SQLX_OFFLINE=true cargo clippy --all-features

build:
	SQLX_OFFLINE=true cargo build --locked

build-for-tests:
	SQLX_OFFLINE=true cargo build --locked --features sim-time

build-for-tests-nix:
	nix build .

e2e: clean-deps start-deps build-for-tests-nix
	bats -t bats

sdl-rust:
	SQLX_OFFLINE=true cargo run --bin write_sdl > lana/admin-server/src/graphql/schema.graphql
	SQLX_OFFLINE=true cargo run --bin write_customer_sdl > lana/customer-server/src/graphql/schema.graphql

sdl-js:
	cd apps/admin-panel && pnpm install && pnpm codegen
	cd apps/customer-portal && pnpm install && pnpm codegen

full-sdl: sdl-rust sdl-js

# Frontend Apps
check-code-apps: sdl-js check-code-apps-admin-panel check-code-apps-customer-portal

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

test-cypress-in-ci-locally:
	cd apps/admin-panel && pnpm cypress:run headless

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

build-x86_64-apple-darwin-release:
	bin/osxcross-compile.sh

test-in-ci: start-deps setup-db
	cargo nextest run --verbose --locked

build-x86_64-unknown-linux-musl-release:
	SQLX_OFFLINE=true cargo build --release --all-features --locked --bin lana-cli --target x86_64-unknown-linux-musl

e2e-in-ci: clean-deps start-deps build-for-tests
	lsof -i :5253 | tail -n 1 | cut -d" " -f2 | xargs -L 1 kill -9 || true
	SA_CREDS_BASE64=$$(cat ./dev/fake-service-account.json | tr -d '\n' | base64 -w 0) bats -t bats

# Login code retrieval
get-admin-login-code:
	@docker exec lana-bank-kratos-admin-pg-1 psql -U dbuser -d default -t -c "SELECT body FROM courier_messages WHERE recipient='$(EMAIL)' ORDER BY created_at DESC LIMIT 1;" | grep -Eo '[0-9]{6}' | head -n1

get-customer-login-code:
	@docker exec lana-bank-kratos-customer-pg-1 psql -U dbuser -d default -t -c "SELECT body FROM courier_messages WHERE recipient='$(EMAIL)' ORDER BY created_at DESC LIMIT 1;" | grep -Eo '[0-9]{6}' | head -n1

get-superadmin-login-code:
	@docker exec lana-bank-kratos-admin-pg-1 psql -U dbuser -d default -t -c "SELECT body FROM courier_messages WHERE recipient='admin@galoy.io' ORDER BY created_at DESC LIMIT 1;" | grep -Eo '[0-9]{6}' | head -n1
