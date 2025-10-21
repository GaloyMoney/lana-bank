use chrono::{DateTime, Utc};
use es_entity::*;
use sqlx::PgPool;

use crate::primitives::{AccountingPeriodId, ChartId};

use super::{
    entity::{AccountingPeriod, AccountingPeriodEvent},
    error::AccountingPeriodError,
};

#[derive(EsRepo)]
#[es_repo(
    entity = "AccountingPeriod",
    err = "AccountingPeriodError",
    columns(
        chart_id(ty = "ChartId", update(persist = false), parent),
        closed_at(ty = "Option<DateTime<Utc>>", create(persist = false)),
    ),
    tbl_prefix = "core"
)]
pub struct AccountingPeriodRepo {
    pool: PgPool,
}

impl AccountingPeriodRepo {
    /// Returns a list of all Accounting Periods that are currently
    /// open on the given chart. No specific order of the periods is
    /// guaranteed.
    pub async fn find_open_accounting_periods(
        &self,
        chart_id: ChartId,
    ) -> Result<Vec<AccountingPeriod>, AccountingPeriodError> {
        let (result, _) = es_query!(
            tbl_prefix = "core",
            r#"
               SELECT ap.id FROM core_accounting_periods ap
               WHERE ap.chart_id = $1 AND ap.closed_at IS NULL"#,
            chart_id as ChartId
        )
        .fetch_n(&mut self.pool.begin().await?, 100)
        .await?;

        Ok(result)
    }
}

impl Clone for AccountingPeriodRepo {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
        }
    }
}

impl AccountingPeriodRepo {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }
}
