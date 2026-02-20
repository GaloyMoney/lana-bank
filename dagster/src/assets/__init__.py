"""Asset definitions for Lana data warehouse."""

from src.assets.bitfinex import bitfinex_protoassets
from src.assets.dbt import (
    TAG_KEY_ASSET_TYPE,
    TAG_VALUE_DBT_MODEL,
    create_dbt_model_assets,
    create_dbt_seed_assets,
)
from src.assets.file_report import (
    create_file_report_multi_asset,
    inform_lana_protoasset,
)
from src.assets.iris import iris_dataset_size
from src.assets.lana import (
    lana_source_protoassets,
    lana_to_dw_el_protoassets,
)
from src.assets.sumsub import sumsub_protoasset
