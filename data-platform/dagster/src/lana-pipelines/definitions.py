import json
import base64

import dagster as dg
import dlt
from dlt.destinations import bigquery

def build_definitions():

    @dlt.source()
    def lana_core_pg():
        
        @dlt.resource(name="lana_table")
        def lana_table():
            for i in range(1, 10):
                yield {"name": "some_name", "id": i}

        return lana_table


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

        base64_credentials = dg.EnvVar("TF_VAR_sa_creds").get_value()
        
        bigquery_dest = create_bigquery_destination(base64_credentials)
        
        pipeline = dlt.pipeline(
            pipeline_name="lana_pipeline",
            destination=bigquery_dest,
            dataset_name="counterweight_dataset"
        )
        
        load_info = pipeline.run(
            lana_core_pg().with_resources("lana_table"),
            write_disposition="replace"
        )
        
        context.log.info(f"Pipeline completed.")
        return load_info

    @dg.asset(
        op_tags={"operation": "example"},
        partitions_def=dg.DailyPartitionsDefinition("2024-01-01"),
    )
    def example_asset(context: dg.AssetExecutionContext):
        context.log.info(context.partition_key)

    partitioned_asset_job = dg.define_asset_job("partitioned_job", selection=[example_asset])
    lana_pipeline_job = dg.define_asset_job("lana_pipeline_job", selection=[lana_pipeline_asset])

    return dg.Definitions(
        assets=[example_asset, lana_pipeline_asset], 
        jobs=[partitioned_asset_job, lana_pipeline_job]
    )
     
defs = build_definitions()