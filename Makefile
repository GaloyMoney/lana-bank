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
	cd core && cargo sqlx migrate run

sqlx-prepare:
	cd core && cargo sqlx prepare

reset-tf-state:
	rm -rf tf/terraform.tfstate

reset-tf-provider:
	rm -rf tf/.terraform
	rm -rf tf/.terraform.lock.hcl

run-tf:
	cd tf && tofu init && tofu apply -auto-approve

init-bq: reset-tf-state clean-deps start-deps setup-db
	rm tf/import.tf || true
	cd tf && tofu init && tofu apply -auto-approve || true
	sleep 5
	cd tf && tofu apply -auto-approve
	git checkout tf/import.tf

reset-deps: reset-tf-state clean-deps start-deps setup-db run-tf

run-server:
	cargo run --bin lava-core -- --config ./bats/lava.yml

check-code: public-sdl admin-sdl
	git diff --exit-code core/src/server/public/schema.graphql
	git diff --exit-code core/src/server/admin/schema.graphql
	SQLX_OFFLINE=true cargo fmt --check --all
	SQLX_OFFLINE=true cargo check
	SQLX_OFFLINE=true cargo clippy --all-features
	SQLX_OFFLINE=true cargo audit

build:
	SQLX_OFFLINE=true cargo build --locked

e2e: clean-deps start-deps build run-tf
	bats -t bats

e2e-in-ci: bump-cala-docker-image clean-deps start-deps build run-tf
	BQ_SERVICE_ACCOUNT_BASE64=$$(cat ./dev/fake-service-account.json | tr -d '\n' | base64 -w 0) bats -t bats

public-sdl:
	SQLX_OFFLINE=true cargo run --bin write_public_sdl > core/src/server/public/schema.graphql

admin-sdl:
	SQLX_OFFLINE=true cargo run --bin write_admin_sdl > core/src/server/admin/schema.graphql
	cd apps/admin-panel && pnpm install && pnpm codegen

bump-cala-schema:
	curl -H "Authorization: token ${GITHUB_TOKEN}" https://raw.githubusercontent.com/GaloyMoney/cala-enterprise/main/schema.graphql > core/src/ledger/cala/graphql/schema.graphql

bump-cala-docker-image:
	docker compose pull cala

bump-cala: bump-cala-docker-image bump-cala-schema

test-in-ci: start-deps setup-db run-tf
	cargo nextest run --verbose --locked

build-x86_64-unknown-linux-musl-release:
	SQLX_OFFLINE=true cargo build --release --locked --bin lava-core --target x86_64-unknown-linux-musl

build-x86_64-apple-darwin-release:
	bin/osxcross-compile.sh

start-admin:
	cd apps/admin-panel && pnpm install --frozen-lockfile && pnpm dev

start-customer-portal:
	cd apps/customer-portal && pnpm install --frozen-lockfile && pnpm dev

check-code-apps: check-code-apps-admin-panel check-code-apps-customer-portal

check-code-apps-admin-panel:
	cd apps/admin-panel && pnpm install --frozen-lockfile && pnpm lint && pnpm tsc-check && pnpm build

check-code-apps-customer-portal:
	cd apps/customer-portal && pnpm install --frozen-lockfile && pnpm lint && pnpm tsc-check && pnpm build

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
