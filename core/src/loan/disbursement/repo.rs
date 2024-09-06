use sqlx::{PgPool, Postgres, Transaction};

use crate::{
    data_export::Export,
    primitives::{DisbursementIdx, LoanId},
};

use crate::loan::error::LoanError;

use super::entity::*;

const BQ_TABLE_NAME: &str = "disbursement_events";

#[derive(Clone)]
pub(in crate::loan) struct DisbursementRepo {
    _pool: PgPool,
    export: Export,
}

impl DisbursementRepo {
    pub fn new(pool: &PgPool, export: &Export) -> Self {
        Self {
            _pool: pool.clone(),
            export: export.clone(),
        }
    }

    pub async fn create_in_tx(
        &self,
        db: &mut Transaction<'_, Postgres>,
        new_disbursement: NewDisbursement,
    ) -> Result<Disbursement, LoanError> {
        sqlx::query!(
            r#"INSERT INTO disbursements (id, loan_id, idx)
            VALUES ($1, $2, $3)"#,
            new_disbursement.id as DisbursementId,
            new_disbursement.loan_id as LoanId,
            new_disbursement.idx as DisbursementIdx,
        )
        .execute(&mut **db)
        .await?;
        let mut events = new_disbursement.initial_events();
        let n_events = events.persist(db).await?;
        self.export
            .export_last(db, BQ_TABLE_NAME, n_events, &events)
            .await?;
        Ok(Disbursement::try_from(events)?)
    }
}
