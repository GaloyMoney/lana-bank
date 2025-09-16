import dagster as dg

from lana_pipelines.assets import build_core_to_bq_el_asset 

def build_definitions():

    table_names = (
        "core_deposit_events_rollup",
        "core_withdrawal_events_rollup"
    )

    el_assets = [
        build_core_to_bq_el_asset(table_name=table_name)
        for table_name in table_names
    ]

    lana_to_dw_job = dg.define_asset_job("lana_to_dw_job", selection=el_assets)


    all_assets = el_assets
    all_jobs = [lana_to_dw_job]

    return dg.Definitions(
        assets=all_assets, 
        jobs=all_jobs
    )
     
defs = build_definitions()