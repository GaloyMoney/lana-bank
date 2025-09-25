# generate-es-reports

`generate-es-reports` is a small Python CLI app to read tables from some database, convert their contents to some file formats and push the files to cloud storage.

## Usage

- Install the package: `poetry install`
- Set up the following env vars.
    - `GOOGLE_APPLICATION_CREDENTIALS_ENVVAR_KEY`: path to a Google Cloud Platform credentials JSON file.
    - `DBT_BIGQUERY_PROJECT_ENVVAR_KEY`: BigQuery project to read data from.
    - `DBT_BIGQUERY_DATASET_ENVVAR_KEY`: BigQuery dataset to read data from.
    - `DOCS_BUCKET_NAME_ENVVAR_KEY`: the name of the destination bucket when writing to Google Cloud Storage.
    - `AIRFLOW_CTX_DAG_RUN_ID_ENVVAR_KEY`: Optional. Set by Airflow in scheduled runs to give a unique run id to execution, although you can tamper it manually to give runs unique IDs. If it's not set, run id defaults to `dev`.
    - `USE_LOCAL_FS`: Optional. If informed, the tool will write to local file system instead of Google Cloud Storage. 
- Call `generate-es-reports run` in your terminal.

## Testing

The app has only one simple test that runs the happy path with hardcoded inputs and writing on the local filesystem. It ain't much, but will help with obvious bugs.

To run it:
- `poetry install`
- `poetry run pytest`

## With meltano

The main goal of `generate-es-reports` is to be called from the outer Meltano project as a utility.

Assuming you've set up the Meltano environment, you can call this tool with `meltano invoke generate-es-reports run`.

If you modify the CLI interface of `generate-es-reports`, please be mindful of checking if it breaks changes in how the meltano utility entry is set up and adjust accordingly.
