"""Dagster definitions entry point - builds all Dagster objects."""

import os
from typing import List, Tuple, Union

import dagster as dg
from src.assets import (
    bitfinex_protoassets,
    iris_dataset_size,
    lana_source_protoassets,
    lana_to_dw_el_protoassets,
)
from src.resources import get_project_resources
from src.core import Protoasset, lana_assetifier
from src.otel import init_telemetry

DAGSTER_AUTOMATIONS_ACTIVE = os.getenv(
    "DAGSTER_AUTOMATIONS_ACTIVE", ""
).strip().lower() in {"1", "true", "t", "yes", "y", "on"}


class DefinitionsBuilder:

    def __init__(self):
        self.assets: List[dg.asset] = []
        self.jobs = []
        self.schedules = []
        self.resources = {}

    def init_telemetry(self):
        init_telemetry()

    def add_resources(
        self,
        resources: Union[dg.ConfigurableResource, Tuple[dg.ConfigurableResource, ...]],
    ):
        self.resources.update(resources)

    def add_asset_from_protoasset(self, protoasset: Protoasset) -> dg.asset:
        asset: dg.asset = lana_assetifier(protoasset=protoasset)
        self.assets.append(asset)

        return asset

    def add_job_from_assets(
        self, job_name: str, assets: Tuple[dg.asset, ...]
    ) -> dg.job:
        new_job = dg.define_asset_job(name=job_name, selection=assets)
        self.jobs.append(new_job)

        return new_job

    def add_job_schedule(self, job: dg.job, cron_expression: str):
        default_status = (
            dg.DefaultScheduleStatus.RUNNING
            if DAGSTER_AUTOMATIONS_ACTIVE
            else dg.DefaultScheduleStatus.STOPPED
        )
        new_job_schedule = dg.ScheduleDefinition(
            name=f"{job.name}_schedule",
            job=job,
            cron_schedule=cron_expression,
            default_status=default_status,
        )

        self.schedules.append(new_job_schedule)

        return new_job_schedule

    def build(self) -> dg.Definitions:
        return dg.Definitions(
            assets=self.assets,
            jobs=self.jobs,
            schedules=self.schedules,
            resources=self.resources,
        )


definition_builder = DefinitionsBuilder()

definition_builder.init_telemetry()
definition_builder.add_resources(get_project_resources())


definition_builder.add_asset_from_protoasset(
    Protoasset(key="iris_dataset_size", callable=iris_dataset_size)
)

bitfinex_protoassets = bitfinex_protoassets()
bitfinex_ticker_asset = definition_builder.add_asset_from_protoasset(
    bitfinex_protoassets["bitfinex_ticker"]
)
bitfinex_trades_asset = definition_builder.add_asset_from_protoasset(
    bitfinex_protoassets["bitfinex_trades"]
)
bitfinex_order_book_asset = definition_builder.add_asset_from_protoasset(
    bitfinex_protoassets["bitfinex_order_book"]
)

bitfinex_ticker_job = definition_builder.add_job_from_assets(
    job_name="bitfinex_ticker_el", assets=(bitfinex_ticker_asset,)
)
definition_builder.add_job_schedule(
    job=bitfinex_ticker_job, cron_expression="* * * * *"
)

bitfinex_trades_job = definition_builder.add_job_from_assets(
    job_name="bitfinex_trades_el", assets=(bitfinex_trades_asset,)
)
definition_builder.add_job_schedule(
    job=bitfinex_trades_job, cron_expression="*/10 * * * *"
)

bitfinex_order_book_job = definition_builder.add_job_from_assets(
    job_name="bitfinex_order_book_el", assets=(bitfinex_order_book_asset,)
)
definition_builder.add_job_schedule(
    job=bitfinex_order_book_job, cron_expression="*/10 * * * *"
)


for lana_source_protoasset in lana_source_protoassets():
    definition_builder.add_asset_from_protoasset(lana_source_protoasset)
for lana_to_dw_el_protoasset in lana_to_dw_el_protoassets():
    definition_builder.add_asset_from_protoasset(lana_to_dw_el_protoasset)

defs = definition_builder.build()
