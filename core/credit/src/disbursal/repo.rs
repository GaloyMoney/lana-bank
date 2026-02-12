use es_entity::clock::ClockHandle;
use sqlx::PgPool;

use es_entity::*;
use obix::out::OutboxEventMarker;
use tracing_macros::record_error_severity;

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
        public_id(ty = "PublicId", list_by)
    ),
    tbl_prefix = "core",
    post_persist_hook = "publish_in_op"
)]
pub(crate) struct DisbursalRepo<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pool: PgPool,
    publisher: CreditFacilityPublisher<E>,
    clock: ClockHandle,
}

impl<E> Clone for DisbursalRepo<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            publisher: self.publisher.clone(),
            clock: self.clock.clone(),
        }
    }
}

impl<E> DisbursalRepo<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub(crate) fn new(
        pool: &PgPool,
        publisher: &CreditFacilityPublisher<E>,
        clock: ClockHandle,
    ) -> Self {
        Self {
            pool: pool.clone(),
            publisher: publisher.clone(),
            clock,
        }
    }

    #[record_error_severity]
    #[tracing::instrument(name = "disbursal.publish_in_op", skip_all)]
    async fn publish_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &Disbursal,
        new_events: es_entity::LastPersisted<'_, DisbursalEvent>,
    ) -> Result<(), DisbursalError> {
        self.publisher
            .publish_disbursal_in_op(op, entity, new_events)
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
            DisbursalsSortBy::PublicId => {
                disbursal_cursor::DisbursalsByPublicIdCursor::from(disbursal).into()
            }
        }
    }
}
