use sqlx::PgPool;

use super::{entity::*, error::*};
use crate::{
    entity::*,
    primitives::{UserId, WithdrawId},
};

#[derive(Clone)]
pub struct WithdrawRepo {
    pool: PgPool,
}

impl WithdrawRepo {
    pub(super) fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    pub(super) async fn create(
        &self,
        new_withdraw: NewWithdraw,
    ) -> Result<Withdraw, WithdrawError> {
        let mut tx = self.pool.begin().await?;
        sqlx::query!(
            r#"INSERT INTO withdraws (id, user_id, reference)
            VALUES ($1, $2, $3)"#,
            new_withdraw.id as WithdrawId,
            new_withdraw.user_id as UserId,
            new_withdraw.reference()
        )
        .execute(&mut *tx)
        .await?;
        let mut events = new_withdraw.initial_events();
        events.persist(&mut tx).await?;
        tx.commit().await?;
        let withdraw = Withdraw::try_from(events)?;
        Ok(withdraw)
    }

    pub async fn find_by_id(&self, id: WithdrawId) -> Result<Withdraw, WithdrawError> {
        let rows = sqlx::query_as!(
            GenericEvent,
            r#"SELECT w.id, e.sequence, e.event,
                      w.created_at AS entity_created_at, e.recorded_at AS event_recorded_at
            FROM withdraws w
            JOIN withdraw_events e ON w.id = e.id
            WHERE w.id = $1
            ORDER BY e.sequence"#,
            id as WithdrawId,
        )
        .fetch_all(&self.pool)
        .await?;

        let res = EntityEvents::load_first::<Withdraw>(rows)?;
        Ok(res)
    }

    pub async fn persist_in_tx(
        &self,
        db: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        settings: &mut Withdraw,
    ) -> Result<(), WithdrawError> {
        settings.events.persist(db).await?;
        Ok(())
    }
}
