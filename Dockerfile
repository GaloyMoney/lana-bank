FROM clux/muslrust:stable AS build
  COPY . /src
  WORKDIR /src
  RUN SQLX_OFFLINE=true cargo build --locked --release --bin lava-core

FROM gcr.io/distroless/static
  COPY --from=build /src/target/x86_64-unknown-linux-musl/release/lava-core /bin/lava-core
  USER 1000
  ENV LAVA_HOME /lava
  CMD ["lava-core"]
