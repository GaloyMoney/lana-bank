use rust_decimal::Decimal;
use sqlx::PgPool;

use es_entity::*;
pub use es_entity::{ListDirection, Sort};
use outbox::OutboxEventMarker;

use crate::{
    event::CoreCreditEvent,
    interest_accrual_cycle::{error::InterestAccrualCycleError, *},
    primitives::*,
    publisher::*,
};

use super::{entity::*, error::CreditFacilityError};

#[derive(EsRepo)]
#[es_repo(
    entity = "CreditFacility",
    err = "CreditFacilityError",
    columns(
        customer_id(ty = "CustomerId", list_for, update(persist = false)),
        approval_process_id(ty = "ApprovalProcessId", list_by, update(persist = "false")),
        collateral_id(ty = "CollateralId", update(persist = false)),
        collateralization_ratio(
            ty = "Option<Decimal>",
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
    post_persist_hook = "publish"
)]
pub struct CreditFacilityRepo<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pool: PgPool,
    publisher: CreditFacilityPublisher<E>,

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
            interest_accruals: self.interest_accruals.clone(),
        }
    }
}

impl<E> CreditFacilityRepo<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub fn new(pool: &PgPool, publisher: &CreditFacilityPublisher<E>) -> Self {
        let interest_accruals = InterestAccrualRepo::new(pool, publisher);
        Self {
            pool: pool.clone(),
            publisher: publisher.clone(),
            interest_accruals,
        }
    }

    async fn publish(
        &self,
        db: &mut es_entity::DbOp<'_>,
        entity: &CreditFacility,
        new_events: es_entity::LastPersisted<'_, CreditFacilityEvent>,
    ) -> Result<(), CreditFacilityError> {
        self.publisher
            .publish_facility(db, entity, new_events)
            .await
    }

    pub async fn find_by_custody_wallet(
        &self,
        wallet_id: CustodyWalletId,
    ) -> Result<CreditFacility, CreditFacilityError> {
        es_query!(
            "core",
            self.pool(),
            r#"
                SELECT cf.id FROM core_credit_facilities cf
                LEFT JOIN core_collaterals co ON cf.collateral_id = co.id
                WHERE co.custody_wallet_id = $1"#,
            wallet_id as CustodyWalletId
        )
        .fetch_one()
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
    ),
    tbl_prefix = "core",
    post_persist_hook = "publish"
)]
pub(super) struct InterestAccrualRepo<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pool: PgPool,
    publisher: CreditFacilityPublisher<E>,
}

impl<E> Clone for InterestAccrualRepo<E>
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

impl<E> InterestAccrualRepo<E>
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
        entity: &InterestAccrualCycle,
        new_events: es_entity::LastPersisted<'_, InterestAccrualCycleEvent>,
    ) -> Result<(), InterestAccrualCycleError> {
        self.publisher
            .publish_interest_accrual_cycle(db, entity, new_events)
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

mod facility_collateralization_state_sqlx {
    use sqlx::{Type, postgres::*};

    use crate::primitives::CollateralizationState;

    impl Type<Postgres> for CollateralizationState {
        fn type_info() -> PgTypeInfo {
            <String as Type<Postgres>>::type_info()
        }

        fn compatible(ty: &PgTypeInfo) -> bool {
            <String as Type<Postgres>>::compatible(ty)
        }
    }

    impl sqlx::Encode<'_, Postgres> for CollateralizationState {
        fn encode_by_ref(
            &self,
            buf: &mut PgArgumentBuffer,
        ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Sync + Send>> {
            <String as sqlx::Encode<'_, Postgres>>::encode(self.to_string(), buf)
        }
    }

    impl<'r> sqlx::Decode<'r, Postgres> for CollateralizationState {
        fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
            let s = <String as sqlx::Decode<Postgres>>::decode(value)?;
            Ok(s.parse().map_err(|e: strum::ParseError| Box::new(e))?)
        }
    }

    impl PgHasArrayType for CollateralizationState {
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
            CreditFacilitiesSortBy::ApprovalProcessId => {
                credit_facility_cursor::CreditFacilitiesByApprovalProcessIdCursor::from(
                    credit_facility,
                )
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
