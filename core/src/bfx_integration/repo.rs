use sqlx::PgPool;

use super::{entity::*, error::*};
use crate::{
    entity::*,
    primitives::{BfxIntegrationId, LedgerAccountId, LedgerAccountSetId},
};

#[derive(Clone)]
pub struct BfxIntegrationRepo {
    pool: PgPool,
}

impl BfxIntegrationRepo {
    pub(super) fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    pub(super) async fn create(
        &self,
        new_bfx_integration: NewBfxIntegration,
    ) -> Result<EntityUpdate<BfxIntegration>, BfxIntegrationError> {
        let mut tx = self.pool.begin().await?;
        sqlx::query!(
            r#"INSERT INTO bfx_integrations (id, omnibus_account_set_id, withdrawal_account_id)
            VALUES ($1, $2, $3)"#,
            new_bfx_integration.id as BfxIntegrationId,
            new_bfx_integration.omnibus_account_set_id as LedgerAccountSetId,
            new_bfx_integration.withdrawal_account_id as LedgerAccountId,
        )
        .execute(&mut *tx)
        .await?;
        let mut events = new_bfx_integration.initial_events();
        let n_new_events = events.persist(&mut tx).await?;
        tx.commit().await?;
        let bfx_integration = BfxIntegration::try_from(events)?;
        Ok(EntityUpdate {
            entity: bfx_integration,
            n_new_events,
        })
    }

    pub async fn find_by_id(
        &self,
        id: BfxIntegrationId,
    ) -> Result<BfxIntegration, BfxIntegrationError> {
        let rows = sqlx::query_as!(
            GenericEvent,
            r#"SELECT b.id, e.sequence, e.event,
                      b.created_at AS entity_created_at, e.recorded_at AS event_recorded_at
            FROM bfx_integrations b
            JOIN bfx_integration_events e ON b.id = e.id
            WHERE b.id = $1
            ORDER BY e.sequence"#,
            id as BfxIntegrationId,
        )
        .fetch_all(&self.pool)
        .await?;

        let res = EntityEvents::load_first::<BfxIntegration>(rows)?;
        Ok(res)
    }

    pub async fn persist_in_tx(
        &self,
        db: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        settings: &mut BfxIntegration,
    ) -> Result<(), BfxIntegrationError> {
        settings.events.persist(db).await?;
        Ok(())
    }
}
