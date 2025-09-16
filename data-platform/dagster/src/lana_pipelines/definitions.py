import json
import base64

import dagster as dg
import dlt
from dlt.destinations import bigquery
from dlt.sources.credentials import ConnectionStringCredentials

from lana_pipelines.resources import create_postgres_resource

def build_definitions():

    def create_bigquery_destination(base64_credentials):
        """Create BigQuery destination with programmatic credentials configuration"""

        try:
            # Decode the base64-encoded JSON credentials
            credentials_json = base64.b64decode(base64_credentials).decode('utf-8')
            credentials = json.loads(credentials_json)
        except (base64.binascii.Error, json.JSONDecodeError) as e:
            raise ValueError(f"Failed to decode base64 credentials: {e}")
        
        # Validate that we have the required fields
        required_fields = ["type", "project_id", "private_key", "client_email"]
        for field in required_fields:
            if field not in credentials:
                raise ValueError(f"Missing required field '{field}' in credentials")
        
        return bigquery(
            credentials=credentials,
            project_id=credentials["project_id"],
            location="US"  # Optional: specify location
        )

    @dg.asset()
    def lana_pipeline_asset(context: dg.AssetExecutionContext):
        """Asset that runs only the lana_table resource and writes to Big Query"""
        context.log.info(f"Running lana_table pipeline.")

        postgres_credentials = ConnectionStringCredentials()
        postgres_credentials.drivername = "postgresql"
        postgres_credentials.database = "pg"
        postgres_credentials.username = "user"
        postgres_credentials.password = "password"
        postgres_credentials.host = "172.17.0.1"
        postgres_credentials.port = 5433

        postgres_table_name = "core_deposit_events_rollup"

        postgres_resource = create_postgres_resource(postgres_credentials, table_name=postgres_table_name)
    
        base64_credentials = dg.EnvVar("TF_VAR_sa_creds").get_value()
        bigquery_dest = create_bigquery_destination(base64_credentials)
        
        pipeline = dlt.pipeline(
            pipeline_name="lana_pipeline",
            destination=bigquery_dest,
            dataset_name="counterweight_dataset"
        )
 
        load_info = pipeline.run(
            postgres_resource,
            write_disposition="replace",
            table_name="test_table"
        )
        
        context.log.info(f"Pipeline completed.")
        context.log.info(load_info)
        return load_info

    lana_pipeline_job = dg.define_asset_job("lana_pipeline_job", selection=[lana_pipeline_asset])

    return dg.Definitions(
        assets=[lana_pipeline_asset], 
        jobs=[lana_pipeline_job]
    )
     
defs = build_definitions()