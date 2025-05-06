FROM clux/muslrust:stable AS build
  RUN mkdir /lana-home
  COPY . /src
  WORKDIR /src
  RUN SQLX_OFFLINE=true cargo build --locked --release --bin lana-cli

FROM ubuntu
  COPY --from=build /src/target/x86_64-unknown-linux-musl/release/lana-cli /bin/lana-core
  COPY --from=build --chown=1000:0 --chmod=755 /lana-home /lana
  USER 1000
  ENV LANA_HOME /lana
  CMD ["lana-cli"]
