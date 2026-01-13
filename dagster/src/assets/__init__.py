"""Asset definitions for Lana data warehouse."""

from src.assets.bitfinex import bitfinex_protoassets
from src.assets.dbt import lana_dbt_protoassets, lana_dbt_seed_protoassets
from src.assets.file_report import (
    generated_file_report_protoassets,
    inform_lana_protoasset,
)
from src.assets.iris import iris_dataset_size
from src.assets.lana import (
    lana_source_protoassets,
    lana_to_dw_el_protoassets,
)
from src.assets.sumsub import sumsub_protoasset
