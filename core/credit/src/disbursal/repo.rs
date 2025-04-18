use sqlx::PgPool;

use es_entity::*;
use outbox::OutboxEventMarker;

use crate::{event::CoreCreditEvent, primitives::*, publisher::CreditFacilityPublisher};

use super::{entity::*, error::DisbursalError};

#[derive(EsRepo)]
#[es_repo(
    entity = "Disbursal",
    err = "DisbursalError",
    columns(
        credit_facility_id(ty = "CreditFacilityId", list_for, update(persist = false)),
        obligation_id(
            ty = "Option<ObligationId>",
            list_for,
            create(persist = false),
            update(accessor = "obligation_id()")
        ),
        approval_process_id(ty = "ApprovalProcessId", list_by, update(persist = "false")),
        concluded_tx_id(ty = "Option<LedgerTxId>", create(persist = false)),
    ),
    tbl_prefix = "core",
    post_persist_hook = "publish"
)]
pub struct DisbursalRepo<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pool: PgPool,
    publisher: CreditFacilityPublisher<E>,
}

impl<E> Clone for DisbursalRepo<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            publisher: self.publisher.clone(),
        }
    }
}

impl<E> DisbursalRepo<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub fn new(pool: &PgPool, publisher: &CreditFacilityPublisher<E>) -> Self {
        Self {
            pool: pool.clone(),
            publisher: publisher.clone(),
        }
    }

    async fn publish(
        &self,
        db: &mut es_entity::DbOp<'_>,
        entity: &Disbursal,
        new_events: es_entity::LastPersisted<'_, DisbursalEvent>,
    ) -> Result<(), DisbursalError> {
        self.publisher
            .publish_disbursal(db, entity, new_events)
            .await
    }
}

impl From<(DisbursalsSortBy, &Disbursal)> for disbursal_cursor::DisbursalsCursor {
    fn from(disbursal_with_sort: (DisbursalsSortBy, &Disbursal)) -> Self {
        let (sort, disbursal) = disbursal_with_sort;
        match sort {
            DisbursalsSortBy::CreatedAt => {
                disbursal_cursor::DisbursalsByCreatedAtCursor::from(disbursal).into()
            }
            DisbursalsSortBy::ApprovalProcessId => {
                disbursal_cursor::DisbursalsByApprovalProcessIdCursor::from(disbursal).into()
            }
            DisbursalsSortBy::Id => disbursal_cursor::DisbursalsByIdCursor::from(disbursal).into(),
        }
    }
}
