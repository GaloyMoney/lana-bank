use async_graphql::{Context, Object, types::connection::*};

use super::*;

#[derive(Default)]
pub struct CustodyQuery;

#[Object]
impl CustodyQuery {
    async fn custodians(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<
        Connection<CustodiansByNameCursor, Custodian, EmptyFields, EmptyFields>,
    > {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        list_with_cursor!(CustodiansByNameCursor, Custodian, after, first, |query| app
            .custody()
            .list_custodians(sub, query))
    }
}

#[derive(Default)]
pub struct CustodyMutation;

#[Object]
impl CustodyMutation {
    async fn custodian_create(
        &self,
        ctx: &Context<'_>,
        input: CustodianCreateInput,
    ) -> async_graphql::Result<CustodianCreatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            CustodianCreatePayload,
            Custodian,
            app.custody()
                .create_custodian(sub, input.name().to_owned(), input.into())
        )
    }

    async fn custodian_config_update(
        &self,
        ctx: &Context<'_>,
        input: CustodianConfigUpdateInput,
    ) -> async_graphql::Result<CustodianConfigUpdatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            CustodianConfigUpdatePayload,
            Custodian,
            app.custody()
                .update_config(sub, input.custodian_id, input.config.into())
        )
    }
}
