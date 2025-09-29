import datetime

import dagster as dg

from lana_pipelines.resources import PostgresResource

def build_lana_el_sensor(lana_to_dw_el_job):
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

        highest_seen_timestamp = (
            datetime.datetime.fromisoformat(context.cursor)
            if context.cursor
            else datetime.datetime(
                1970, 1, 1, 0, 0, 0, tzinfo=datetime.timezone.utc
            )
        )

        context.log.info(
            f"Highest timestamp seen in Dagster: {highest_seen_timestamp}."
        )
        context.log.info(
            f"Current highest timestamp in source: {current_highest_timestamp}."
        )

        if current_highest_timestamp > highest_seen_timestamp:
            yield dg.RunRequest(run_key=str(current_highest_timestamp))
        else:
            yield dg.SkipReason("No new files found")

        context.update_cursor(current_highest_timestamp.isoformat())

    return lana_el_sensor
    