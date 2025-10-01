import dlt


def prepare_lana_el_pipeline(lana_core_pg, dw_bq, table_name):
    dlt_postgres_resource = lana_core_pg.create_dlt_postgres_resource(
        table_name=table_name
    )
    dlt_bq_destination = dw_bq.get_dlt_destination()

    pipeline = dlt.pipeline(
        pipeline_name=table_name,
        destination=dlt_bq_destination,
        dataset_name=dw_bq.target_dataset,
    )

    # Ready to be called with source and disposition already hardcoded
    def wrapped_pipeline():
        load_info = pipeline.run(
            dlt_postgres_resource,
            write_disposition="replace",
            table_name=table_name,
        )
        return load_info

    return wrapped_pipeline
