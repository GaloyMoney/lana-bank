from typing import Sequence

import dagster as dg
from src.otel import JOB_TRACEPARENT_TAG, PARENT_JOB_TRACEPARENT_TAG


def build_file_report_sensors(
    inform_lana_job: dg.JobDefinition,
    monitored_jobs: Sequence[dg.JobDefinition],
    dagster_automations_active: bool,
):
    default_status = (
        dg.DefaultSensorStatus.RUNNING
        if dagster_automations_active
        else dg.DefaultSensorStatus.STOPPED
    )

    @dg.run_status_sensor(
        run_status=dg.DagsterRunStatus.SUCCESS,
        request_job=inform_lana_job,
        monitored_jobs=monitored_jobs,
        monitor_all_code_locations=False,
        default_status=default_status,
    )
    def file_reports_success_sensor(context: dg.RunStatusSensorContext):
        # Extract the traceparent from the monitored job run
        tags = dict(context.dagster_run.tags or {})
        parent_traceparent = tags.get(JOB_TRACEPARENT_TAG)

        # Pass the parent traceparent as a tag to the inform_lana_job
        run_tags = {}
        if parent_traceparent:
            run_tags[PARENT_JOB_TRACEPARENT_TAG] = parent_traceparent

        yield dg.RunRequest(
            run_key=f"inform_lana_success_{context.dagster_run.run_id}", tags=run_tags
        )

    @dg.run_status_sensor(
        run_status=dg.DagsterRunStatus.FAILURE,
        request_job=inform_lana_job,
        monitored_jobs=monitored_jobs,
        monitor_all_code_locations=False,
        default_status=default_status,
    )
    def file_reports_failure_sensor(context: dg.RunStatusSensorContext):
        # Extract the traceparent from the monitored job run
        tags = dict(context.dagster_run.tags or {})
        parent_traceparent = tags.get(JOB_TRACEPARENT_TAG)

        # Pass the parent traceparent as a tag to the inform_lana_job
        run_tags = {}
        if parent_traceparent:
            run_tags[PARENT_JOB_TRACEPARENT_TAG] = parent_traceparent

        yield dg.RunRequest(
            run_key=f"inform_lana_failure_{context.dagster_run.run_id}", tags=run_tags
        )

    return file_reports_success_sensor, file_reports_failure_sensor
