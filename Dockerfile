FROM clux/muslrust:stable AS build
  RUN mkdir /lava-home
  COPY . /src
  WORKDIR /src
  RUN SQLX_OFFLINE=true cargo build --locked --release --bin lava-core

FROM ubuntu
  COPY --from=build /src/target/x86_64-unknown-linux-musl/release/lava-core /bin/lava-core
  COPY --from=build --chown=1000:0 --chmod=755 /lava-home /lava
  USER 1000
  ENV LAVA_HOME /lava
  CMD ["lava-core"]
