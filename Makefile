dev-up: reset-tf-state
	cd dev && tilt up

dev-down:
	cd dev && tilt down

next-watch:
	cargo watch -s 'cargo nextest run'

clean-deps:
	docker compose down -t 1

start-deps:
	docker compose up --wait -d integration-deps

setup-db:
	cd lana/app && cargo sqlx migrate run

sqlx-prepare:
	cd lib/job && cargo sqlx prepare
	cd lib/audit && cargo sqlx prepare
	cd lib/outbox && cargo sqlx prepare
	cd core/governance && cargo sqlx prepare
	cd core/customer && cargo sqlx prepare
	cd core/user && cargo sqlx prepare
	cd core/deposit && cargo sqlx prepare
	cd core/chart-of-accounts && cargo sqlx prepare
	cd lana/app && cargo sqlx prepare
	cd lana/dashboard && cargo sqlx prepare

reset-tf-state:
	rm -rf tf/terraform.tfstate

reset-tf-provider:
	rm -rf tf/.terraform
	rm -rf tf/.terraform.lock.hcl

delete-bq-tables:
	cd tf && tofu state list | grep 'module\.setup\.google_bigquery_table\.' | awk '{print "-target='\''" $$1 "'\''"}' | xargs tofu destroy -auto-approve

init-bq: delete-bq-tables reset-tf-state clean-deps start-deps setup-db
	rm tf/import.tf || true
	cd tf && tofu init && tofu apply -auto-approve || true
	sleep 5
	cd tf && tofu apply -auto-approve
	git checkout tf/import.tf

reset-deps: reset-tf-state clean-deps start-deps setup-db

run-server:
	cargo run --bin lana-cli --features sim-time -- --config ./bats/lana-sim-time.yml | tee .e2e-logs

run-server-with-bootstrap:
	cargo run --bin lana-cli --all-features -- --config ./bats/lana-sim-time.yml | tee .e2e-logs

check-code: sdl customer-sdl
	git diff --exit-code lana/customer-server/src/graphql/schema.graphql
	git diff --exit-code lana/admin-server/src/graphql/schema.graphql
	SQLX_OFFLINE=true cargo fmt --check --all
	SQLX_OFFLINE=true cargo check
	SQLX_OFFLINE=true cargo clippy --all-features
	SQLX_OFFLINE=true cargo audit

clippy:
	SQLX_OFFLINE=true cargo clippy --all-features

build:
	SQLX_OFFLINE=true cargo build --locked

build-for-tests:
	SQLX_OFFLINE=true cargo build --locked --features sim-time

e2e: reset-tf-state clean-deps start-deps build-for-tests
	bats -t bats

e2e-in-ci: clean-deps start-deps build-for-tests
	lsof -i :5253 | tail -n 1 | cut -d" " -f2 | xargs -L 1 kill -9 || true
	SA_CREDS_BASE64=$$(cat ./dev/fake-service-account.json | tr -d '\n' | base64 -w 0) bats -t bats


sdl:
	SQLX_OFFLINE=true cargo run --bin write_sdl > lana/admin-server/src/graphql/schema.graphql
	cd apps/admin-panel && pnpm install && pnpm codegen

customer-sdl:
	SQLX_OFFLINE=true cargo run --bin write_customer_sdl > lana/customer-server/src/graphql/schema.graphql

test-in-ci: start-deps setup-db
	cargo nextest run --verbose --locked

build-x86_64-unknown-linux-musl-release:
	SQLX_OFFLINE=true cargo build --release --locked --bin lana-cli --target x86_64-unknown-linux-musl

build-x86_64-apple-darwin-release:
	bin/osxcross-compile.sh

start-admin:
	cd apps/admin-panel && pnpm install --frozen-lockfile && pnpm dev

start-customer-portal:
	cd apps/customer-portal && pnpm install --frozen-lockfile && pnpm dev

check-code-apps: check-code-apps-admin-panel

check-code-apps-admin-panel:
	cd apps/admin-panel && pnpm install --frozen-lockfile && pnpm lint && pnpm tsc-check && pnpm build

check-code-apps-customer-portal:
	cd apps/customer-portal && pnpm install --frozen-lockfile && pnpm lint && pnpm tsc-check && pnpm build

build-storybook-admin-panel:
	cd apps/admin-panel && pnpm install --frozen-lockfile && pnpm run build-storybook

# add https://xxx.ngrok-free.app/sumsub/callback to test integration with sumsub
ngrok:
	ngrok http 5253

tilt-in-ci:
	./dev/bin/tilt-ci.sh

test-cypress-in-ci-through-browserstack:
	cd apps/admin-panel && pnpm cypress:run browserstack

pg2bq-run:
	meltano run tap-postgres target-bigquery

bq-pipeline-run:
	meltano run dbt-bigquery:run

check-code-pipeline:
	meltano invoke sqlfluff:lint

lint-code-pipeline:
	meltano invoke sqlfluff:fix

bitfinex-run:
	meltano run tap-bitfinexapi target-bigquery
