use std::io::Read;

use async_graphql::{Context, Object, types::connection::*};

use admin_graphql_shared::primitives::UUID;
use lana_app::customer::prospect_cursor::ProspectsByCreatedAtCursor;

use crate::{customer::*, document::*, prospect::*, sumsub::*};

mutation_payload! { CustomerTelegramHandleUpdatePayload, customer: CustomerBase }
mutation_payload! { CustomerEmailUpdatePayload, customer: CustomerBase }
mutation_payload! { ProspectCreatePayload, prospect: ProspectBase }
mutation_payload! { ProspectClosePayload, prospect: ProspectBase }
mutation_payload! { ProspectConvertPayload, customer: CustomerBase }
mutation_payload! { CustomerDocumentCreatePayload, document: CustomerDocument }
mutation_payload! { CustomerDocumentArchivePayload, document: CustomerDocument }

#[derive(Default)]
pub struct CustomerQuery;

#[Object]
impl CustomerQuery {
    async fn customer(
        &self,
        ctx: &Context<'_>,
        id: UUID,
    ) -> async_graphql::Result<Option<CustomerBase>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(CustomerBase, app.customers().find_by_id(sub, id))
    }

    async fn customer_by_email(
        &self,
        ctx: &Context<'_>,
        email: String,
    ) -> async_graphql::Result<Option<CustomerBase>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(CustomerBase, app.customers().find_by_email(sub, email))
    }

    async fn customer_by_public_id(
        &self,
        ctx: &Context<'_>,
        id: PublicId,
    ) -> async_graphql::Result<Option<CustomerBase>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(CustomerBase, app.customers().find_by_public_id(sub, id))
    }

    async fn customers(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
        #[graphql(default_with = "Some(CustomersSort::default())")] sort: Option<CustomersSort>,
        filter: Option<CustomersFilter>,
    ) -> async_graphql::Result<Connection<CustomersCursor, CustomerBase, EmptyFields, EmptyFields>>
    {
        let filter = DomainCustomersFilters {
            kyc_verification: filter.and_then(|f| f.kyc_verification),
            ..Default::default()
        };

        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let sort = Sort {
            by: DomainCustomersSortBy::from(sort.unwrap_or_default()),
            direction: ListDirection::Descending,
        };
        list_with_combo_cursor!(
            CustomersCursor,
            CustomerBase,
            sort.by,
            after,
            first,
            |query| app.customers().list(sub, query, filter, sort)
        )
    }

    async fn prospect(
        &self,
        ctx: &Context<'_>,
        id: UUID,
    ) -> async_graphql::Result<Option<ProspectBase>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(ProspectBase, app.customers().find_prospect_by_id(sub, id))
    }

    async fn prospect_by_public_id(
        &self,
        ctx: &Context<'_>,
        id: PublicId,
    ) -> async_graphql::Result<Option<ProspectBase>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(
            ProspectBase,
            app.customers().find_prospect_by_public_id(sub, id)
        )
    }

    async fn prospects(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
        stage: Option<lana_app::customer::ProspectStage>,
    ) -> async_graphql::Result<
        Connection<ProspectsByCreatedAtCursor, ProspectBase, EmptyFields, EmptyFields>,
    > {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        list_with_cursor!(
            ProspectsByCreatedAtCursor,
            ProspectBase,
            after,
            first,
            |query| app
                .customers()
                .list_prospects(sub, query, ListDirection::Descending, stage,)
        )
    }

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
pub struct CustomerMutation;

#[Object]
impl CustomerMutation {
    pub async fn customer_document_attach(
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

    async fn prospect_create(
        &self,
        ctx: &Context<'_>,
        input: ProspectCreateInput,
    ) -> async_graphql::Result<ProspectCreatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            ProspectCreatePayload,
            ProspectBase,
            app.customers().create_prospect(
                sub,
                input.email,
                input.telegram_handle,
                input.customer_type
            )
        )
    }

    async fn prospect_close(
        &self,
        ctx: &Context<'_>,
        input: ProspectCloseInput,
    ) -> async_graphql::Result<ProspectClosePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            ProspectClosePayload,
            ProspectBase,
            app.customers().close_prospect(sub, input.prospect_id)
        )
    }

    async fn prospect_convert(
        &self,
        ctx: &Context<'_>,
        input: ProspectConvertInput,
    ) -> async_graphql::Result<ProspectConvertPayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let require_verified = app
            .exposed_domain_configs()
            .get::<lana_app::deposit::RequireVerifiedCustomerForAccount>(sub)
            .await?
            .value();
        if require_verified {
            return Err(async_graphql::Error::new(
                "Manual conversion is only available when 'Require verified customer for account' is disabled",
            ));
        }
        exec_mutation!(
            ProspectConvertPayload,
            CustomerBase,
            app.customers().convert_prospect(sub, input.prospect_id)
        )
    }

    async fn customer_telegram_handle_update(
        &self,
        ctx: &Context<'_>,
        input: CustomerTelegramHandleUpdateInput,
    ) -> async_graphql::Result<CustomerTelegramHandleUpdatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            CustomerTelegramHandleUpdatePayload,
            CustomerBase,
            app.customers()
                .update_telegram_handle(sub, input.customer_id, input.telegram_handle)
        )
    }

    async fn customer_email_update(
        &self,
        ctx: &Context<'_>,
        input: CustomerEmailUpdateInput,
    ) -> async_graphql::Result<CustomerEmailUpdatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            CustomerEmailUpdatePayload,
            CustomerBase,
            app.customers()
                .update_email(sub, input.customer_id, input.email)
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
            .generate_document_download_link(sub, input.document_id)
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

    pub async fn sumsub_permalink_create(
        &self,
        ctx: &Context<'_>,
        input: SumsubPermalinkCreateInput,
    ) -> async_graphql::Result<SumsubPermalinkCreatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let permalink = app
            .customer_kyc()
            .create_verification_link(
                sub,
                lana_app::primitives::ProspectId::from(input.prospect_id),
            )
            .await?;
        Ok(SumsubPermalinkCreatePayload { url: permalink.url })
    }

    /// ⚠️ TEST ONLY: Creates a complete test applicant for Sumsub integration testing.
    /// This method is behind a compilation flag and should only be used in test environments.
    #[cfg(feature = "sumsub-testing")]
    pub async fn sumsub_test_applicant_create(
        &self,
        ctx: &Context<'_>,
        input: SumsubTestApplicantCreateInput,
    ) -> async_graphql::Result<SumsubTestApplicantCreatePayload> {
        let (app, _sub) = app_and_sub_from_ctx!(ctx);
        let applicant_id = app
            .customer_kyc()
            .create_complete_test_applicant(lana_app::primitives::ProspectId::from(
                input.prospect_id,
            ))
            .await?;
        Ok(SumsubTestApplicantCreatePayload { applicant_id })
    }
}
