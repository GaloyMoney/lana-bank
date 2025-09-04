import dagster as dg
import dlt

def build_definitions():

    @dlt.source()
    def lana_core_pg():
        
        @dlt.resource(name="lana_table", write_disposition="replace")
        def lana_table():
            for i in range(1, 10):
                yield {"name": "some_name", "id": i}

        return lana_table


    @dg.asset()
    def lana_pipeline_asset(context: dg.AssetExecutionContext):
        """Asset that runs only the lana_table resource and writes to Postgres"""
        context.log.info(f"Running lana_table pipeline for partition: {context.partition_key}")
        
        # Create the pipeline
        pipeline = dlt.pipeline(
            pipeline_name="lana_pipeline",
            destination="postgres",
            dataset_name="lana_data"
        )
        
        # Run only the lana_table resource from the source
        load_info = pipeline.run(lana_core_pg().with_resources("lana_table"))
        
        context.log.info(f"Pipeline completed. Load info: {load_info}")
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