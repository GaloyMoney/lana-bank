FROM python:3.13-alpine

# Copy your Dagster project. You may need to replace the filepath depending on your project structure
COPY . /

# This makes sure that logs show up immediately instead of being buffered
ENV PYTHONUNBUFFERED=1

RUN apk --no-cache add curl

RUN pip install --upgrade pip

# Install dagster and any other dependencies your project requires
RUN \
    pip install \
        dagster \
        dagster-postgres \
        dagster-k8s \
        # add any other dependencies here
        pandas

# Create the iris_analysis directory and download the definitions.py file
RUN mkdir -p /iris_analysis/ && \
    curl -o /iris_analysis/definitions.py https://raw.githubusercontent.com/dagster-io/dagster/refs/heads/master/examples/deploy_k8s/iris_analysis/definitions.py

WORKDIR /iris_analysis/

# Expose the port that your Dagster instance will run on
EXPOSE 3030