use sqlx::PgPool;

use super::{entity::*, error::*};
use crate::{
    entity::*,
    primitives::{CustomerId, DepositId},
};

#[derive(Clone)]
pub struct DepositRepo {
    pool: PgPool,
}

impl DepositRepo {
    pub(super) fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    pub(super) async fn create_in_tx(
        &self,
        db: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        new_deposit: NewDeposit,
    ) -> Result<Deposit, DepositError> {
        sqlx::query!(
            r#"INSERT INTO deposits (id, customer_id, reference)
            VALUES ($1, $2, $3)"#,
            new_deposit.id as DepositId,
            new_deposit.customer_id as CustomerId,
            new_deposit.reference()
        )
        .execute(&mut **db)
        .await?;
        let mut events = new_deposit.initial_events();
        events.persist(db).await?;
        let deposit = Deposit::try_from(events)?;
        Ok(deposit)
    }

    pub async fn find_by_id(&self, id: DepositId) -> Result<Deposit, DepositError> {
        let rows = sqlx::query_as!(
            GenericEvent,
            r#"SELECT w.id, e.sequence, e.event,
                      w.created_at AS entity_created_at, e.recorded_at AS event_recorded_at
            FROM deposits w
            JOIN deposit_events e ON w.id = e.id
            WHERE w.id = $1
            ORDER BY e.sequence"#,
            id as DepositId,
        )
        .fetch_all(&self.pool)
        .await?;

        let res = EntityEvents::load_first::<Deposit>(rows)?;
        Ok(res)
    }
}
