import dagster as dg
import dlt
from dlt.sources.credentials import ConnectionStringCredentials
from dagster_dbt import DbtCliResource, dbt_assets

from lana_pipelines.resources import create_postgres_resource
from lana_pipelines.destinations import create_bigquery_destination
from lana_pipelines.resources import dbt_manifest_path

def build_lana_source_asset(table_name):

    lana_source_asset = dg.AssetSpec(
        key=f"el_source_asset__lana__{table_name}",
        tags={"asset_type": "el_source__asset", "system": "lana"},
    )

    return lana_source_asset


def build_lana_to_dw_el_asset(table_name):

    name = f"{table_name}"
    
    @dg.asset(
        key_prefix=["lana"],
        name=name, 
        deps=[f"el_source_asset__lana__{table_name}"],
        tags={"asset_type": "el_target__asset", "system": "lana"},
    )
    def lana_to_dw_el_asset(context: dg.AssetExecutionContext):
        context.log.info(
            f"Running lana_to_dw_el_asset pipeline for table {table_name}."
        )

        postgres_credentials = ConnectionStringCredentials()
        postgres_credentials.drivername = "postgresql"
        postgres_credentials.database = "pg"
        postgres_credentials.username = "user"
        postgres_credentials.password = "password"
        postgres_credentials.host = "172.17.0.1"
        postgres_credentials.port = 5433

        postgres_resource = create_postgres_resource(
            postgres_credentials, table_name=table_name
        )

        base64_credentials = dg.EnvVar("TF_VAR_sa_creds").get_value()
        bigquery_dest = create_bigquery_destination(base64_credentials)

        pipeline = dlt.pipeline(
            pipeline_name=name,
            destination=bigquery_dest,
            dataset_name="counterweight_dataset",
        )

        destination_table_name = f"{table_name}"
        
        load_info = pipeline.run(
            postgres_resource,
            write_disposition="replace",
            table_name=destination_table_name,
        )

        context.log.info(f"Pipeline completed.")
        context.log.info(load_info)
        return load_info

    return lana_to_dw_el_asset


def build_dbt_assets():

    @dbt_assets(manifest=dbt_manifest_path)
    def dbt_models(context: dg.AssetExecutionContext, dbt: DbtCliResource):
        yield from dbt.cli(["build"], context=context).stream()

    return dbt_models
