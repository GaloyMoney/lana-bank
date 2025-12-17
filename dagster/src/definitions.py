"""Dagster definitions entry point - builds all Dagster objects."""

import os
from typing import List, Tuple, Union

import dagster as dg
from src.assets import (
    bitfinex_protoassets,
    file_report_protoassets,
    inform_lana_protoasset,
    iris_dataset_size,
    lana_dbt_protoassets,
    lana_source_protoassets,
    lana_to_dw_el_protoassets,
    sumsub_protoasset,
)
from src.core import Protoasset, lana_assetifier
from src.otel import init_telemetry
from src.resources import get_project_resources
from src.sensors import build_dbt_automation_sensor, build_file_report_sensors

DAGSTER_AUTOMATIONS_ACTIVE = os.getenv(
    "DAGSTER_AUTOMATIONS_ACTIVE", ""
).strip().lower() in {"1", "true", "t", "yes", "y", "on"}


class DefinitionsBuilder:

    def __init__(self):
        self.assets: List[dg.asset] = []
        self.jobs = []
        self.schedules = []
        self.sensors = []
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

    def add_sensor(self, sensor: dg.SensorDefinition):
        self.sensors.append(sensor)
        return sensor

    def build(self) -> dg.Definitions:
        return dg.Definitions(
            assets=self.assets,
            jobs=self.jobs,
            schedules=self.schedules,
            sensors=self.sensors,
            resources=self.resources,
        )


definition_builder = DefinitionsBuilder()

definition_builder.init_telemetry()
definition_builder.add_resources(get_project_resources())


definition_builder.add_asset_from_protoasset(
    Protoasset(key=dg.AssetKey("iris_dataset_size"), callable=iris_dataset_size)
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
    job=bitfinex_ticker_job, cron_expression="*/10 * * * *"
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


# Sumsub applicants EL (sensor-triggered on lana/sumsub_callbacks)
sumsub_applicants_protoasset = sumsub_protoasset()
sumsub_applicants_asset = definition_builder.add_asset_from_protoasset(
    sumsub_applicants_protoasset
)

sumsub_applicants_job = definition_builder.add_job_from_assets(
    job_name="sumsub_applicants_el",
    assets=(sumsub_applicants_asset,),
)

definition_builder.add_sensor(
    dg.AssetSensorDefinition(
        name="run_sumsub_applicants_el_on_callbacks",
        asset_key=dg.AssetKey(["lana", "sumsub_callbacks"]),
        job=sumsub_applicants_job,
        default_status=(
            dg.DefaultSensorStatus.RUNNING
            if DAGSTER_AUTOMATIONS_ACTIVE
            else dg.DefaultSensorStatus.STOPPED
        ),
    )
)


for lana_source_protoasset in lana_source_protoassets():
    definition_builder.add_asset_from_protoasset(lana_source_protoasset)

lana_el_protoassets = lana_to_dw_el_protoassets()

lana_to_dw_el_assets = []
for lana_to_dw_el_protoasset in lana_el_protoassets:
    lana_to_dw_el_asset = definition_builder.add_asset_from_protoasset(
        lana_to_dw_el_protoasset
    )
    lana_to_dw_el_assets.append(lana_to_dw_el_asset)

lana_to_dw_el_job = definition_builder.add_job_from_assets(
    job_name="lana_to_dw_el",
    assets=tuple(lana_to_dw_el_assets),
)
definition_builder.add_job_schedule(job=lana_to_dw_el_job, cron_expression="0 0 * * *")

for dbt_protoasset in lana_dbt_protoassets(source_protoassets=lana_el_protoassets):
    definition_builder.add_asset_from_protoasset(dbt_protoasset)

dbt_automation_sensor = build_dbt_automation_sensor(
    dagster_automations_active=DAGSTER_AUTOMATIONS_ACTIVE
)
definition_builder.add_sensor(dbt_automation_sensor)

report_protoassets = file_report_protoassets()

report_generation_assets = [
    definition_builder.add_asset_from_protoasset(protoasset)
    for protoasset in report_protoassets.values()
]
file_reports_job = definition_builder.add_job_from_assets(
    job_name="file_reports_generation", assets=tuple(report_generation_assets)
)
definition_builder.add_job_schedule(job=file_reports_job, cron_expression="0 */2 * * *")

inform_lana_asset = definition_builder.add_asset_from_protoasset(
    inform_lana_protoasset()
)
inform_lana_job = definition_builder.add_job_from_assets(
    job_name="notify_lana_job", assets=(inform_lana_asset,)
)

file_reports_success_sensor, file_reports_failure_sensor = build_file_report_sensors(
    inform_lana_job=inform_lana_job,
    monitored_jobs=[file_reports_job],
    dagster_automations_active=DAGSTER_AUTOMATIONS_ACTIVE,
)

definition_builder.add_sensor(file_reports_success_sensor)
definition_builder.add_sensor(file_reports_failure_sensor)

defs = definition_builder.build()
