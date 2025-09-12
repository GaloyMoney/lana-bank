# Dagster

`lana-bank`s dagster project.

## How to run

Start the containers with `docker compose up -d --build`.
Stop them with `docker compose down`.

## Code locations

Code locations are independent code packages that contain dagster `Definitions`. A single dagster deployment can fetch objects from multiple code locations. For now we only have one.

If you need to add another for some reason:
- Add a new service in the docker compose file, like the `lana_pipelines` one.
- Add an entry in `workspace.yaml`, adding the container name and the port the grpc server is listening on.

## How to add environment variables within dagster context

- Use dagster's `EnvVar`.
- In `dagster.yml`, add the name of the var under: `run_launcher > config > env_vars`.
- Finally, pass it to the service `dagster_daemon` defined in the docker compose file, either as a explicit env var or as part of an `.env` file.
