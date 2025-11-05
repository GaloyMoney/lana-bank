use sqlx::PgPool;

use es_entity::*;

use crate::primitives::{AccountingCalendarId, ChartId};

use super::{entity::*, error::AccountingCalendarError};

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "AccountingCalendar",
    err = "AccountingCalendarError",
    columns(
        reference(ty = "String", create(accessor = "reference()")),
        chart_id(ty = "ChartId", list_for),
        is_open(ty = "bool", create(persist = false)),
    ),
    tbl_prefix = "core"
)]
pub struct AccountingCalendarRepo {
    pool: PgPool,
}

impl AccountingCalendarRepo {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn find_is_open_by_chart_id(
        &self,
        chart_id: ChartId,
    ) -> Result<AccountingCalendar, AccountingCalendarError> {
        match self
            .list_for_chart_id_by_created_at(
                chart_id,
                Default::default(),
                ListDirection::Descending,
            )
            .await?
            .entities
            .into_iter()
            .find(|f| f.is_open)
        {
            Some(calendar) => Ok(calendar),
            None => Err(AccountingCalendarError::OpenAccountingCalendarNotFoundForChart(chart_id)),
        }
    }
}
