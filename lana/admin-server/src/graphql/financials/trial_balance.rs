use async_graphql::{connection::*, *};

use crate::{
    graphql::{account::AccountAmountsByCurrency, accounting::AccountCode},
    primitives::*,
};

use lana_app::trial_balance::TrialBalanceAccountCursor;

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct TrialBalance {
    name: String,
    total: AccountAmountsByCurrency,

    #[graphql(skip)]
    from: Timestamp,
    #[graphql(skip)]
    until: Timestamp,
}

#[ComplexObject]
impl TrialBalance {
    pub async fn accounts(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<
        Connection<TrialBalanceAccountCursor, TrialBalanceAccount, EmptyFields, EmptyFields>,
    > {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);

        query(
            after,
            None,
            Some(first),
            None,
            |after, _, first, _| async move {
                let first = first.expect("First always exists");
                let query_args = es_entity::PaginatedQueryArgs { first, after };
                let res = app
                    .accounting()
                    .trial_balances()
                    .trial_balance_accounts(
                        sub,
                        self.name.to_string(),
                        self.from.into_inner(),
                        Some(self.until.into_inner()),
                        query_args,
                    )
                    .await?;

                let mut connection = Connection::new(false, res.has_next_page);
                connection
                    .edges
                    .extend(res.entities.into_iter().map(|entry| {
                        let cursor = TrialBalanceAccountCursor::from(&entry);
                        Edge::new(cursor, TrialBalanceAccount::from(entry))
                    }));
                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
    }
}
#[derive(SimpleObject)]
pub struct TrialBalanceAccount {
    id: UUID,
    name: String,
    amounts: AccountAmountsByCurrency,
    code: AccountCode,
}

impl From<lana_app::trial_balance::TrialBalanceAccount> for TrialBalanceAccount {
    fn from(line_item: lana_app::trial_balance::TrialBalanceAccount) -> Self {
        TrialBalanceAccount {
            id: line_item.id.into(),
            name: line_item.name.to_string(),
            code: AccountCode::from(&line_item.code),
            amounts: line_item.into(),
        }
    }
}

impl From<lana_app::trial_balance::TrialBalanceRoot> for TrialBalance {
    fn from(trial_balance: lana_app::trial_balance::TrialBalanceRoot) -> Self {
        TrialBalance {
            name: trial_balance.name.to_string(),
            total: trial_balance.clone().into(),
            from: trial_balance.from.into(),
            until: trial_balance
                .until
                .expect("Mandatory 'until' value missing")
                .into(),
        }
    }
}
