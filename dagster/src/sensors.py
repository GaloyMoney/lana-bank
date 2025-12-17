from typing import Sequence

import dagster as dg
from src.assets.dbt import TAG_KEY_ASSET_TYPE, TAG_VALUE_DBT_MODEL


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


def build_dbt_automation_sensor(
    dagster_automations_active: bool,
) -> dg.AutomationConditionSensorDefinition:
    return dg.AutomationConditionSensorDefinition(
        name="dbt_automation_condition_sensor",
        target=dg.AssetSelection.tag(TAG_KEY_ASSET_TYPE, TAG_VALUE_DBT_MODEL),
        default_status=(
            dg.DefaultSensorStatus.RUNNING
            if dagster_automations_active
            else dg.DefaultSensorStatus.STOPPED
        ),
    )


def build_sumsub_sensor(
    sumsub_applicants_job: dg.JobDefinition,
    dagster_automations_active: bool,
) -> dg.SensorDefinition:
    def _trigger_sumsub_on_callbacks(context: dg.SensorEvaluationContext, _asset_event):
        yield dg.RunRequest(run_key=f"sumsub_{_asset_event.event_log_entry.storage_id}")

    return dg.AssetSensorDefinition(
        name="sumsub_applicants_callbacks_sensor",
        asset_key=dg.AssetKey(["lana", "sumsub_callbacks"]),
        job=sumsub_applicants_job,
        asset_materialization_fn=_trigger_sumsub_on_callbacks,
        default_status=(
            dg.DefaultSensorStatus.RUNNING
            if dagster_automations_active
            else dg.DefaultSensorStatus.STOPPED
        ),
    )
