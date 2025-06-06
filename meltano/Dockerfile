# registry.gitlab.com/meltano/meltano:latest is also available in GitLab Registry
ARG MELTANO_IMAGE=meltano/meltano:latest
FROM $MELTANO_IMAGE

WORKDIR /project

# Copy only the meltano directory to maintain the same structure as local dev
COPY . ./meltano/

# Set the Meltano project root to the meltano subdirectory
ENV MELTANO_PROJECT_ROOT=./meltano

RUN meltano install

# Don't allow changes to containerized project files
ENV MELTANO_PROJECT_READONLY=1

# Expose default port used by `meltano ui`
EXPOSE 5000

ENTRYPOINT ["meltano"]
