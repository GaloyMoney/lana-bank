import dagster as dg

from lana_pipelines.assets import build_lana_source_asset, build_lana_to_dw_el_asset, build_dbt_assets, build_generate_es_report_asset
from lana_pipelines.resources import dbt_resource


def build_definitions():

    lana_to_dw_el_tables = (
        "core_deposit_account_events_rollup",
        "core_deposit_events_rollup",
        "core_withdrawal_events_rollup",
        "core_public_ids",
    )

    lana_source_assets = [
        build_lana_source_asset(table_name=table_name)
        for table_name in lana_to_dw_el_tables
    ]

    lana_to_dw_el_assets = [
        build_lana_to_dw_el_asset(table_name=table_name)
        for table_name in lana_to_dw_el_tables
    ]

    dbt_assets = [build_dbt_assets()]

    generate_es_report_asset = [build_generate_es_report_asset()]

    lana_to_dw_el_job = dg.define_asset_job(
        name="lana_to_dw_el_job",
        selection=lana_to_dw_el_assets,
        # Below is a silly hardcode to prevent running into rate limiting with
        # BQ: we should research how to avoid it because it gets triggered 
        # with rather low volumes.
        config={
            "execution": {
                "config": {
                    "multiprocess": {"max_concurrent": 4}, 
                }
            }
        },
    )

    build_dbt_job = dg.define_asset_job(
        name="build_dbt_job",
        selection=dbt_assets,
    )

    build_generate_es_report_job = dg.define_asset_job(
        name="generate_es_report_job",
        selection=generate_es_report_asset,
    )

    all_assets = lana_source_assets + lana_to_dw_el_assets + dbt_assets + generate_es_report_asset
    all_jobs = [lana_to_dw_el_job, build_dbt_job, build_generate_es_report_job]
    all_resources = {
        "dbt": dbt_resource
    }


    flana_to_dw_el_job_schedule = dg.ScheduleDefinition(
        name="lana_to_dw_el_job_schedule",
        cron_schedule="* * * * *",
        job=lana_to_dw_el_job,
        default_status=dg.DefaultScheduleStatus.RUNNING
    )

    build_dbt_job_schedule = dg.ScheduleDefinition(
        name="build_dbt_job_schedule",
        cron_schedule="*/2 * * * *",
        job=build_dbt_job,
        default_status=dg.DefaultScheduleStatus.RUNNING
    )

    build_generate_es_report_job_schedule = dg.ScheduleDefinition(
        name="build_generate_es_report_job_schedule",
        cron_schedule="*/3 * * * *",
        job=build_generate_es_report_job,
        default_status=dg.DefaultScheduleStatus.RUNNING
    )

    all_schedules = [
        flana_to_dw_el_job_schedule,
        build_dbt_job_schedule,
        build_generate_es_report_job_schedule
    ]

    return dg.Definitions(
        assets=all_assets,
        jobs=all_jobs, 
        resources=all_resources,
        schedules=all_schedules
    )


defs = build_definitions()
