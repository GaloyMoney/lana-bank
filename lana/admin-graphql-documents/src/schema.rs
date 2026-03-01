use async_graphql::{Context, Object};

use std::io::Read;

use admin_graphql_shared::primitives::*;

use super::*;

#[derive(Default)]
pub struct DocumentsQuery;

#[Object]
impl DocumentsQuery {
    async fn customer_document(
        &self,
        ctx: &Context<'_>,
        id: UUID,
    ) -> async_graphql::Result<Option<CustomerDocument>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(
            CustomerDocument,
            app.customers().find_customer_document_by_id(sub, id)
        )
    }
}

#[derive(Default)]
pub struct DocumentsMutation;

#[Object]
impl DocumentsMutation {
    async fn customer_document_attach(
        &self,
        ctx: &Context<'_>,
        input: CustomerDocumentCreateInput,
    ) -> async_graphql::Result<CustomerDocumentCreatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);

        let mut file = input.file.value(ctx)?;
        let mut data = Vec::new();
        file.content.read_to_end(&mut data)?;
        exec_mutation!(
            CustomerDocumentCreatePayload,
            CustomerDocument,
            app.customers().create_document(
                sub,
                input.customer_id,
                data,
                file.filename,
                file.content_type
                    .unwrap_or_else(|| "application/octet-stream".to_string()),
            )
        )
    }

    async fn customer_document_download_link_generate(
        &self,
        ctx: &Context<'_>,
        input: CustomerDocumentDownloadLinksGenerateInput,
    ) -> async_graphql::Result<CustomerDocumentDownloadLinksGeneratePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let doc = app
            .customers()
            .generate_document_download_link(
                sub,
                lana_app::customer::CustomerDocumentId::from(input.document_id),
            )
            .await?;

        Ok(CustomerDocumentDownloadLinksGeneratePayload::from(doc))
    }

    async fn customer_document_delete(
        &self,
        ctx: &Context<'_>,
        input: CustomerDocumentDeleteInput,
    ) -> async_graphql::Result<CustomerDocumentDeletePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        app.customers()
            .delete_document(sub, input.document_id)
            .await?;
        Ok(CustomerDocumentDeletePayload {
            deleted_document_id: input.document_id,
        })
    }

    async fn customer_document_archive(
        &self,
        ctx: &Context<'_>,
        input: CustomerDocumentArchiveInput,
    ) -> async_graphql::Result<CustomerDocumentArchivePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            CustomerDocumentArchivePayload,
            CustomerDocument,
            app.customers().archive_document(sub, input.document_id)
        )
    }
}
