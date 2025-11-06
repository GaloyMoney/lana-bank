FROM python:3.13-alpine

# This makes sure that logs show up immediately instead of being buffered
ENV PYTHONUNBUFFERED=1

RUN apk --no-cache add curl build-base

RUN pip install --upgrade pip

# Install dagster general dependencies
RUN pip install \
    dagster~=1.12.0 \
    dagster-postgres~=0.28.0 \
    dagster-docker~=0.28.0 \
    dagster-dlt~=0.28.0 \
    dagster-dbt~=0.28.0 \
    dagster-k8s~=0.28.0 \
    dlt[bigquery]~=1.18.2 \
    dbt-bigquery~=1.10.3

# Install lana specific packages
RUN pip install \
    pandas

# Add GCP secrets file
ENV SECRETS_FOLDER=/lana_dw/secrets/
RUN mkdir -p $SECRETS_FOLDER
COPY keyfile.json $SECRETS_FOLDER

# Add our project specific code
RUN mkdir -p /lana_dw/
COPY src/ /lana_dw

# Install `generate-es-reports`
RUN pip install /lana_dw/generate-es-reports

EXPOSE 4000

CMD ["dagster", "api", "grpc", "-h", "0.0.0.0", "-p", "4000", "-d", "/lana_dw/", "-f", "/lana_dw/definitions.py"]
