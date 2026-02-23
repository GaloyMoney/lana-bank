#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod config;
pub mod custodian;
pub mod error;
mod primitives;
pub mod public;
mod publisher;
pub mod wallet;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum::IntoDiscriminant as _;
use tracing::instrument;
use tracing_macros::record_error_severity;

use std::collections::HashMap;

use es_entity::DbOp;
use es_entity::clock::ClockHandle;
use obix::inbox::{Inbox, InboxConfig, InboxEvent, InboxHandler, InboxResult};
use obix::out::{Outbox, OutboxEventMarker};
pub use public::*;
pub use publisher::CustodyPublisher;

use audit::AuditSvc;
use authz::PermissionCheck;
use encryption::{EncryptionConfig, EncryptionKey};
use money::Satoshis;

pub use custodian::*;
pub use wallet::*;

pub use config::CustodyConfig;
use error::CoreCustodyError;
pub use primitives::*;

#[cfg(feature = "json-schema")]
pub mod event_schema {
    pub use crate::custodian::CustodianEvent;
    pub use crate::wallet::WalletEvent;
}

#[derive(Serialize, Deserialize)]
struct WebhookPayload {
    provider: String,
    uri: String,
    headers: HashMap<String, String>,
    payload: bytes::Bytes,
}

struct CustodianWebhookHandler<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustodyEvent>,
{
    authz: Perms,
    custodians: CustodianRepo,
    wallets: WalletRepo<E>,
    encryption_config: EncryptionConfig,
    config: CustodyConfig,
}

impl<Perms, E> Clone for CustodianWebhookHandler<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustodyEvent>,
{
    fn clone(&self) -> Self {
        Self {
            authz: self.authz.clone(),
            custodians: self.custodians.clone(),
            wallets: self.wallets.clone(),
            encryption_config: self.encryption_config.clone(),
            config: self.config.clone(),
        }
    }
}

impl<Perms, E> InboxHandler for CustodianWebhookHandler<Perms, E>
where
    Perms: PermissionCheck + Send + Sync,
    E: OutboxEventMarker<CoreCustodyEvent> + Send + Sync,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCustodyObject>,
{
    async fn handle(
        &self,
        event: &InboxEvent,
    ) -> Result<InboxResult, Box<dyn std::error::Error + Send + Sync>> {
        let payload: WebhookPayload = event.payload()?;

        match self.process_webhook(payload).await {
            Ok(_) => Ok(InboxResult::Complete),
            Err(e) => Err(Box::new(e)),
        }
    }
}

impl<Perms, E> CustodianWebhookHandler<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustodyEvent>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCustodyObject>,
{
    fn new(
        pool: &sqlx::PgPool,
        authz: &Perms,
        encryption_config: &EncryptionConfig,
        config: &CustodyConfig,
        outbox: &Outbox<E>,
        clock: ClockHandle,
    ) -> Self {
        let custodians = CustodianRepo::new(pool, clock.clone());
        let wallets = WalletRepo::new(pool, &CustodyPublisher::new(outbox), clock);
        Self {
            authz: authz.clone(),
            encryption_config: encryption_config.clone(),
            config: config.clone(),
            custodians,
            wallets,
        }
    }

    #[record_error_severity]
    #[instrument(name = "custody.process_webhook", skip(self))]
    async fn process_webhook(
        &self,
        WebhookPayload {
            provider,
            headers,
            payload,
            ..
        }: WebhookPayload,
    ) -> Result<(), CoreCustodyError> {
        let provider_name = provider.clone();
        let custodian = self.custodians.find_by_provider(provider).await;

        let header_map: http::HeaderMap = headers
            .into_iter()
            .filter_map(|(key, value)| Some((key.parse().ok()?, value.parse().ok()?)))
            .collect();

        if let Ok(custodian) = custodian
            && let Some(notification) = custodian
                .custodian_client(&self.encryption_config.key, &self.config.custody_providers)?
                .process_webhook(&header_map, payload)
                .await?
        {
            match notification {
                CustodianNotification::WalletBalanceChanged {
                    external_wallet_id,
                    new_balance,
                    changed_at,
                } => {
                    self.update_wallet_balance(
                        provider_name.clone(),
                        external_wallet_id,
                        new_balance,
                        changed_at,
                    )
                    .await?;
                }
            }
        }

        Ok(())
    }

    #[record_error_severity]
    #[instrument(name = "custody.update_wallet_balance", skip(self))]
    async fn update_wallet_balance(
        &self,
        provider: String,
        external_wallet_id: String,
        new_balance: Satoshis,
        update_time: DateTime<Utc>,
    ) -> Result<(), CoreCustodyError> {
        let mut db = self.wallets.begin_op().await?;

        let mut wallet = self
            .wallets
            .find_by_external_wallet_id_in_op(&mut db, external_wallet_id)
            .await?;

        self.authz
            .audit()
            .record_system_entry_in_op(
                &mut db,
                audit::SystemActor::from(provider),
                CoreCustodyObject::wallet(wallet.id),
                CoreCustodyAction::WALLET_UPDATE,
            )
            .await?;

        if wallet
            .update_balance(new_balance, update_time)
            .did_execute()
        {
            self.wallets.update_in_op(&mut db, &mut wallet).await?;
        }

        db.commit().await?;

        Ok(())
    }
}

const CUSTODY_INBOX_JOB: job::JobType = job::JobType::new("custody-inbox");
pub struct CoreCustody<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustodyEvent>,
{
    authz: Perms,
    custodians: CustodianRepo,
    encryption_config: EncryptionConfig,
    config: CustodyConfig,
    wallets: WalletRepo<E>,
    inbox: Inbox,
    clock: ClockHandle,
}

impl<Perms, E> CoreCustody<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCustodyEvent>,
{
    #[record_error_severity]
    #[tracing::instrument(name = "custody.init", skip_all)]
    pub async fn init(
        pool: &sqlx::PgPool,
        authz: &Perms,
        encryption_config: EncryptionConfig,
        config: CustodyConfig,
        outbox: &Outbox<E>,
        jobs: &mut job::Jobs,
        clock: ClockHandle,
    ) -> Result<Self, CoreCustodyError> {
        let handler = CustodianWebhookHandler::new(
            pool,
            authz,
            &encryption_config,
            &config,
            outbox,
            clock.clone(),
        );

        let inbox_config = InboxConfig::new(CUSTODY_INBOX_JOB);
        let inbox = Inbox::new(pool, jobs, inbox_config, handler);

        let custody = Self {
            authz: authz.clone(),
            custodians: CustodianRepo::new(pool, clock.clone()),
            encryption_config,
            config,
            wallets: WalletRepo::new(pool, &CustodyPublisher::new(outbox), clock.clone()),
            inbox,
            clock,
        };

        if let Some(deprecated_key) = custody.encryption_config.deprecated_encryption_key.as_ref() {
            custody.rotate_encryption_key(deprecated_key).await?;
        }

        Ok(custody)
    }

    #[cfg(feature = "mock-custodian")]
    #[record_error_severity]
    #[instrument(name = "credit_facility.ensure_mock_custodian_in_op", skip(self, db))]
    pub async fn ensure_mock_custodian_in_op(
        &self,
        db: &mut DbOp<'_>,
    ) -> Result<(), CoreCustodyError> {
        if self
            .custodians
            .maybe_find_by_id_in_op(&mut *db, CustodianId::mock_custodian_id())
            .await?
            .is_none()
        {
            let _ = self
                .create_mock_custodian_in_op(db, "Mock Custodian", CustodianConfig::Mock)
                .await?;
        }

        Ok(())
    }

    #[cfg(feature = "mock-custodian")]
    #[record_error_severity]
    #[instrument(name = "core_custody.create_mock_custodian_in_op", skip(self, db))]
    pub async fn create_mock_custodian_in_op(
        &self,
        db: &mut DbOp<'_>,
        custodian_name: impl AsRef<str> + std::fmt::Debug,
        custodian_config: CustodianConfig,
    ) -> Result<Custodian, CoreCustodyError> {
        let custodian_id = if custodian_config == CustodianConfig::Mock {
            CustodianId::mock_custodian_id()
        } else {
            CustodianId::new()
        };

        let new_custodian = NewCustodian::builder()
            .id(custodian_id)
            .name(custodian_name.as_ref().to_owned())
            .provider(custodian_config.discriminant().to_string())
            .encrypted_custodian_config(custodian_config, &self.encryption_config.key)
            .build()
            .expect("should always build a new custodian");

        let custodian = self.custodians.create_in_op(db, new_custodian).await?;

        Ok(custodian)
    }

    #[record_error_severity]
    #[instrument(name = "core_custody.create_custodian_in_op", skip(self, db))]
    pub async fn create_custodian_in_op(
        &self,
        db: &mut DbOp<'_>,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        custodian_name: impl AsRef<str> + std::fmt::Debug,
        custodian_config: CustodianConfig,
    ) -> Result<Custodian, CoreCustodyError> {
        self.authz
            .enforce_permission(
                sub,
                CoreCustodyObject::all_custodians(),
                CoreCustodyAction::CUSTODIAN_CREATE,
            )
            .await?;

        // We should not be calling any external service in any environment
        // with mock custodian.
        #[cfg(not(feature = "mock-custodian"))]
        custodian_config
            .clone()
            .custodian_client(&self.config.custody_providers)?
            .verify_client()
            .await?;

        #[cfg(feature = "mock-custodian")]
        let custodian_id = if custodian_config == CustodianConfig::Mock {
            CustodianId::mock_custodian_id()
        } else {
            CustodianId::new()
        };

        #[cfg(not(feature = "mock-custodian"))]
        let custodian_id = CustodianId::new();

        let new_custodian = NewCustodian::builder()
            .id(custodian_id)
            .name(custodian_name.as_ref().to_owned())
            .provider(custodian_config.discriminant().to_string())
            .encrypted_custodian_config(custodian_config, &self.encryption_config.key)
            .build()
            .expect("should always build a new custodian");

        let custodian = self.custodians.create_in_op(db, new_custodian).await?;

        Ok(custodian)
    }

    #[record_error_severity]
    #[instrument(name = "core_custody.create_custodian", skip(self, custodian_config))]
    pub async fn create_custodian(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        custodian_name: impl AsRef<str> + std::fmt::Debug,
        custodian_config: CustodianConfig,
    ) -> Result<Custodian, CoreCustodyError> {
        let mut db = self.custodians.begin_op().await?;

        let custodian = self
            .create_custodian_in_op(&mut db, sub, custodian_name, custodian_config)
            .await?;

        db.commit().await?;

        Ok(custodian)
    }

    pub async fn update_config(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        custodian_id: impl Into<CustodianId> + std::fmt::Debug,
        config: CustodianConfig,
    ) -> Result<Custodian, CoreCustodyError> {
        let id = custodian_id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreCustodyObject::custodian(id),
                CoreCustodyAction::CUSTODIAN_UPDATE,
            )
            .await?;
        let mut op = self.custodians.begin_op().await?;
        let mut custodian = self.custodians.find_by_id_in_op(&mut op, id).await?;

        if custodian
            .update_custodian_config(config, &self.encryption_config.key)
            .did_execute()
        {
            self.custodians
                .update_config_in_op(&mut op, &mut custodian)
                .await?;
            op.commit().await?;
        }

        Ok(custodian)
    }

    async fn rotate_encryption_key(
        &self,
        deprecated_key: &EncryptionKey,
    ) -> Result<(), CoreCustodyError> {
        let mut op = self.custodians.begin_op().await?;

        self.authz
            .audit()
            .record_system_entry_in_op(
                &mut op,
                crate::primitives::CUSTODY_KEY_ROTATION,
                CoreCustodyObject::all_custodians(),
                CoreCustodyAction::CUSTODIAN_UPDATE,
            )
            .await?;

        let mut custodians = self.custodians.list_all_in_op(&mut op).await?;

        for custodian in custodians.iter_mut() {
            if custodian
                .rotate_encryption_key(&self.encryption_config.key, deprecated_key)?
                .did_execute()
            {
                self.custodians
                    .update_config_in_op(&mut op, custodian)
                    .await?;
            }
        }

        op.commit().await?;

        Ok(())
    }

    #[record_error_severity]
    #[instrument(name = "core_custody.find_all_wallets", skip(self))]
    pub async fn find_all_wallets<T: From<Wallet>>(
        &self,
        ids: &[WalletId],
    ) -> Result<HashMap<WalletId, T>, CoreCustodyError> {
        Ok(self.wallets.find_all(ids).await?)
    }

    #[record_error_severity]
    #[instrument(name = "core_custody.find_all_custodians", skip(self))]
    pub async fn find_all_custodians<T: From<Custodian>>(
        &self,
        ids: &[CustodianId],
    ) -> Result<HashMap<CustodianId, T>, CoreCustodyError> {
        Ok(self.custodians.find_all(ids).await?)
    }

    #[record_error_severity]
    #[instrument(name = "core_custody.list_custodians", skip(self))]
    pub async fn list_custodians(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        query: es_entity::PaginatedQueryArgs<CustodiansByNameCursor>,
    ) -> Result<es_entity::PaginatedQueryRet<Custodian, CustodiansByNameCursor>, CoreCustodyError>
    {
        self.authz
            .enforce_permission(
                sub,
                CoreCustodyObject::all_custodians(),
                CoreCustodyAction::CUSTODIAN_LIST,
            )
            .await?;
        Ok(self
            .custodians
            .list_by_name(query, es_entity::ListDirection::Ascending)
            .await?)
    }

    #[record_error_severity]
    #[instrument(name = "custody.create_wallet_in_op", skip(self, db))]
    pub async fn create_wallet_in_op(
        &self,
        db: &mut DbOp<'_>,
        custodian_id: CustodianId,
        wallet_label: &str,
    ) -> Result<Wallet, CoreCustodyError> {
        let custodian = self
            .custodians
            .find_by_id_in_op(&mut *db, &custodian_id)
            .await?;

        let client = custodian
            .custodian_client(&self.encryption_config.key, &self.config.custody_providers)?;

        let external_wallet = client.initialize_wallet(wallet_label).await?;

        let new_wallet = NewWallet::builder()
            .id(WalletId::new())
            .custodian_id(custodian_id)
            .external_wallet_id(external_wallet.external_id)
            .custodian_response(external_wallet.full_response)
            .address(external_wallet.address)
            .network(external_wallet.network)
            .build()
            .expect("all fields for new wallet provided");

        let wallet = self.wallets.create_in_op(db, new_wallet).await?;

        Ok(wallet)
    }

    #[record_error_severity]
    #[instrument(name = "custody.handle_webhook", skip(self))]
    pub async fn handle_webhook(
        &self,
        provider: String,
        uri: http::Uri,
        headers: http::HeaderMap,
        payload: bytes::Bytes,
    ) -> Result<(), CoreCustodyError> {
        let idempotency_key = self.extract_idempotency_key(&headers);

        let headers_map: HashMap<String, String> = headers
            .iter()
            .map(|(name, value)| {
                (
                    name.as_str().to_owned(),
                    value.to_str().unwrap_or("<unreadable>").to_owned(),
                )
            })
            .collect();

        let webhook_payload = WebhookPayload {
            provider,
            uri: uri.to_string(),
            headers: headers_map,
            payload,
        };

        let _res = self
            .inbox
            .persist_and_queue_job(&idempotency_key, webhook_payload)
            .await?;

        Ok(())
    }

    fn extract_idempotency_key(&self, headers: &http::HeaderMap) -> String {
        const IDEMPOTENCY_HEADER_KEYS: &[&str] = &[
            "idempotency-key",
            "x-komainu-signature",
            "x-signature-sha256",
        ];

        for key in IDEMPOTENCY_HEADER_KEYS {
            if let Some(value) = headers.get(*key).and_then(|v| v.to_str().ok()) {
                return value.to_owned();
            }
        }

        // Fallback: hash all headers
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        let mut sorted_headers: Vec<_> = headers
            .iter()
            .map(|(k, v)| (k.as_str(), v.to_str().unwrap_or("")))
            .collect();
        sorted_headers.sort_by_key(|(k, _)| *k);
        for (key, value) in sorted_headers {
            hasher.update(format!("{key}:{value}\n"));
        }
        format!("{:x}", hasher.finalize())
    }
}

impl<Perms, E> Clone for CoreCustody<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustodyEvent>,
{
    fn clone(&self) -> Self {
        Self {
            authz: self.authz.clone(),
            custodians: self.custodians.clone(),
            wallets: self.wallets.clone(),
            encryption_config: self.encryption_config.clone(),
            config: self.config.clone(),
            inbox: self.inbox.clone(),
            clock: self.clock.clone(),
        }
    }
}
