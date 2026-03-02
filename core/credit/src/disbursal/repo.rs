use es_entity::clock::ClockHandle;
use sqlx::PgPool;

use es_entity::*;
use obix::out::OutboxEventMarker;
use tracing_macros::record_error_severity;

use crate::{CoreCreditEvent, primitives::*, publisher::CreditFacilityPublisher};

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
        public_id(ty = "PublicId", list_by),
        status(ty = "DisbursalStatus", list_for, update(accessor = "status()"))
    ),
    tbl_prefix = "core",
    post_persist_hook = "publish_in_op"
)]
pub struct DisbursalRepo<E>
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
    pub fn new(pool: &PgPool, publisher: &CreditFacilityPublisher<E>, clock: ClockHandle) -> Self {
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

mod disbursal_status_sqlx {
    use sqlx::{Type, postgres::*};

    use crate::primitives::DisbursalStatus;

    impl Type<Postgres> for DisbursalStatus {
        fn type_info() -> PgTypeInfo {
            <String as Type<Postgres>>::type_info()
        }

        fn compatible(ty: &PgTypeInfo) -> bool {
            <String as Type<Postgres>>::compatible(ty)
        }
    }

    impl sqlx::Encode<'_, Postgres> for DisbursalStatus {
        fn encode_by_ref(
            &self,
            buf: &mut PgArgumentBuffer,
        ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Sync + Send>> {
            <String as sqlx::Encode<'_, Postgres>>::encode(self.to_string(), buf)
        }
    }

    impl<'r> sqlx::Decode<'r, Postgres> for DisbursalStatus {
        fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
            let s = <String as sqlx::Decode<Postgres>>::decode(value)?;
            Ok(s.parse().map_err(|e: strum::ParseError| Box::new(e))?)
        }
    }

    impl PgHasArrayType for DisbursalStatus {
        fn array_type_info() -> PgTypeInfo {
            <String as sqlx::postgres::PgHasArrayType>::array_type_info()
        }
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
