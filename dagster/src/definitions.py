"""Dagster definitions entry point - builds all Dagster objects."""

import os
from typing import Callable, Tuple, Union

import dagster as dg
from src.assets import (
    bitfinex_order_book,
    bitfinex_ticker,
    bitfinex_trades,
    iris_dataset_size,
    lana_resources,
    lana_source_assets,
    lana_to_dw_el_assets,

)
from src.core import lana_assetifier
from src.otel import init_telemetry

DAGSTER_AUTOMATIONS_ACTIVE = os.getenv(
    "DAGSTER_AUTOMATIONS_ACTIVE", ""
).strip().lower() in {"1", "true", "t", "yes", "y", "on"}


class DefinitionsBuilder:

    def __init__(self):
        self.assets = []
        self.jobs = []
        self.schedules = []
        self.resources = {}

    def init_telemetry(self):
        init_telemetry()

    def add_assets(self, assets: Union[dg.asset, Tuple[dg.asset, ...]]):
        self.assets.extend(assets)

    def add_resources(self, resources: Union[dg.ConfigurableResource, Tuple[dg.ConfigurableResource, ...]]):
        self.resources.update(resources)

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
            assets=self.assets, jobs=self.jobs, schedules=self.schedules, resources=self.resources
        )


definition_builder = DefinitionsBuilder()

definition_builder.init_telemetry()
definition_builder.add_callable_as_asset(iris_dataset_size)

bitfinex_ticker_asset = definition_builder.add_callable_as_asset(bitfinex_ticker)
bitfinex_trades_asset = definition_builder.add_callable_as_asset(bitfinex_trades)
bitfinex_order_book_asset = definition_builder.add_callable_as_asset(
    bitfinex_order_book
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

definition_builder.add_resources(lana_resources())
definition_builder.add_assets(lana_source_assets())
definition_builder.add_assets(lana_to_dw_el_assets())

defs = definition_builder.build()
