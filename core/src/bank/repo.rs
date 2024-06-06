use sqlx::PgPool;

use super::{entity::*, error::*};
use crate::{entity::*, primitives::*};

#[derive(Clone)]
pub struct BankRepo {
    pool: PgPool,
}

impl BankRepo {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    pub(super) async fn create(&self, new_bank: NewBank) -> Result<EntityUpdate<Bank>, BankError> {
        let mut db_tx = self.pool.begin().await?;
        sqlx::query!(
            r#"INSERT INTO banks (id)
            VALUES ($1)"#,
            new_bank.id as BankId,
        )
        .execute(&mut *db_tx)
        .await?;
        let mut events = new_bank.initial_events();
        let n_new_events = events.persist(&mut db_tx).await?;
        db_tx.commit().await?;
        let bank = Bank::try_from(events)?;
        Ok(EntityUpdate {
            entity: bank,
            n_new_events,
        })
    }

    pub(super) async fn find_by_id(&self, id: BankId) -> Result<Bank, BankError> {
        let rows = sqlx::query_as!(
            GenericEvent,
            r#"SELECT l.id, e.sequence, e.event,
                      l.created_at AS entity_created_at, e.recorded_at AS event_recorded_at
            FROM banks l
            JOIN bank_events e ON l.id = e.id
            WHERE l.id = $1
            ORDER BY e.sequence"#,
            id as BankId,
        )
        .fetch_all(&self.pool)
        .await?;

        let res = EntityEvents::load_first::<Bank>(rows)?;
        Ok(res)
    }
}
