use async_graphql::{Context, Error, Object, types::connection::*};

use super::*;
use lana_app::customer::prospect_cursor::ProspectsByCreatedAtCursor;

#[derive(Default)]
pub struct CustomerQuery;

#[Object]
impl CustomerQuery {
    async fn customer(
        &self,
        ctx: &Context<'_>,
        id: UUID,
    ) -> async_graphql::Result<Option<Customer>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(Customer, ctx, app.customers().find_by_id(sub, id))
    }

    async fn customer_by_email(
        &self,
        ctx: &Context<'_>,
        email: String,
    ) -> async_graphql::Result<Option<Customer>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(Customer, ctx, app.customers().find_by_email(sub, email))
    }

    async fn customer_by_public_id(
        &self,
        ctx: &Context<'_>,
        id: PublicId,
    ) -> async_graphql::Result<Option<Customer>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(Customer, ctx, app.customers().find_by_public_id(sub, id))
    }

    async fn customers(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
        #[graphql(default_with = "Some(CustomersSort::default())")] sort: Option<CustomersSort>,
        filter: Option<CustomersFilter>,
    ) -> async_graphql::Result<Connection<CustomersCursor, Customer, EmptyFields, EmptyFields>>
    {
        let filter = DomainCustomersFilters {
            kyc_verification: filter.and_then(|f| f.kyc_verification),
            ..Default::default()
        };

        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let sort = Sort {
            by: DomainCustomersSortBy::from(sort.unwrap_or_default()),
            direction: es_entity::ListDirection::Descending,
        };
        list_with_combo_cursor!(
            CustomersCursor,
            Customer,
            sort.by,
            ctx,
            after,
            first,
            |query| app.customers().list(sub, query, filter, sort)
        )
    }

    async fn prospect(
        &self,
        ctx: &Context<'_>,
        id: UUID,
    ) -> async_graphql::Result<Option<Prospect>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(Prospect, ctx, app.customers().find_prospect_by_id(sub, id))
    }

    async fn prospect_by_public_id(
        &self,
        ctx: &Context<'_>,
        id: PublicId,
    ) -> async_graphql::Result<Option<Prospect>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(
            Prospect,
            ctx,
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
        Connection<ProspectsByCreatedAtCursor, Prospect, EmptyFields, EmptyFields>,
    > {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        list_with_cursor!(
            ProspectsByCreatedAtCursor,
            Prospect,
            ctx,
            after,
            first,
            |query| app.customers().list_prospects(
                sub,
                query,
                es_entity::ListDirection::Descending,
                stage,
            )
        )
    }
}

#[derive(Default)]
pub struct CustomerMutation;

#[Object]
impl CustomerMutation {
    async fn prospect_create(
        &self,
        ctx: &Context<'_>,
        input: ProspectCreateInput,
    ) -> async_graphql::Result<ProspectCreatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            ProspectCreatePayload,
            Prospect,
            ctx,
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
            Prospect,
            ctx,
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
            return Err(Error::new(
                "Manual conversion is only available when 'Require verified customer for account' is disabled",
            ));
        }
        exec_mutation!(
            ProspectConvertPayload,
            Customer,
            ctx,
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
            Customer,
            ctx,
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
            Customer,
            ctx,
            app.customers()
                .update_email(sub, input.customer_id, input.email)
        )
    }
}
