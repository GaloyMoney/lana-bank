use sqlx::PgPool;

pub use es_entity::Sort;
use es_entity::*;

use crate::primitives::*;

use super::{entity::*, error::*};

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "Customer",
    err = "CustomerError",
    columns(
        email(ty = "String", list_by),
        telegram_id(ty = "String", list_by),
        status(ty = "AccountStatus", list_for)
    )
)]
pub struct CustomerRepo {
    pool: PgPool,
}

impl CustomerRepo {
    pub(super) fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }
}

mod account_status_sqlx {
    use sqlx::{postgres::*, Type};

    use crate::primitives::AccountStatus;

    impl Type<Postgres> for AccountStatus {
        fn type_info() -> PgTypeInfo {
            <String as Type<Postgres>>::type_info()
        }

        fn compatible(ty: &PgTypeInfo) -> bool {
            <String as Type<Postgres>>::compatible(ty)
        }
    }

    impl sqlx::Encode<'_, Postgres> for AccountStatus {
        fn encode_by_ref(
            &self,
            buf: &mut PgArgumentBuffer,
        ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Sync + Send>> {
            <String as sqlx::Encode<'_, Postgres>>::encode(self.to_string(), buf)
        }
    }

    impl<'r> sqlx::Decode<'r, Postgres> for AccountStatus {
        fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
            let s = <String as sqlx::Decode<Postgres>>::decode(value)?;
            Ok(s.parse().map_err(|e: strum::ParseError| Box::new(e))?)
        }
    }

    impl PgHasArrayType for AccountStatus {
        fn array_type_info() -> PgTypeInfo {
            <String as sqlx::postgres::PgHasArrayType>::array_type_info()
        }
    }
}

impl From<(CustomersSortBy, &Customer)> for customer_cursor::CustomersCursor {
    fn from(customer_with_sort: (CustomersSortBy, &Customer)) -> Self {
        let (sort, customer) = customer_with_sort;
        match sort {
            CustomersSortBy::CreatedAt => {
                customer_cursor::CustomersByCreatedAtCursor::from(customer).into()
            }
            CustomersSortBy::Email => {
                customer_cursor::CustomersByEmailCursor::from(customer).into()
            }
            CustomersSortBy::TelegramId => {
                customer_cursor::CustomersByTelegramIdCursor::from(customer).into()
            }
            CustomersSortBy::Id => customer_cursor::CustomersByIdCursor::from(customer).into(),
        }
    }
}
