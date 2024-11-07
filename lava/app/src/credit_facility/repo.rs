use rust_decimal::Decimal;
use sqlx::PgPool;

use es_entity::*;

use crate::{primitives::*, terms::CollateralizationState};

use super::{entity::*, error::CreditFacilityError, publisher::*};

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "CreditFacility",
    err = "CreditFacilityError",
    columns(
        customer_id(ty = "CustomerId", list_for, update(persist = false)),
        approval_process_id(ty = "ApprovalProcessId", update(persist = "false")),
        collateralization_ratio(
            ty = "Option<Decimal>",
            create(persist = false),
            update(accessor = "collateralization_ratio()")
        ),
        collateralization_state(
            ty = "CollateralizationState",
            list_for,
            create(persist = false),
            update(accessor = "last_collateralization_state()")
        ),
        status(
            ty = "CreditFacilityStatus",
            list_for,
            create(persist = false),
            update(accessor = "status()")
        )
    ),
    post_persist_hook = "publish"
)]
pub struct CreditFacilityRepo {
    pool: PgPool,
    publisher: CreditFacilityPublisher,
}

impl CreditFacilityRepo {
    pub(super) fn new(pool: &PgPool, publisher: &CreditFacilityPublisher) -> Self {
        Self {
            pool: pool.clone(),
            publisher: publisher.clone(),
        }
    }

    async fn publish(
        &self,
        db: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        entity: &CreditFacility,
        new_events: es_entity::LastPersisted<'_, CreditFacilityEvent>,
    ) -> Result<(), CreditFacilityError> {
        self.publisher.publish(db, entity, new_events).await
    }
}

mod facility_status_sqlx {
    use sqlx::{postgres::*, Type};

    use crate::primitives::CreditFacilityStatus;

    impl Type<Postgres> for CreditFacilityStatus {
        fn type_info() -> PgTypeInfo {
            <String as Type<Postgres>>::type_info()
        }

        fn compatible(ty: &PgTypeInfo) -> bool {
            <String as Type<Postgres>>::compatible(ty)
        }
    }

    impl<'q> sqlx::Encode<'q, Postgres> for CreditFacilityStatus {
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
    use sqlx::{postgres::*, Type};

    use crate::terms::CollateralizationState;

    impl Type<Postgres> for CollateralizationState {
        fn type_info() -> PgTypeInfo {
            <String as Type<Postgres>>::type_info()
        }

        fn compatible(ty: &PgTypeInfo) -> bool {
            <String as Type<Postgres>>::compatible(ty)
        }
    }

    impl<'q> sqlx::Encode<'q, Postgres> for CollateralizationState {
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
