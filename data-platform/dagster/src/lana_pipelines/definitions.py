import datetime

import dagster as dg

from lana_pipelines.assets import build_all_lana_source_assets, build_all_lana_to_dw_el_assets, build_dbt_assets, build_generate_es_report_asset
from lana_pipelines.resources import BigQueryResource, PostgresResource, dbt_resource
from lana_pipelines import constants

class DefinitionBuilder():
     
    def __init__(self, automation_style = "no_automation"):

        self.automation_style = automation_style

        self.asset_definitions = []
        self.job_definitions = []
        self.resource_definitions = {}
        self.schedule_definitions = []
        self.sensor_definitions = []

        self.EL_TABLES = constants.LANA_EL_TABLE_NAMES

    def build_resources(self):
        self.resource_definitions["lana_core_pg"] = PostgresResource()
        self.resource_definitions["dw_bq"] = BigQueryResource(base64_credentials=dg.EnvVar("TF_VAR_sa_creds").get_value())
        self.resource_definitions["dbt"] = dbt_resource
    
    def build_lana_source_layer(self):
        assets = build_all_lana_source_assets(
            table_names=constants.LANA_EL_TABLE_NAMES
        )
        self.asset_definitions.extend(assets)

    def build_lana_to_dw_el_layer(self):
        assets = build_all_lana_to_dw_el_assets()
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
            def lana_el_sensor(context, lana_core_pg: PostgresResource):
                current_highest_timestamp = lana_core_pg.poll_max_value_in_table_col(
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
             
    def build_generate_es_reports_layer(self):
        generate_es_report_asset = [build_generate_es_report_asset()]
        self.asset_definitions.extend(generate_es_report_asset)

        build_generate_es_report_job = dg.define_asset_job(
            name="generate_es_report_job",
            selection=generate_es_report_asset,
        )
        self.job_definitions.append(build_generate_es_report_job)

        if self.automation_style == "scheduled":
            build_generate_es_report_job_schedule = dg.ScheduleDefinition(
                name="build_generate_es_report_job_schedule",
                cron_schedule="*/3 * * * *",
                job=build_generate_es_report_job,
                default_status=dg.DefaultScheduleStatus.RUNNING
            )
            self.schedule_definitions.append(build_generate_es_report_job_schedule)

        if self.automation_style == "mixed":
            build_generate_es_report_job_schedule = dg.ScheduleDefinition(
                name="build_generate_es_report_job_schedule",
                cron_schedule="*/3 * * * *",
                job=build_generate_es_report_job,
                default_status=dg.DefaultScheduleStatus.RUNNING
            )
            self.schedule_definitions.append(build_generate_es_report_job_schedule)


def build_definitions():

    definition_builder = DefinitionBuilder(
        automation_style=dg.EnvVar("AUTOMATION_STYLE").get_value()
    )

    definition_builder.build_resources()
    definition_builder.build_lana_source_layer()
    definition_builder.build_lana_to_dw_el_layer()
    definition_builder.build_dbt_layer()
    definition_builder.build_generate_es_reports_layer()

    return dg.Definitions(
        assets=definition_builder.asset_definitions,
        jobs=definition_builder.job_definitions, 
        resources=definition_builder.resource_definitions,
        schedules=definition_builder.schedule_definitions,
        sensors=definition_builder.sensor_definitions
    )


defs = build_definitions()
