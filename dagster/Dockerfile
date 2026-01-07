FROM python:3.13-alpine

# This makes sure that logs show up immediately instead of being buffered
ENV PYTHONUNBUFFERED=1

# Install SSL certificates and dependencies needed for HTTPS
RUN apk --no-cache add \
    curl \
    build-base \
    ca-certificates \
    openssl \
    libffi-dev \
    openssl-dev \
    && update-ca-certificates

RUN pip install --upgrade pip

# Install dagster general dependencies
ARG DAGSTER_VERSION=1.12.0
ARG DAGSTER_EXT_VERSION=0.28.1
ARG OTEL_VERSION=1.38.0
RUN pip install \
    dagster~=${DAGSTER_VERSION} \
    dagster-postgres~=${DAGSTER_EXT_VERSION} \
    dagster-k8s~=${DAGSTER_EXT_VERSION} \
    dagster-dbt~=${DAGSTER_EXT_VERSION} \
    opentelemetry-api~=${OTEL_VERSION} \
    opentelemetry-sdk~=${OTEL_VERSION} \
    opentelemetry-exporter-otlp-proto-grpc~=${OTEL_VERSION}

# Install lana specific packages
RUN pip install \
    dbt-core~=1.10.3 \
    dbt-bigquery~=1.10.3 \
    dlt[bigquery]~=1.18.1 \
    dagster-dlt~=${DAGSTER_EXT_VERSION} \
    pandas

# Add our project specific code
RUN mkdir -p /lana-dw
COPY src/ /lana-dw/src/

# Pre-generate dbt manifest at build time (local-only, no BQ connection needed)
# Dummy env vars required for profiles.yml Jinja evaluation - values don't matter
# since dbt parse doesn't actually connect to BigQuery
RUN DBT_BIGQUERY_PROJECT=build-placeholder \
    DBT_BIGQUERY_DATASET=build-placeholder \
    DBT_BIGQUERY_CREDENTIALS_JSON='{"type":"service_account","project_id":"x"}' \
    dbt parse --project-dir /lana-dw/src/dbt_lana_dw --profiles-dir /lana-dw/src/dbt_lana_dw

EXPOSE 4000

CMD ["dagster", "api", "grpc", "-h", "0.0.0.0", "-p", "4000", "-f", "/lana-dw/src/definitions.py", "-d", "/lana-dw"]
