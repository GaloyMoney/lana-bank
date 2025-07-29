# Meltano

Meltano project to run multiple data tasks, such as:
- Extract-Load (EL) towards the Data Warehouse (DW).
- `dbt` transformations within the DW.
- Report generation.

## Setup and development

### Requirements

We assume you've followed the `nix` and `direnv` instructions in Lana's root `README.md`, which prepare the Python environment for you. You can check by running `which meltano`, which show indicate that the meltano binary is reachable and pointing to a `nix` store (`/nix/store/...`). Running the data stack locally alsorequires you to have a development instance of `lana` available in your machine. Please refer to the root `README.md` to do that. 

Additionally, you will also need a GCP keyfile for a Service Account to secure BigQuery (BQ) access for whatever environment you're working on and add its contents into your environment. Follow these steps:

- Format your JSON keyfile into base 64 with `base64 -w0 /path/to/your/file.json`.
- At the root `.env`, add this line: `TF_VAR_sa_creds=ewY3RfaWQiOiAogIY2eXBlIjogIn....`

### Install meltano deps

From this dir, run `meltano install` to set up the different meltano plugins that are defined in the project.

### Running

Once your app is running, you can run an EL from the app database to the Data Warehouse with: `meltano run tap-postgres target-bigquery`. After that, you should be able to see the loaded data in tour BQ development datasets.

You can then proceed to run the `dbt` transformations on BQ by running: `meltano invoke dbt-bigquery run`. You can pass any other `dbt` CLI commands and options to `meltano invoke dbt-bigquery`.
