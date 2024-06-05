next-watch:
	cargo watch -s 'cargo nextest run'

clean-deps:
	docker compose down -t 1

start-deps:
	docker compose up -d integration-deps

setup-db:
	cd core && sleep 2 && cargo sqlx migrate run

sqlx-prepare:
	cd core && cargo sqlx prepare

reset-deps: clean-deps start-deps setup-db

run-server:
	cargo run --bin lava-core -- --config ./bats/lava.yml

check-code: sdl
	git diff --exit-code core/schema.graphql
	SQLX_OFFLINE=true cargo fmt --check --all
	SQLX_OFFLINE=true cargo check
	SQLX_OFFLINE=true cargo clippy --all-features
	SQLX_OFFLINE=true cargo audit

build:
	SQLX_OFFLINE=true cargo build --locked

e2e: clean-deps start-deps build
	bats -t bats

e2e-in-ci: bump-cala-docker-image e2e

sdl:
	SQLX_OFFLINE=true cargo run --bin write_sdl > core/schema.graphql

bump-cala-schema:
	curl https://raw.githubusercontent.com/GaloyMoney/cala/main/cala-server/schema.graphql > core/src/ledger/cala/graphql/schema.graphql

bump-cala-docker-image:
	docker pull us.gcr.io/galoy-org/cala:edge

bump-cala: bump-cala-docker-image bump-cala-schema

test-in-ci: start-deps
	sleep 3
	cd core && cargo sqlx migrate run
	cargo nextest run --verbose --locked

build-x86_64-unknown-linux-musl-release:
	SQLX_OFFLINE=true cargo build --release --locked --bin lava-core --target x86_64-unknown-linux-musl

build-x86_64-apple-darwin-release:
	bin/osxcross-compile.sh
