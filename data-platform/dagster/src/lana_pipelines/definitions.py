import dagster as dg

from lana_pipelines.assets import build_core_to_bq_el_asset 

def build_definitions():

    el_assets = build_core_to_bq_el_asset()

    lana_to_dw_job = dg.define_asset_job("lana_to_dw_job", selection=[el_assets])

    return dg.Definitions(
        assets=[el_assets], 
        jobs=[lana_to_dw_job]
    )
     
defs = build_definitions()