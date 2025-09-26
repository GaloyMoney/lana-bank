import datetime

import dagster as dg

from lana_pipelines.assets import build_lana_source_asset, build_lana_to_dw_el_asset, build_dbt_assets, build_generate_es_report_asset
from lana_pipelines.resources import dbt_resource, poll_max_value_in_table_col


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

    all_schedules = []
    all_sensors = []

    if dg.EnvVar("AUTOMATION_STYLE").get_value() == "scheduled":

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

    if dg.EnvVar("AUTOMATION_STYLE").get_value() == "mixed":

        @dg.sensor(
            job=lana_to_dw_el_job,
            minimum_interval_seconds=10,
            default_status=dg.DefaultSensorStatus.RUNNING,
        )
        def lana_el_sensor(context):
            connection_string = "postgresql://user:password@172.17.0.1:5433/pg"

            current_highest_timestamp = poll_max_value_in_table_col(
                connection_string_details=connection_string,
                table_name="core_deposit_events_rollup",
                fieldname="created_at",
                
            )

            highest_seen_timestamp = datetime.datetime.fromisoformat(context.cursor) if context.cursor else datetime.datetime(1970, 1, 1, 0, 0, 0, tzinfo=datetime.timezone.utc)

            context.log.info(f"Highest timestamp seen in Dagster: {highest_seen_timestamp}.")
            context.log.info(f"Current highest timestamp in source: {current_highest_timestamp}.")

            if current_highest_timestamp > highest_seen_timestamp:
                    yield dg.RunRequest(run_key=str(current_highest_timestamp))
            else:
                yield dg.SkipReason("No new files found")
            
            context.update_cursor(current_highest_timestamp.isoformat())

        all_sensors = [lana_el_sensor]
        

        build_generate_es_report_job_schedule = dg.ScheduleDefinition(
            name="build_generate_es_report_job_schedule",
            cron_schedule="*/3 * * * *",
            job=build_generate_es_report_job,
            default_status=dg.DefaultScheduleStatus.RUNNING
        )

        all_schedules = [build_generate_es_report_job_schedule]


    return dg.Definitions(
        assets=all_assets,
        jobs=all_jobs, 
        resources=all_resources,
        schedules=all_schedules,
        sensors=all_sensors
    )


defs = build_definitions()
