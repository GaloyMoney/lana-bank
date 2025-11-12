"""Dagster definitions entry point - builds all Dagster objects."""

import dagster as dg
from typing import Callable, Tuple

from src.core import lana_assetifier
from src.assets import (
    iris_dataset_size,
    bitfinex_ticker,
    bitfinex_trades,
    bitfinex_order_book,
)
from src.otel import init_telemetry


class DefinitionsBuilder:

    def __init__(self):
        self.assets = []
        self.jobs = []

    def init_telemetry(self):
        init_telemetry()

    def add_callable_as_asset(self, callable: Callable) -> dg.asset:
        asset = lana_assetifier(asset_key=callable.__name__, callable=callable)
        self.assets.append(asset)

        return asset

    def add_job_from_assets(
        self, job_name: str, assets: Tuple[dg.asset, ...]
    ) -> dg.job:
        new_job = dg.define_asset_job(name=job_name, selection=assets)
        self.jobs.append(new_job)

        return new_job

    def build(self) -> dg.Definitions:
        return dg.Definitions(assets=self.assets, jobs=self.jobs)


definition_builder = DefinitionsBuilder()

definition_builder.init_telemetry()
definition_builder.add_callable_as_asset(iris_dataset_size)

bitfinex_callables = (bitfinex_ticker, bitfinex_trades, bitfinex_order_book)
bitfinex_assets = tuple(
    definition_builder.add_callable_as_asset(bitfinex_callable)
    for bitfinex_callable in bitfinex_callables
)
definition_builder.add_job_from_assets(
    job_name="bitfinex_el",
    assets=bitfinex_assets
)

defs = definition_builder.build()
