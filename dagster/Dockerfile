FROM python:3.13-alpine

# This makes sure that logs show up immediately instead of being buffered
ENV PYTHONUNBUFFERED=1

RUN apk --no-cache add curl

RUN pip install --upgrade pip

# Install dagster general dependencies
RUN pip install \
    dagster~=1.11.16 \
    dagster-postgres~=0.27.16 \
    dagster-k8s~=0.27.16

# Install lana specific packages
RUN pip install \
    pandas

# Add our project specific code
RUN mkdir -p /lana-dw/
COPY src/definitions.py /lana-dw/definitions.py

EXPOSE 4000

CMD ["dagster", "api", "grpc", "-h", "0.0.0.0", "-p", "4000", "-f", "/lana-dw/definitions.py"]
