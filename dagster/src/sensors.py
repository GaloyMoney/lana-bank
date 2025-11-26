import dagster as dg
from typing import Sequence


def build_file_report_sensors(
    inform_lana_job: dg.JobDefinition,
    monitored_jobs: Sequence[dg.JobDefinition],
    dagster_automations_active: bool,
):
    # Get location name from environment variable, fallback to "Lana DW" for backward compatibility
    location_name = dg.EnvVar("DAGSTER_CODE_LOCATION_NAME").get_value() or "Lana DW"
    default_status = (
        dg.DefaultSensorStatus.RUNNING
        if dagster_automations_active
        else dg.DefaultSensorStatus.STOPPED
    )

    job_selectors = [
        dg.JobSelector(job_name=job.name, location_name=location_name)
        for job in monitored_jobs
    ]

    @dg.run_status_sensor(
        run_status=dg.DagsterRunStatus.SUCCESS,
        request_job=inform_lana_job,
        monitored_jobs=job_selectors,
        monitor_all_code_locations=False,
        default_status=default_status,
    )
    def file_reports_success_sensor(context: dg.RunStatusSensorContext):
        yield dg.RunRequest(run_key=f"inform_lana_success_{context.dagster_run.run_id}")

    @dg.run_status_sensor(
        run_status=dg.DagsterRunStatus.FAILURE,
        request_job=inform_lana_job,
        monitored_jobs=job_selectors,
        monitor_all_code_locations=False,
        default_status=default_status,
    )
    def file_reports_failure_sensor(context: dg.RunStatusSensorContext):
        yield dg.RunRequest(run_key=f"inform_lana_failure_{context.dagster_run.run_id}")

    return file_reports_success_sensor, file_reports_failure_sensor
