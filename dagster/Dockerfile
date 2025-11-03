FROM python:3.13-alpine

# This makes sure that logs show up immediately instead of being buffered
ENV PYTHONUNBUFFERED=1

# Install SSL certificates and dependencies needed for HTTPS
RUN apk --no-cache add \
    curl \
    ca-certificates \
    openssl \
    libffi-dev \
    openssl-dev \
    && update-ca-certificates

RUN pip install --upgrade pip

# Install dagster general dependencies
RUN pip install \
    dagster~=1.11.16 \
    dagster-postgres~=0.27.16 \
    dagster-k8s~=0.27.16 \
    opentelemetry-api~=1.38.0 \
    opentelemetry-sdk~=1.38.0 \
    opentelemetry-exporter-otlp-proto-grpc~=1.38.0

# Install lana specific packages
RUN pip install \
    dlt[bigquery]~=1.18.1 \
    dagster-dlt~=0.28.0 \
    pandas

# Add our project specific code
RUN mkdir -p /lana-dw
COPY src/ /lana-dw/src/

EXPOSE 4000

CMD ["dagster", "api", "grpc", "-h", "0.0.0.0", "-p", "4000", "-f", "/lana-dw/src/definitions.py", "-d", "/lana-dw"]
