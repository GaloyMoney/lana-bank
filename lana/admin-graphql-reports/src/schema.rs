use admin_graphql_shared::primitives::UUID;
use async_graphql::{Context, Object, types::connection::*};

use super::*;

#[derive(Default)]
pub struct ReportsQuery;

#[Object]
impl ReportsQuery {
    async fn report_runs(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<
        Connection<ReportRunsByCreatedAtCursor, ReportRun, EmptyFields, EmptyFields>,
    > {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        list_with_cursor!(
            ReportRunsByCreatedAtCursor,
            ReportRun,
            after,
            first,
            |query| app.reports().list_report_runs(sub, query)
        )
    }

    async fn report_run(
        &self,
        ctx: &Context<'_>,
        id: UUID,
    ) -> async_graphql::Result<Option<ReportRun>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(ReportRun, app.reports().find_report_run_by_id(sub, id))
    }
}

#[derive(Default)]
pub struct ReportsMutation;

#[Object]
impl ReportsMutation {
    async fn trigger_report_run(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<ReportRunCreatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let _job_id = app.reports().trigger_report_run_job(sub).await?;
        Ok(ReportRunCreatePayload { run_id: None })
    }

    async fn report_file_generate_download_link(
        &self,
        ctx: &Context<'_>,
        input: ReportFileGenerateDownloadLinkInput,
    ) -> async_graphql::Result<ReportFileGenerateDownloadLinkPayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let url = app
            .reports()
            .generate_report_file_download_link(sub, input.report_id, input.extension)
            .await?;
        Ok(ReportFileGenerateDownloadLinkPayload { url })
    }
}
