use async_graphql::{Context, Object, types::connection::*};

use super::{credit_config::*, deposit_config::*, domain_config::*};

#[derive(Default)]
pub struct ConfigQuery;

#[Object]
impl ConfigQuery {
    async fn deposit_config(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<DepositModuleConfig>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let config = app
            .deposits()
            .chart_of_accounts_integrations()
            .get_config(sub)
            .await?;
        Ok(config.map(DepositModuleConfig::from))
    }

    async fn domain_configs(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<
        Connection<DomainConfigsByKeyCursor, DomainConfig, EmptyFields, EmptyFields>,
    > {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        list_with_cursor!(
            DomainConfigsByKeyCursor,
            DomainConfig,
            after,
            first,
            |query| app.exposed_domain_configs().list(sub, query)
        )
    }

    async fn credit_config(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<CreditModuleConfig>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let config = app
            .credit()
            .chart_of_accounts_integrations()
            .get_config(sub)
            .await?;
        Ok(config.map(CreditModuleConfig::from))
    }
}

#[derive(Default)]
pub struct ConfigMutation;

#[Object]
impl ConfigMutation {
    async fn domain_config_update(
        &self,
        ctx: &Context<'_>,
        input: DomainConfigUpdateInput,
    ) -> async_graphql::Result<DomainConfigUpdatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            DomainConfigUpdatePayload,
            DomainConfig,
            app.exposed_domain_configs().update_from_json(
                sub,
                input.domain_config_id,
                input.value.into_inner(),
            )
        )
    }
}
