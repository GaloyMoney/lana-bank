use es_entity::clock::ClockHandle;
use sqlx::PgPool;

use es_entity::*;
use obix::out::OutboxEventMarker;

use old_money::UsdCents;

use crate::{CoreCreditEvent, primitives::*, publisher::*};
use core_credit_collateral::CollateralId;

use super::entity::*;

#[derive(EsRepo)]
#[es_repo(
    entity = "PendingCreditFacility",
    columns(
        customer_id(ty = "CustomerId", list_for(by(created_at)), update(persist = false)),
        credit_facility_proposal_id(ty = "CreditFacilityProposalId", update(persist = false)),
        approval_process_id(ty = "ApprovalProcessId", list_by, update(persist = "false")),
        collateral_id(ty = "CollateralId", update(persist = false)),
        collateralization_ratio(
            ty = "CollateralizationRatio",
            list_by,
            create(persist = false),
            update(accessor = "last_collateralization_ratio()")
        ),
        collateralization_state(
            ty = "PendingCreditFacilityCollateralizationState",
            list_for,
            update(accessor = "last_collateralization_state()")
        ),
        status(
            ty = "PendingCreditFacilityStatus",
            list_for,
            update(accessor = "status()")
        ),
        amount(ty = "UsdCents", list_by, update(persist = false))
    ),
    tbl_prefix = "core",
    post_persist_hook = "publish_in_op"
)]
pub struct PendingCreditFacilityRepo<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pool: PgPool,
    publisher: CreditFacilityPublisher<E>,
    clock: ClockHandle,
}

impl<E> Clone for PendingCreditFacilityRepo<E>
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

impl<E> PendingCreditFacilityRepo<E>
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

    #[tracing::instrument(name = "pending_credit_facility.publish_in_op", skip_all)]
    async fn publish_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &PendingCreditFacility,
        new_events: es_entity::LastPersisted<'_, PendingCreditFacilityEvent>,
    ) -> Result<(), sqlx::Error> {
        self.publisher
            .publish_pending_credit_facility_in_op(op, entity, new_events)
            .await
    }
}

impl From<(PendingCreditFacilitiesSortBy, &PendingCreditFacility)>
    for pending_credit_facility_cursor::PendingCreditFacilitiesCursor
{
    fn from(facility_with_sort: (PendingCreditFacilitiesSortBy, &PendingCreditFacility)) -> Self {
        let (sort, facility) = facility_with_sort;
        match sort {
            PendingCreditFacilitiesSortBy::CreatedAt => {
                pending_credit_facility_cursor::PendingCreditFacilitiesByCreatedAtCursor::from(
                    facility,
                )
                .into()
            }
            PendingCreditFacilitiesSortBy::Id => {
                pending_credit_facility_cursor::PendingCreditFacilitiesByIdCursor::from(facility)
                    .into()
            }
            PendingCreditFacilitiesSortBy::ApprovalProcessId => {
                pending_credit_facility_cursor::PendingCreditFacilitiesByApprovalProcessIdCursor::from(facility)
                    .into()
            }
            PendingCreditFacilitiesSortBy::CollateralizationRatio => {
                pending_credit_facility_cursor::PendingCreditFacilitiesByCollateralizationRatioCursor::from(facility)
                    .into()
            }
            PendingCreditFacilitiesSortBy::Amount => {
                pending_credit_facility_cursor::PendingCreditFacilitiesByAmountCursor::from(
                    facility,
                )
                .into()
            }
        }
    }
}

mod pending_status_sqlx {
    use sqlx::{Type, postgres::*};

    use crate::primitives::PendingCreditFacilityStatus;

    impl Type<Postgres> for PendingCreditFacilityStatus {
        fn type_info() -> PgTypeInfo {
            <String as Type<Postgres>>::type_info()
        }

        fn compatible(ty: &PgTypeInfo) -> bool {
            <String as Type<Postgres>>::compatible(ty)
        }
    }

    impl sqlx::Encode<'_, Postgres> for PendingCreditFacilityStatus {
        fn encode_by_ref(
            &self,
            buf: &mut PgArgumentBuffer,
        ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Sync + Send>> {
            <String as sqlx::Encode<'_, Postgres>>::encode(self.to_string(), buf)
        }
    }

    impl<'r> sqlx::Decode<'r, Postgres> for PendingCreditFacilityStatus {
        fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
            let s = <String as sqlx::Decode<Postgres>>::decode(value)?;
            Ok(s.parse().map_err(|e: strum::ParseError| Box::new(e))?)
        }
    }

    impl PgHasArrayType for PendingCreditFacilityStatus {
        fn array_type_info() -> PgTypeInfo {
            <String as sqlx::postgres::PgHasArrayType>::array_type_info()
        }
    }
}
