import datetime

import dagster as dg

from lana_pipelines.assets import build_lana_source_asset, build_lana_to_dw_el_asset, build_dbt_assets, build_generate_es_report_asset
from lana_pipelines.resources import dbt_resource, poll_max_value_in_table_col


class DefinitionBuilder():
     
    def __init__(self, automation_style = "no_automation"):

        self.automation_style = automation_style

        self.asset_definitions = []
        self.job_definitions = []
        self.resource_definitions = []
        self.schedule_definitions = []
        self.sensor_definitions = []

        self.EL_TABLES = (
            "core_deposit_account_events_rollup",
            "core_deposit_events_rollup",
            "core_withdrawal_events_rollup",
            "core_public_ids",
        )
    
    def build_lana_source_layer(self):
        assets = [build_lana_source_asset(table_name=table_name) for table_name in self.EL_TABLES]
        self.asset_definitions.extend(assets)

    def build_lana_to_dw_el_layer(self):
        assets = [build_lana_to_dw_el_asset(table_name=table_name) for table_name in self.EL_TABLES]
        self.asset_definitions.extend(assets)
        
        lana_to_dw_el_job = dg.define_asset_job(
        name="lana_to_dw_el_job",
        selection=assets,
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

        self.job_definitions.append(lana_to_dw_el_job)

        if self.automation_style == "scheduled":
            lana_to_dw_el_job_schedule = dg.ScheduleDefinition(
                name="lana_to_dw_el_job_schedule",
                cron_schedule="* * * * *",
                job=lana_to_dw_el_job,
                default_status=dg.DefaultScheduleStatus.RUNNING
            )

            self.schedule_definitions.append(lana_to_dw_el_job_schedule)
        
        if self.automation_style == "mixed":
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

            self.sensor_definitions.append(lana_el_sensor)
         
    def build_dbt_layer(self):
        dbt_assets = [build_dbt_assets()]
        self.asset_definitions.extend(dbt_assets)

        build_dbt_job = dg.define_asset_job(
            name="build_dbt_job",
            selection=dbt_assets,
        )
        build_seed_bank_address_job = dg.define_asset_job(
        name="build_seed_bank_address_job",
        selection="seed_bank_address",
    )
        self.job_definitions.extend([build_dbt_job, build_seed_bank_address_job])

        if self.automation_style == "scheduled":
            build_dbt_job_schedule = dg.ScheduleDefinition(
                name="build_dbt_job_schedule",
                cron_schedule="*/2 * * * *",
                job=build_dbt_job,
                default_status=dg.DefaultScheduleStatus.RUNNING
            )

            build_seed_bank_address_job_schedule = dg.ScheduleDefinition(
                name="build_seed_bank_address_job_schedule",
                cron_schedule="*/2 * * * *",
                job=build_seed_bank_address_job,
                default_status=dg.DefaultScheduleStatus.RUNNING
            )

            self.schedule_definitions.extend([build_dbt_job_schedule, build_seed_bank_address_job_schedule])

        if self.automation_style == "mixed":
            build_seed_bank_address_job_schedule = dg.ScheduleDefinition(
                name="build_seed_bank_address_job_schedule",
                cron_schedule="* * * * *",
                job=build_seed_bank_address_job,
                default_status=dg.DefaultScheduleStatus.RUNNING
            )
            self.schedule_definitions.append(build_seed_bank_address_job_schedule)
             

def build_definitions():

    definition_builder = DefinitionBuilder(
        automation_style=dg.EnvVar("AUTOMATION_STYLE").get_value()
    )

    definition_builder.build_lana_source_layer()
    definition_builder.build_lana_to_dw_el_layer()
    definition_builder.build_dbt_layer()

    generate_es_report_asset = [build_generate_es_report_asset()]

    build_generate_es_report_job = dg.define_asset_job(
        name="generate_es_report_job",
        selection=generate_es_report_asset,
    )

    

    all_assets = definition_builder.asset_definitions + generate_es_report_asset
    all_jobs = definition_builder.job_definitions + [build_generate_es_report_job]
    all_resources = {
        "dbt": dbt_resource
    }

    all_schedules = []
    all_sensors = []

    if dg.EnvVar("AUTOMATION_STYLE").get_value() == "scheduled":

        build_generate_es_report_job_schedule = dg.ScheduleDefinition(
            name="build_generate_es_report_job_schedule",
            cron_schedule="*/3 * * * *",
            job=build_generate_es_report_job,
            default_status=dg.DefaultScheduleStatus.RUNNING
        )

        all_schedules = definition_builder.schedule_definitions + [
            build_generate_es_report_job_schedule
        ]

    if dg.EnvVar("AUTOMATION_STYLE").get_value() == "mixed":
        all_sensors = definition_builder.sensor_definitions
        
        build_generate_es_report_job_schedule = dg.ScheduleDefinition(
            name="build_generate_es_report_job_schedule",
            cron_schedule="*/3 * * * *",
            job=build_generate_es_report_job,
            default_status=dg.DefaultScheduleStatus.RUNNING
        )

        

        all_schedules = definition_builder.schedule_definitions + [build_generate_es_report_job_schedule]

    if dg.EnvVar("AUTOMATION_STYLE").get_value() == "no_automation":
         # We do nothing and let all_schedules and all_sensors go in empty
         pass

    return dg.Definitions(
        assets=all_assets,
        jobs=all_jobs, 
        resources=all_resources,
        schedules=all_schedules,
        sensors=all_sensors
    )


defs = build_definitions()
