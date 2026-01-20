from typing import Sequence

import dagster as dg
from src.assets.dbt import TAG_KEY_ASSET_TYPE, TAG_VALUE_DBT_MODEL
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
            run_key=f"inform_lana_success_{context.dagster_run.run_id}", tags=tags
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
            run_key=f"inform_lana_failure_{context.dagster_run.run_id}", tags=tags
        )

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
    def _trigger_sumsub_on_inbox_events(
        _context: dg.SensorEvaluationContext, asset_event
    ):
        dagster_event = getattr(asset_event, "dagster_event", None)
        event_id = getattr(dagster_event, "event_log_entry_id", None) or getattr(
            asset_event, "run_id", None
        )

        yield dg.RunRequest(run_key=f"sumsub_applicants_from_inbox_events_{event_id}")

    return dg.AssetSensorDefinition(
        name="sumsub_applicant_inbox_events_sensor",
        asset_key=dg.AssetKey(["lana", "inbox_events"]),
        job_name=sumsub_applicants_job.name,
        asset_materialization_fn=_trigger_sumsub_on_inbox_events,
        default_status=(
            dg.DefaultSensorStatus.RUNNING
            if dagster_automations_active
            else dg.DefaultSensorStatus.STOPPED
        ),
    )
