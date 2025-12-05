from typing import Sequence

import dagster as dg

from src.otel import JOB_TRACEPARENT_TAG


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
        # Pass the parent job's traceparent to continue the same trace
        parent_tags = dict(context.dagster_run.tags or {})
        tags = {}
        if traceparent := parent_tags.get(JOB_TRACEPARENT_TAG):
            tags[JOB_TRACEPARENT_TAG] = traceparent
        
        yield dg.RunRequest(
            run_key=f"inform_lana_success_{context.dagster_run.run_id}",
            tags=tags
        )

    @dg.run_status_sensor(
        run_status=dg.DagsterRunStatus.FAILURE,
        request_job=inform_lana_job,
        monitored_jobs=monitored_jobs,
        monitor_all_code_locations=False,
        default_status=default_status,
    )
    def file_reports_failure_sensor(context: dg.RunStatusSensorContext):
        # Pass the parent job's traceparent to continue the same trace
        parent_tags = dict(context.dagster_run.tags or {})
        tags = {}
        if traceparent := parent_tags.get(JOB_TRACEPARENT_TAG):
            tags[JOB_TRACEPARENT_TAG] = traceparent
        
        yield dg.RunRequest(
            run_key=f"inform_lana_failure_{context.dagster_run.run_id}",
            tags=tags
        )

    return file_reports_success_sensor, file_reports_failure_sensor
