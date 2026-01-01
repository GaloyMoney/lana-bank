# Dagster

Dagster is our orchestrator and where we define and run the analytics and reporting data pipelines on top of Lana.


## Dev env

- You can make dagster services run alongside the application when you `make dev-up` by adding `DAGSTER=true` in an `.env` file at the repo root.
- Alternatively, you can start and stop the dagster services independently from Lana with `make dagster-up`, `make dagster-stop`, `make dagster-down`.

The webserver UI will be accessible at `localhost:3000`.

By default, no jobs will run automatically in the local env. If you want to activate automations so they run automatically after deployment, add this to your repo's `.env`.

```
DAGSTER_AUTOMATIONS_ACTIVE=true
```

You can also start with automations deactivated and then activate them yourself through the webserver UI or GraphQL.

## Formatting

From the repo root, rely on the shared Makefile + flake tooling:

```bash
make dagster-fmt        # apply black, isort, and sqlfmt
make dagster-fmt-check  # verify formatting (black/isort/sqlfmt)
```

## Env vars

Env vars are the preferred way to:
- Make dagster change its behaviour depending on the env
- Passing in secrets such as database credentials.

Follow these steps to use a new env var in dagster (assuming a mock `MY_ENV_VAR`):

- Within `.py` files, you can refer to the value of an env varwith: `dg.EnvVar("MY_ENV_VAR").get_value()`. Bear in mind that the lookup is safe, so if dagster can't find the var, this statement will simply return `None`.
- You will need to add `MY_ENV_VAR` to `dagster.yaml` under `run_launcher.config.env_vars`.
- And then also in the `docker-compose.dagster.yml` under `services.dagster_daemon.environment`.
- Additionally, if your var is used outside of dagster assets/sensors/etc (ie called on `.py` loading, not on dagster run runtime), you also need to add it under `services.dagster-code-location-lana-dw.environment`.

## Bumping dagster versions

Dagster releases simultaneously for their `core` and `library` python packages. You can check release numbers for each at https://github.com/dagster-io/dagster/releases.

To bump the dagster versions:
- Modify `dagster/Dockerfile` to bump in our code location image.
- Modify `dagster/Dockerfile_dagster` to bump in our local env dagster services.
- If deploying with a helm chart, make sure that the helm chart version you use is in sync with the version number that the code location image is using. Dagster helm charts follow the same versioning schedule as the `core` packages.

## Opentelemetry

The project is designed to send traces for every dagster run. You get this feature for free by properly using the method `add_callable_as_asset` of `DefinitionsBuilder` in `src/definitions.py`.

Each asset materialization gets represented as a single span, and any exception happening during the materialization runtime will result in a span with `error=true`.

# Testing the Sumsub extractor

To test the Sumsub extractor without wiring a new webhook:

- Complete a KYC flow on the staging deployment to generate a callback.
- From the staging bastion, dump only the data for the table:
- `make reset-deps`
- `psql postgres://user:password@localhost:5433/pg < public.inbox_events.sql`
- Open the UI at `http://localhost:3000/assets/sumsub_applicants` and materialize
