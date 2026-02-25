use es_entity::clock::ClockHandle;
use sqlx::PgPool;

use es_entity::*;
pub use es_entity::{ListDirection, Sort};
use obix::out::OutboxEventMarker;
use tracing_macros::record_error_severity;

use core_credit_collateral::CollateralId;

use crate::{CoreCreditEvent, primitives::*, publisher::*};

use super::{
    entity::*,
    error::CreditFacilityError,
    interest_accrual_cycle::{error::InterestAccrualCycleError, *},
};

#[derive(EsRepo)]
#[es_repo(
    entity = "CreditFacility",
    err = "CreditFacilityError",
    columns(
        customer_id(ty = "CustomerId", list_for(by(created_at)), update(persist = false)),
        collateral_id(ty = "CollateralId", update(persist = false)),
        pending_credit_facility_id(ty = "PendingCreditFacilityId", update(persist = false)),
        collateralization_ratio(
            ty = "CollateralizationRatio",
            list_by,
            create(persist = false),
            update(accessor = "last_collateralization_ratio()")
        ),
        collateralization_state(
            ty = "CollateralizationState",
            list_for,
            update(accessor = "last_collateralization_state()")
        ),
        status(ty = "CreditFacilityStatus", list_for, update(accessor = "status()")),
        public_id(ty = "PublicId", list_by)
    ),
    tbl_prefix = "core",
    post_persist_hook = "publish_in_op"
)]
pub struct CreditFacilityRepo<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pool: PgPool,
    publisher: CreditFacilityPublisher<E>,
    clock: ClockHandle,

    #[es_repo(nested)]
    interest_accruals: InterestAccrualRepo<E>,
}

impl<E> Clone for CreditFacilityRepo<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            publisher: self.publisher.clone(),
            clock: self.clock.clone(),
            interest_accruals: self.interest_accruals.clone(),
        }
    }
}

impl<E> CreditFacilityRepo<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub fn new(pool: &PgPool, publisher: &CreditFacilityPublisher<E>, clock: ClockHandle) -> Self {
        let interest_accruals = InterestAccrualRepo::new(pool, publisher, clock.clone());
        Self {
            pool: pool.clone(),
            publisher: publisher.clone(),
            clock,
            interest_accruals,
        }
    }

    #[record_error_severity]
    #[tracing::instrument(name = "credit_facility.publish_in_op", skip_all)]
    async fn publish_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &CreditFacility,
        new_events: es_entity::LastPersisted<'_, CreditFacilityEvent>,
    ) -> Result<(), CreditFacilityError> {
        self.publisher
            .publish_facility_in_op(op, entity, new_events)
            .await
    }

    #[record_error_severity]
    #[tracing::instrument(name = "credit_facility.find_by_custody_wallet", skip_all)]
    pub async fn find_by_custody_wallet(
        &self,
        wallet_id: CustodyWalletId,
    ) -> Result<CreditFacility, CreditFacilityError> {
        es_query!(
            tbl_prefix = "core",
            r#"
                SELECT cf.id FROM core_credit_facilities cf
                LEFT JOIN core_collaterals co ON cf.collateral_id = co.id
                WHERE co.custody_wallet_id = $1"#,
            wallet_id as CustodyWalletId
        )
        .fetch_one(&mut self.pool().begin().await?)
        .await
    }
}

#[derive(EsRepo)]
#[es_repo(
    entity = "InterestAccrualCycle",
    err = "InterestAccrualCycleError",
    columns(
        credit_facility_id(ty = "CreditFacilityId", update(persist = false), list_for, parent),
        idx(ty = "InterestAccrualCycleIdx", update(persist = false), list_by),
        next_accrual_period_end(
            ty = "Option<chrono::NaiveDate>",
            create(accessor = "next_accrual_period_end()"),
            update(accessor = "next_accrual_period_end()")
        ),
    ),
    tbl_prefix = "core",
    post_persist_hook = "publish_in_op"
)]
pub(super) struct InterestAccrualRepo<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pool: PgPool,
    publisher: CreditFacilityPublisher<E>,
    clock: ClockHandle,
}

impl<E> Clone for InterestAccrualRepo<E>
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

impl<E> InterestAccrualRepo<E>
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
    #[tracing::instrument(name = "interest_accrual_cycle.publish_in_op", skip_all)]
    async fn publish_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &InterestAccrualCycle,
        new_events: es_entity::LastPersisted<'_, InterestAccrualCycleEvent>,
    ) -> Result<(), InterestAccrualCycleError> {
        self.publisher
            .publish_interest_accrual_cycle_in_op(op, entity, new_events)
            .await
    }
}

mod facility_status_sqlx {
    use sqlx::{Type, postgres::*};

    use crate::primitives::CreditFacilityStatus;

    impl Type<Postgres> for CreditFacilityStatus {
        fn type_info() -> PgTypeInfo {
            <String as Type<Postgres>>::type_info()
        }

        fn compatible(ty: &PgTypeInfo) -> bool {
            <String as Type<Postgres>>::compatible(ty)
        }
    }

    impl sqlx::Encode<'_, Postgres> for CreditFacilityStatus {
        fn encode_by_ref(
            &self,
            buf: &mut PgArgumentBuffer,
        ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Sync + Send>> {
            <String as sqlx::Encode<'_, Postgres>>::encode(self.to_string(), buf)
        }
    }

    impl<'r> sqlx::Decode<'r, Postgres> for CreditFacilityStatus {
        fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
            let s = <String as sqlx::Decode<Postgres>>::decode(value)?;
            Ok(s.parse().map_err(|e: strum::ParseError| Box::new(e))?)
        }
    }

    impl PgHasArrayType for CreditFacilityStatus {
        fn array_type_info() -> PgTypeInfo {
            <String as sqlx::postgres::PgHasArrayType>::array_type_info()
        }
    }
}

impl From<(CreditFacilitiesSortBy, &CreditFacility)>
    for credit_facility_cursor::CreditFacilitiesCursor
{
    fn from(credit_facility_with_sort: (CreditFacilitiesSortBy, &CreditFacility)) -> Self {
        let (sort, credit_facility) = credit_facility_with_sort;
        match sort {
            CreditFacilitiesSortBy::CreatedAt => {
                credit_facility_cursor::CreditFacilitiesByCreatedAtCursor::from(credit_facility)
                    .into()
            }
            CreditFacilitiesSortBy::CollateralizationRatio => {
                credit_facility_cursor::CreditFacilitiesByCollateralizationRatioCursor::from(
                    credit_facility,
                )
                .into()
            }
            CreditFacilitiesSortBy::Id => {
                credit_facility_cursor::CreditFacilitiesByIdCursor::from(credit_facility).into()
            }
            CreditFacilitiesSortBy::PublicId => {
                credit_facility_cursor::CreditFacilitiesByPublicIdCursor::from(credit_facility)
                    .into()
            }
        }
    }
}
