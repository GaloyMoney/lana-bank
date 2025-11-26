from typing import Sequence

import dagster as dg


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
        yield dg.RunRequest(run_key=f"inform_lana_success_{context.dagster_run.run_id}")

    @dg.run_status_sensor(
        run_status=dg.DagsterRunStatus.FAILURE,
        request_job=inform_lana_job,
        monitored_jobs=monitored_jobs,
        monitor_all_code_locations=False,
        default_status=default_status,
    )
    def file_reports_failure_sensor(context: dg.RunStatusSensorContext):
        yield dg.RunRequest(run_key=f"inform_lana_failure_{context.dagster_run.run_id}")

    return file_reports_success_sensor, file_reports_failure_sensor
