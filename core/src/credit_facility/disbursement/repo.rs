use sqlx::{PgPool, Postgres, Transaction};

use crate::{
    data_export::Export,
    entity::{EntityEvents, GenericEvent},
    primitives::{CreditFacilityId, DisbursementId, DisbursementIdx},
};

use crate::credit_facility::error::CreditFacilityError;

use super::{entity::*, error::DisbursementError};

const BQ_TABLE_NAME: &str = "disbursement_events";

#[derive(Clone)]
pub(in crate::credit_facility) struct DisbursementRepo {
    pool: PgPool,
    export: Export,
}

impl DisbursementRepo {
    pub fn new(pool: &PgPool, export: &Export) -> Self {
        Self {
            pool: pool.clone(),
            export: export.clone(),
        }
    }

    pub async fn create_in_tx(
        &self,
        db: &mut Transaction<'_, Postgres>,
        new_disbursement: NewDisbursement,
    ) -> Result<Disbursement, DisbursementError> {
        sqlx::query!(
            r#"INSERT INTO disbursements (id, credit_facility_id, idx)
            VALUES ($1, $2, $3)"#,
            new_disbursement.id as DisbursementId,
            new_disbursement.facility_id as CreditFacilityId,
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
    ) -> Result<(), CreditFacilityError> {
        let n_events = disbursement.events.persist(db).await?;
        self.export
            .export_last(db, BQ_TABLE_NAME, n_events, &disbursement.events)
            .await?;
        Ok(())
    }

    pub async fn find_by_idx_for_credit_facility(
        &self,
        facility_id: CreditFacilityId,
        idx: DisbursementIdx,
    ) -> Result<Disbursement, DisbursementError> {
        let rows = sqlx::query_as!(
            GenericEvent,
            r#"SELECT d.id, e.sequence, e.event,
                      d.created_at AS entity_created_at, e.recorded_at AS event_recorded_at
            FROM disbursements d
            JOIN disbursement_events e ON d.id = e.id
            WHERE d.credit_facility_id = $1 AND d.idx = $2
            ORDER BY e.sequence"#,
            facility_id as CreditFacilityId,
            idx as DisbursementIdx,
        )
        .fetch_all(&self.pool)
        .await?;

        let res = EntityEvents::load_first::<Disbursement>(rows)?;
        Ok(res)
    }

    pub async fn list(
        &self,
        facility_id: CreditFacilityId,
    ) -> Result<Vec<Disbursement>, DisbursementError> {
        let rows = sqlx::query_as!(
            GenericEvent,
            r#"SELECT d.id, e.sequence, e.event,
                      d.created_at AS entity_created_at, e.recorded_at AS event_recorded_at
            FROM disbursements d
            JOIN disbursement_events e ON d.id = e.id
            where d.credit_facility_id = $1
            ORDER BY d.id, e.sequence"#,
            facility_id as CreditFacilityId,
        )
        .fetch_all(&self.pool)
        .await?;

        let n = rows.len();
        let res = EntityEvents::load_n::<Disbursement>(rows, n)?;
        Ok(res.0)
    }
}
