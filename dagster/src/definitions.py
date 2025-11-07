"""Dagster definitions entry point - builds all Dagster objects."""

import dagster as dg
from typing import Callable

from src.core import lana_assetifier
from src.assets import iris_dataset_size
from src.otel import init_telemetry

class DefinitionsBuilder:

    def __init__(self):
        self.assets = []
    
    def init_telemetry(self):
        init_telemetry()

    def add_callable_as_asset(self, callable: Callable):
        asset = lana_assetifier(asset_key=callable.__name__, callable=callable)
        self.assets.append(asset)

    def build(self) -> dg.Definitions:
        return dg.Definitions(assets=self.assets)


definition_builder = DefinitionsBuilder()

definition_builder.init_telemetry()
definition_builder.add_callable_as_asset(iris_dataset_size)

defs = definition_builder.build()
