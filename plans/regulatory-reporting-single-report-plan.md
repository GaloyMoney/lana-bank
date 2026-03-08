# Regulatory Reporting Single-Report Plan

## Verified locally

- `file_reports_generation` accepts `assetSelection` for a single logical report.
- `file_reports_generation` accepts `runConfigData.ops.file_report_assets.config.as_of_date`.
- Runs launched that way still appear under the existing `pipelineName: "file_reports_generation"` filter.
- `ExecutionParams.executionMetadata.tags` is available in Dagster GraphQL.
- Custom launch tags come back through `runsOrError.results[].tags`.
- A single-report run still emits one materialization per file format, so the existing norm/name aggregation model remains valid.

## Phase 1

### Backend

1. Add a Rust-side report definition catalog sourced from `dagster/generate_es_reports/reports.yml`.
2. Change manual Dagster launches to use:
   - `jobName: "file_reports_generation"`
   - `assetSelection` for the selected report outputs
   - `runConfigData` only when `as_of_date` is provided
   - `executionMetadata.tags` for Lana request metadata
3. Extend `ReportRun` to store requested report metadata and optional `as_of_date`.
4. Parse those values from Dagster run tags during sync.
5. Expose:
   - `availableReportDefinitions`
   - `triggerReportRun(input: ...)`
   - requested report metadata on `ReportRun`

### Admin panel

1. Replace the current no-input manual trigger with a definition-driven flow.
2. Show available report definitions on the regulatory reporting page.
3. Require an `as_of_date` input when `supports_as_of` is true.
4. Keep the existing run history, but add requested-report metadata to it.

### Notes

- Keep using `file_reports_generation`; no `__ASSET_JOB` fallback is needed.
- Keep the current report aggregation model; no `Report` shape changes are required for Phase 1.
- Keep the current async "appears shortly" UX for now. Immediate redirect can be a later phase.
