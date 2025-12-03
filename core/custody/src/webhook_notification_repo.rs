use sqlx::PgPool;
use tracing::instrument;
use tracing_macros::record_error_severity;

use crate::{error::*, primitives::*};

#[derive(Clone)]
pub(crate) struct WebhookNotificationRepo {
    pool: PgPool,
}

impl WebhookNotificationRepo {
    pub(crate) fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    #[record_error_severity]
    #[instrument(name = "custody.webhook_notification.persist", skip(self))]
    pub async fn persist(
        &self,
        custodian_id: Option<CustodianId>,
        uri: &http::Uri,
        headers: &http::HeaderMap,
        payload: &[u8],
    ) -> Result<(), CoreCustodyError> {
        let payload = serde_json::from_slice::<serde_json::Value>(payload).unwrap_or_default();

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
}
