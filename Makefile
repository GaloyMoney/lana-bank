dev-up:
	cd dev && tilt up

dev-down:
	cd dev && tilt down

next-watch:
	cargo watch -s 'cargo nextest run'

clean-deps:
	docker compose down -t 1

start-deps:
	docker compose up --wait -d

setup-db:
	cd lana/app && cargo sqlx migrate run

sqlx-prepare:
	cd lib/job && cargo sqlx prepare
	cd lib/audit && cargo sqlx prepare
	cd lib/outbox && cargo sqlx prepare
	cd core/governance && cargo sqlx prepare
	cd core/user && cargo sqlx prepare
	cd core/deposit && cargo sqlx prepare
	cd lana/app && cargo sqlx prepare
	cd lana/dashboard && cargo sqlx prepare

reset-tf-state:
	rm -rf tf/terraform.tfstate

reset-tf-provider:
	rm -rf tf/.terraform
	rm -rf tf/.terraform.lock.hcl

delete-bq-tables:
	cd tf && tofu state list | grep 'module\.setup\.google_bigquery_table\.' | awk '{print "-target='\''" $$1 "'\''"}' | xargs tofu destroy -auto-approve

run-tf:
	cd tf && tofu init && tofu apply -auto-approve

init-bq: delete-bq-tables reset-tf-state clean-deps start-deps setup-db
	rm tf/import.tf || true
	cd tf && tofu init && tofu apply -auto-approve || true
	sleep 5
	cd tf && tofu apply -auto-approve
	git checkout tf/import.tf

reset-deps: reset-tf-state clean-deps start-deps setup-db run-tf

run-server:
	cargo run --bin lana-cli --features sim-time -- --config ./bats/lana-sim-time.yml | tee .e2e-logs

check-code: sdl
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

e2e: reset-tf-state clean-deps start-deps build-for-tests run-tf
	bats -t bats

e2e-in-ci: bump-cala-docker-image clean-deps start-deps build-for-tests run-tf
	SA_CREDS_BASE64=$$(cat ./dev/fake-service-account.json | tr -d '\n' | base64 -w 0) bats -t bats


sdl:
	SQLX_OFFLINE=true cargo run --bin write_sdl > lana/admin-server/src/graphql/schema.graphql
	cd apps/admin-panel && pnpm install && pnpm codegen

bump-cala-schema:
	curl -H "Authorization: token ${GITHUB_TOKEN}" https://raw.githubusercontent.com/GaloyMoney/cala-enterprise/main/schema.graphql > lana/app/src/ledger/cala/graphql/schema.graphql

bump-cala-docker-image:
	docker compose pull cala

bump-cala: bump-cala-docker-image bump-cala-schema

test-in-ci: start-deps setup-db run-tf
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

push-dataform-branch:
	git push -f origin HEAD:${DATAFORM_BRANCH}

dataform-install:
	yarn install
	node_modules/.bin/dataform install

dataform-run:
	node_modules/.bin/dataform run --timeout 5m --schema-suffix=${DATAFORM_SCHEMA_SUFFIX} --vars=${DATAFORM_VARS}

dataform-run-dev:
	node_modules/.bin/dataform run --timeout 5m --schema-suffix=${DATAFORM_SCHEMA_SUFFIX} --vars=${DATAFORM_VARS} --tags=current-dev

dataform-run-staging:
	node_modules/.bin/dataform run --timeout 5m --schema-suffix=${DATAFORM_SCHEMA_SUFFIX} --vars="executionEnv=volcano-staging,devUser=${TF_VAR_name_prefix}"
