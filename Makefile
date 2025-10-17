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

check-code-rust:
	SQLX_OFFLINE=true cargo fmt --check --all
	SQLX_OFFLINE=true cargo check
	SQLX_OFFLINE=true cargo clippy --all-features --all-targets
	SQLX_OFFLINE=true cargo audit
	cargo deny check --hide-inclusion-graph
	cargo machete


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

run-server-normal:
	cargo run --features sim-time,mock-custodian,sumsub-testing --bin lana-cli -- --config ./bats/lana-normal.yml 2>&1 | tee .e2e-logs

run-server:
	cargo run --features sim-time,mock-custodian,sumsub-testing --bin lana-cli -- --config ./bats/lana.yml > >(tee .e2e-logs) 2>&1

run-server-nix:
	nix run . -- --config ./bats/lana.yml 2>&1 | tee .e2e-logs

run-server-with-bootstrap:
	cargo run --all-features --bin lana-cli -- --config ./bats/lana.yml | tee .e2e-logs

check-code: check-code-apps
	nix flake check

update-schemas:
	SQLX_OFFLINE=true cargo run --package entity-rollups --all-features -- update-schemas --force-recreate

e2e: reset-deps
	bats -t bats

# Cargo alternative for faster compilation during development
sdl-rust:
	SQLX_OFFLINE=true cargo run --bin write_sdl > lana/admin-server/src/graphql/schema.graphql
	SQLX_OFFLINE=true cargo run --bin write_customer_sdl > lana/customer-server/src/graphql/schema.graphql

# Generate default configuration file
generate-default-config:
	SQLX_OFFLINE=true cargo run -p lana-cli --all-features -- dump-default-config > dev/lana.default.yml

sdl-js:
	cd apps/admin-panel && pnpm install && pnpm codegen
	cd apps/customer-portal && pnpm install && pnpm codegen

sdl: sdl-rust sdl-js

# Cargo alternative for full SDL generation
sdl-cargo: sdl-rust-cargo sdl-js

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
	@curl -s -o /dev/null -w "Response code: %{response_code}\n" http://localhost:3001/api/health || echo "Admin panel direct check failed"
	@curl -s -o /dev/null -w "Response code: %{response_code}\n" http://admin.localhost:4455/api/health || echo "Admin panel via proxy failed"
	@echo "Database connectivity check:"
	@echo "Container status:"
	@$${ENGINE_DEFAULT:-docker} ps --filter "label=com.docker.compose.project=lana-bank" --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}" || echo "Failed to check container status"
	@echo "--- End Health Checks ---"
	@echo "--- Running Cypress Tests ---"
	@echo "Installing Cypress binary if missing..."
	cd apps/admin-panel && pnpm exec cypress install
	cd apps/admin-panel && CI=true pnpm cypress:run-headless

# Meltano
meltano-bitfinex-run:
	meltano run tap-bitfinexapi target-bigquery

meltano-sumsub-run:
	meltano run tap-sumsubapi target-bigquery

meltano-pg2bq-run:
	meltano run tap-postgres target-bigquery

meltano-pipeline-seed:
	meltano run dbt-bigquery:seed

meltano-pipeline-run:
	meltano run dbt-bigquery:run

meltano-pipeline-test:
	meltano run dbt-bigquery:test

meltano-sqlfluff-lint:
	cd meltano && meltano invoke sqlfluff:lint

meltano-sqlfluff-fix:
	cd meltano && meltano invoke sqlfluff:fix

meltano-sqlfluff-install:
	cd meltano && rm -rf .meltano/utilities/sqlfluff && meltano install utility sqlfluff && .meltano/utilities/sqlfluff/venv/bin/pip install --quiet --upgrade platformdirs

meltano-drop-old-run:
	meltano run drop-old-relations

meltano-drop-all-run:
	meltano run drop-all-relations

# Meltano EL writes into a different BQ dataset than dbt. This empties that dataset.
bq-drop-meltano-landing:
	bq ls -n 100000 --project_id=$(DBT_BIGQUERY_PROJECT) $(TARGET_BIGQUERY_DATASET) | awk 'NR>2 {print $$1}' | xargs -P 32 -n1 -I{} bash -c 'echo "Deleting: $(DBT_BIGQUERY_PROJECT):$(TARGET_BIGQUERY_DATASET).{}"; bq rm -f -t $(DBT_BIGQUERY_PROJECT):$(TARGET_BIGQUERY_DATASET).{}'

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

auth-kcadm:
	kcadm.sh config credentials --server "$$KC_URL" --realm "$$REALM" --user "$$ADMIN_USER" --password "$$ADMIN_PASS"

### this is what is passed on to the server
auth-secret:
	source <(./dev/keycloak/create-client-local-dev.sh --emit-env)

### token is fetched from the server using the client secret
auth-token:
	curl -s -X POST "$$KC_URL/realms/$$KC_REALM/protocol/openid-connect/token" \
		-d grant_type=client_credentials \
		-d client_id="$$KC_CLIENT_ID" \
		-d client_secret="$$KC_CLIENT_SECRET" | jq -r .access_token

create-user:
	./dev/keycloak/create-user.sh

docs-serve:
	cd docs && mdbook serve -n 0.0.0.0
