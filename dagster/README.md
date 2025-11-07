# Dagster

Dagster is our orchestrator and where we define and run the analytics and reporting data pipelines on top of Lana.


## Dev env

- You can make dagster services run alongside the application when you `make dev-up` by adding `DAGSTER=true` in an `.env` file at the repo root.
- Alternatively, you can start and stop the dagster services independently from Lana with `make dagster-up`, `make dagster-stop`, `make dagster-down`.

The webserver UI will be accessible at `localhost:3000`.

## Env vars

Env vars are the preferred way to:
- Make dagster change its behaviour depending on the env
- Passing in secrets such as database credentials.

Follow these steps to use a new env var in dagster (assuming a mock `MY_ENV_VAR`):

- Within `.py` files, you can refer to the value of an env varwith: `dg.EnvVar("MY_ENV_VAR").get_value()`. Bear in mind that the lookup is safe, so if dagster can't find the var, this statement will simply return `None`.
- You will need to add `MY_ENV_VAR` to `dagster.yaml` under `run_launcher.config.env_vars`.
- And then also in the `docker-compose.dagster.yml` under `services.dagster_daemon.environment`.
- Additionally, if your var is used outside of dagster assets/sensors/etc (ie called on `.py` loading, not on dagster run runtime), you also need to add it under `services.dagster-code-location-lana-dw.environment`.

## Opentelemetry

The project is designed to send traces for every dagster run. You get this feature for free by properly using the method `add_callable_as_asset` of `DefinitionsBuilder` in `src/definitions.py`.

Each asset materialization gets represented as a single span, and any exception happening during the materialization runtime will result in a span with `error=true`.
