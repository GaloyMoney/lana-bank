FROM clux/muslrust:stable AS build
  RUN mkdir /lana-home
  COPY . /src
  WORKDIR /src
  RUN SQLX_OFFLINE=true cargo build --locked --all-features --bin lana-cli
  # Use a single RUN command to find and copy the binary
  RUN find /src/target -name "lana-cli" -type f -exec cp {} /lana-cli \;

FROM ubuntu
  COPY --from=build /lana-cli /bin/lana-cli
  COPY --from=build --chown=1000:0 --chmod=755 /lana-home /lana
  USER 1000
  ENV LANA_HOME=/lana
  CMD ["lana-cli"]
