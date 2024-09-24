use sqlx::{PgPool, Postgres, Transaction};

use crate::{
    data_export::Export,
    primitives::{LoanId, UnaccruedInterestId, UnaccruedInterestIdx},
};

use crate::loan::error::LoanError;

use super::entity::*;

const BQ_TABLE_NAME: &str = "unaccrued_interest_events";

#[derive(Clone)]
pub(in crate::loan) struct UnaccruedInterestRepo {
    _pool: PgPool,
    export: Export,
}

impl UnaccruedInterestRepo {
    pub fn new(pool: &PgPool, export: &Export) -> Self {
        Self {
            _pool: pool.clone(),
            export: export.clone(),
        }
    }

    pub async fn create_in_tx(
        &self,
        db: &mut Transaction<'_, Postgres>,
        new_disbursement: NewUnaccruedInterest,
    ) -> Result<UnaccruedInterest, LoanError> {
        sqlx::query!(
            r#"INSERT INTO unaccrued_interests (id, loan_id, idx)
            VALUES ($1, $2, $3)"#,
            new_disbursement.id as UnaccruedInterestId,
            new_disbursement.loan_id as LoanId,
            new_disbursement.idx as UnaccruedInterestIdx,
        )
        .execute(&mut **db)
        .await?;
        let mut events = new_disbursement.initial_events();
        let n_events = events.persist(db).await?;
        self.export
            .export_last(db, BQ_TABLE_NAME, n_events, &events)
            .await?;
        Ok(UnaccruedInterest::try_from(events)?)
    }
}
