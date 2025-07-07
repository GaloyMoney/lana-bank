use sqlx::PgPool;

use es_entity::*;

use crate::primitives::*;

use super::{entity::*, error::*};

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "Custodian",
    err = "CustodianError",
    columns(name(ty = "String", list_by), provider(ty = "String", find_by)),
    tbl_prefix = "core"
)]
pub(crate) struct CustodianRepo {
    pool: PgPool,
}

impl CustodianRepo {
    pub(crate) fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn list_all(&self) -> Result<Vec<Custodian>, CustodianError> {
        let mut custodians = Vec::new();
        let mut next = Some(PaginatedQueryArgs::default());

        while let Some(query) = next.take() {
            let mut ret = self.list_by_id(query, Default::default()).await?;

            custodians.append(&mut ret.entities);
            next = ret.into_next_query();
        }

        Ok(custodians)
    }

    pub async fn persist_webhook_notification(
        &self,
        custodian_id: Option<CustodianId>,
        uri: &http::Uri,
        headers: &http::HeaderMap,
        payload: &serde_json::Value,
    ) -> Result<(), CustodianError> {
        let headers = serde_json::to_value(
            headers
                .iter()
                .map(|(name, value)| (name.as_str(), value.to_str().unwrap_or("<unreadable>")))
                .collect::<Vec<_>>(),
        )
        .expect("valid JSON");

        sqlx::query!(
            r#"
              INSERT INTO core_custodian_webhook_notifications (custodian_id, uri, headers, payload)
              VALUES ($1, $2, $3, $4)
            "#,
            custodian_id as Option<CustodianId>,
            uri.to_string(),
            headers,
            payload
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_config_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        custodian: &mut Custodian,
    ) -> Result<(), CustodianError> {
        sqlx::query!(
            r#"
            UPDATE core_custodian_events
            SET event = jsonb_set(event, '{encrypted_custodian_config}', 'null'::jsonb, false)
            WHERE id = $1 
              AND event_type = 'config_updated'
              AND event->'encrypted_custodian_config' IS NOT NULL;
            "#,
            custodian.id as CustodianId,
        )
        .execute(&mut **op.tx())
        .await?;

        self.update_in_op(op, custodian).await?;

        Ok(())
    }
}
