next-watch:
	cargo watch -s 'cargo nextest run'

clean-deps:
	docker compose down

start-deps:
	docker compose up -d integration-deps

setup-db:
	cd core && cargo sqlx migrate run

reset-deps: clean-deps start-deps setup-db

run-server:
	cargo run --bin lava-core -- --config ./bats/lava.yml

rust-example:
	cargo run --bin cala-ledger-example-rust

update-lib-in-nodejs-example:
	cd cala-nodejs && SQLX_OFFLINE=true yarn build
	cd examples/nodejs && rm -rf ./node_modules && yarn install

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

sdl:
	SQLX_OFFLINE=true cargo run --bin write_sdl > core/schema.graphql

bump-cala-schema:
	curl https://raw.githubusercontent.com/GaloyMoney/cala/main/cala-server/schema.graphql > core/src/ledger/cala/graphql/schema.graphql

test-in-ci: start-deps
	sleep 3
	cd core && cargo sqlx migrate run
	cargo nextest run --verbose --locked

build-x86_64-unknown-linux-musl-release:
	SQLX_OFFLINE=true cargo build --release --locked --bin lava-core --target x86_64-unknown-linux-musl

build-x86_64-apple-darwin-release:
	bin/osxcross-compile.sh
