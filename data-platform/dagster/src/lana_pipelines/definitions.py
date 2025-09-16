import dagster as dg

from lana_pipelines.assets import build_lana_source_asset, build_lana_to_dw_el_asset 

def build_definitions():

    lana_to_dw_el_tables = (
        "core_deposit_events_rollup",
        "core_withdrawal_events_rollup"
    )

    lana_source_assets = [
        build_lana_source_asset(table_name=table_name)
        for table_name in lana_to_dw_el_tables
    ]

    lana_to_dw_el_assets = [
        build_lana_to_dw_el_asset(table_name=table_name)
        for table_name in lana_to_dw_el_tables
    ]

    lana_to_dw_el_job = dg.define_asset_job("lana_to_dw_el_job", selection=lana_to_dw_el_assets)


    all_assets = lana_source_assets + lana_to_dw_el_assets
    all_jobs = [lana_to_dw_el_job]

    return dg.Definitions(
        assets=all_assets, 
        jobs=all_jobs
    )
     
defs = build_definitions()