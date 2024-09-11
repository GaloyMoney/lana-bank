use sqlx::{PgPool, Postgres, Transaction};

use crate::{
    data_export::Export,
    entity::{EntityEvents, GenericEvent},
    primitives::{DisbursementId, DisbursementIdx, LoanId},
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

    pub async fn persist_in_tx(
        &self,
        db: &mut Transaction<'_, Postgres>,
        disbursement: &mut Disbursement,
    ) -> Result<(), LoanError> {
        let n_events = disbursement._events.persist(db).await?;
        self.export
            .export_last(db, BQ_TABLE_NAME, n_events, &disbursement._events)
            .await?;
        Ok(())
    }

    pub async fn find_by_idx_for_loan(
        &self,
        loan_id: LoanId,
        idx: DisbursementIdx,
    ) -> Result<Disbursement, LoanError> {
        let rows = sqlx::query_as!(
            GenericEvent,
            r#"SELECT d.id, e.sequence, e.event,
                      d.created_at AS entity_created_at, e.recorded_at AS event_recorded_at
            FROM disbursements d
            JOIN disbursement_events e ON d.id = e.id
            WHERE d.loan_id = $1 AND d.idx = $2
            ORDER BY e.sequence"#,
            loan_id as LoanId,
            idx as DisbursementIdx,
        )
        .fetch_all(&self._pool)
        .await?;

        let res = EntityEvents::load_first::<Disbursement>(rows)?;
        Ok(res)
    }
}
