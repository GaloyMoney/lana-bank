use std::collections::HashMap;

use crate::{
    ledger::{error::LedgerError, Ledger},
    server::shared_graphql::objects::PaginationKey,
};

use async_graphql::dataloader::Loader;

use super::account_set::ChartOfAccountsCategoryAccount;

pub struct ChartOfAccountsLoader {
    pub cala: Ledger,
}

impl Loader<PaginationKey> for ChartOfAccountsLoader {
    type Value = Vec<ChartOfAccountsCategoryAccount>;
    type Error = LedgerError;

    async fn load(
        &self,
        keys: &[PaginationKey],
    ) -> Result<HashMap<PaginationKey, Vec<ChartOfAccountsCategoryAccount>>, Self::Error> {
        let PaginationKey {
            key: _,
            first,
            after,
        } = &keys[0];

        let result = if let Some(chart_of_accounts) = self
            .cala
            .chart_of_accounts_paginated(i64::from(*first), after.clone())
            .await?
        {
            chart_of_accounts
                .categories
                .into_iter()
                .map(|category| {
                    (
                        PaginationKey {
                            key: category.id.into(),
                            first: *first,
                            after: after.clone(),
                        },
                        category
                            .category_accounts
                            .iter()
                            .map(|account| ChartOfAccountsCategoryAccount::from(account.clone()))
                            .collect(),
                    )
                })
                .collect()
        } else {
            HashMap::new()
        };

        Ok(result)
    }
}
