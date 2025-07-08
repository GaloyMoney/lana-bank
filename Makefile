# Docker, Podman and Tilt
dev-up:
	cd dev && tilt up

dev-down:
	cd dev && tilt down

podman-service-start:
	@./dev/bin/podman-service-start.sh

# ── Container Management ──────────────────────────────────────────────────────────
# The ENGINE_DEFAULT and DOCKER_HOST environment variables are automatically set based on
# available container engines. To force use of podman, set ENGINE_DEFAULT=podman in your environment.
# The podman-* targets below are Linux-only and used for manual podman service setup.


# ── Test Targets ───────────────────────────────────────────────────────────────────

next-watch:
	cargo watch -s 'cargo nextest run'

clean-deps:
	./dev/bin/clean-deps.sh

start-deps:
	./dev/bin/docker-compose-up.sh

# Rust backend
setup-db:
	cd lana/app && cargo sqlx migrate run

sqlx-prepare:
	cargo sqlx prepare --workspace

reset-deps: clean-deps start-deps setup-db

run-server:
	cargo run --features sim-time --bin lana-cli -- --config ./bats/lana.yml | tee .e2e-logs

run-server-with-bootstrap:
	cargo run --all-features --bin lana-cli -- --config ./bats/lana.yml | tee .e2e-logs

check-code: check-code-rust-cargo check-code-apps check-code-tf

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
	nix run .#entity-rollups -- update-schemas --force-recreate

update-schemas-cargo:
	SQLX_OFFLINE=true cargo run --bin entity-rollups --all-features -- update-schemas --force-recreate

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
	@curl -s -o /dev/null -w "Response code: %{response_code}\n" http://admin.localhost:4455 || echo "Admin panel via proxy failed"
	@echo "Database connectivity check:"
	@echo "Container status:"
	@$${ENGINE_DEFAULT:-docker} ps --filter "label=com.docker.compose.project=lana-bank" --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}" || echo "Failed to check container status"
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

create-airflow-admin:
	meltano invoke airflow users create -e admin@galoy.io -f Admin -l Galoy -u admin -p admin --role Admin

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
	@$${ENGINE_DEFAULT:-docker} exec lana-bank-kratos-admin-pg-1 psql -U dbuser -d default -t -c "SELECT body FROM courier_messages WHERE recipient='$(EMAIL)' ORDER BY created_at DESC LIMIT 1;" | grep -Eo '[0-9]{6}' | head -n1

get-customer-login-code:
	@$${ENGINE_DEFAULT:-docker} exec lana-bank-kratos-customer-pg-1 psql -U dbuser -d default -t -c "SELECT body FROM courier_messages WHERE recipient='$(EMAIL)' ORDER BY created_at DESC LIMIT 1;" | grep -Eo '[0-9]{6}' | head -n1

get-superadmin-login-code:
	@$${ENGINE_DEFAULT:-docker} exec lana-bank-kratos-admin-pg-1 psql -U dbuser -d default -t -c "SELECT body FROM courier_messages WHERE recipient='admin@galoy.io' ORDER BY created_at DESC LIMIT 1;" | grep -Eo '[0-9]{6}' | head -n1
