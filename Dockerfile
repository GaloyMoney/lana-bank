FROM clux/muslrust:stable AS build
  COPY . /src
  WORKDIR /src
  RUN SQLX_OFFLINE=true cargo build --locked --release --all-features --bin lana-cli

FROM ubuntu
  COPY --from=build /src/target/x86_64-unknown-linux-musl/release/lana-cli /bin/lana-core
  USER 1000
  CMD ["lana-cli"]
