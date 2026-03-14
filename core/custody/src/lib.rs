#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod config;
pub mod custodian;
pub mod error;
mod jobs;
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

use es_entity::clock::ClockHandle;
use es_entity::{AtomicOperation, DbOp};
use obix::inbox::{Inbox, InboxConfig, InboxEvent, InboxHandler, InboxResult};
use obix::out::{Outbox, OutboxEventMarker};
pub use public::*;
pub use publisher::CustodyPublisher;

use audit::AuditSvc;
use authz::PermissionCheck;
use encryption::{EncryptionConfig, EncryptionKey};
use old_money::Satoshis;

pub use custodian::*;
pub use wallet::*;

pub use config::CustodyConfig;
use error::CoreCustodyError;
use jobs::self_custody_balance_sync;
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
                .custodian_client(
                    &self.encryption_config.encryption_key,
                    &self.config.custody_providers,
                )?
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

        let self_custody_balance_sync_job_spawner = jobs.add_initializer(
            self_custody_balance_sync::SelfCustodyBalanceSyncJobInit::new(
                &custody.authz,
                &custody.custodians,
                &custody.wallets,
                &custody.encryption_config,
                &custody.config,
            ),
        );
        self_custody_balance_sync_job_spawner
            .spawn_unique(
                job::JobId::new(),
                self_custody_balance_sync::SelfCustodyBalanceSyncJobConfig::<E> {
                    _phantom: std::marker::PhantomData,
                },
            )
            .await?;

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
            .encrypted_custodian_config(custodian_config, &self.encryption_config.encryption_key)
            .build()
            .expect("should always build a new custodian");

        let custodian = self.custodians.create_in_op(db, new_custodian).await?;

        Ok(custodian)
    }

    pub async fn subject_can_create_custodian(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        enforce: bool,
    ) -> Result<Option<audit::AuditInfo>, CoreCustodyError> {
        Ok(self
            .authz
            .evaluate_permission(
                sub,
                CoreCustodyObject::all_custodians(),
                CoreCustodyAction::CUSTODIAN_CREATE,
                enforce,
            )
            .await?)
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
        self.subject_can_create_custodian(sub, true)
            .await?
            .expect("audit info missing");

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
            .encrypted_custodian_config(custodian_config, &self.encryption_config.encryption_key)
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
            .update_custodian_config(&self.encryption_config.encryption_key, config)?
            .did_execute()
        {
            self.custodians
                .update_in_op(&mut op, &mut custodian)
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
                .rotate_encryption_key(&self.encryption_config.encryption_key, deprecated_key)?
                .did_execute()
            {
                self.custodians.update_in_op(&mut op, custodian).await?;
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
    #[instrument(name = "core_custody.find_all_custodians_authorized", skip(self))]
    pub async fn find_all_custodians_authorized<T: From<Custodian>>(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        ids: &[CustodianId],
    ) -> Result<HashMap<CustodianId, T>, CoreCustodyError> {
        self.authz
            .enforce_permission(
                sub,
                CoreCustodyObject::all_custodians(),
                CoreCustodyAction::CUSTODIAN_LIST,
            )
            .await?;
        Ok(self.custodians.find_all(ids).await?)
    }

    #[record_error_severity]
    #[instrument(name = "core_custody.list_custodians", skip(self))]
    pub async fn list_custodians(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        query: es_entity::PaginatedQueryArgs<custodian_cursor::CustodiansCursor>,
        sort: es_entity::Sort<CustodiansSortBy>,
    ) -> Result<
        es_entity::PaginatedQueryRet<Custodian, custodian_cursor::CustodiansCursor>,
        CoreCustodyError,
    > {
        self.authz
            .enforce_permission(
                sub,
                CoreCustodyObject::all_custodians(),
                CoreCustodyAction::CUSTODIAN_LIST,
            )
            .await?;
        Ok(self
            .custodians
            .list_for_filters(Default::default(), sort, query)
            .await?)
    }

    #[record_error_severity]
    #[instrument(name = "custody.create_wallet_in_op", skip(self, db))]
    pub async fn create_wallet_in_op(
        &self,
        db: &mut DbOp<'_>,
        custodian_id: CustodianId,
        wallet_label: &str,
    ) -> Result<Option<Wallet>, CoreCustodyError> {
        self.lock_custodian_in_op(db, custodian_id).await?;

        let mut custodian = self
            .custodians
            .find_by_id_in_op(&mut *db, &custodian_id)
            .await?;

        let receive_index = custodian
            .prepare_wallet_creation()
            .expect("wallet creation preparation always executes");

        let client = custodian.clone().custodian_client(
            &self.encryption_config.encryption_key,
            &self.config.custody_providers,
        )?;

        let external_wallet = client
            .initialize_wallet(wallet_label, receive_index)
            .await?;

        if receive_index.is_some() {
            self.custodians.update_in_op(db, &mut custodian).await?;
        }

        if let Some(external_wallet) = external_wallet {
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
            Ok(Some(wallet))
        } else {
            Ok(None)
        }
    }

    async fn lock_custodian_in_op(
        &self,
        db: &mut DbOp<'_>,
        custodian_id: CustodianId,
    ) -> Result<(), CoreCustodyError> {
        sqlx::query("SELECT id FROM core_custodians WHERE id = $1 FOR UPDATE")
            .bind(custodian_id)
            .execute(db.as_executor())
            .await?;
        Ok(())
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

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

    use axum::{Router, routing::get};
    use es_entity::clock::{ArtificialClockConfig, ClockHandle};
    use serde::{Deserialize, Serialize};

    use super::*;

    #[derive(Debug, Serialize, Deserialize, obix::OutboxEvent)]
    #[serde(tag = "module")]
    enum DummyEvent {
        CoreCustody(CoreCustodyEvent),
        #[serde(other)]
        Unknown,
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    struct DummyAction;

    impl From<CoreCustodyAction> for DummyAction {
        fn from(_: CoreCustodyAction) -> Self {
            Self
        }
    }

    impl std::fmt::Display for DummyAction {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "dummy")
        }
    }

    impl std::str::FromStr for DummyAction {
        type Err = strum::ParseError;

        fn from_str(_: &str) -> Result<Self, Self::Err> {
            Ok(Self)
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    struct DummyObject;

    impl From<CoreCustodyObject> for DummyObject {
        fn from(_: CoreCustodyObject) -> Self {
            Self
        }
    }

    impl std::fmt::Display for DummyObject {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "dummy")
        }
    }

    impl std::str::FromStr for DummyObject {
        type Err = &'static str;

        fn from_str(_: &str) -> Result<Self, Self::Err> {
            Ok(Self)
        }
    }

    type DummyPerms = authz::dummy::DummyPerms<DummyAction, DummyObject>;

    #[tokio::test]
    async fn self_custody_wallets_use_unique_receive_addresses() -> anyhow::Result<()> {
        let Ok(pg_con) = std::env::var("PG_CON") else {
            return Ok(());
        };

        let pool = sqlx::PgPool::connect(&pg_con).await?;
        let (clock, _time) = ClockHandle::artificial(ArtificialClockConfig::manual());
        let outbox = obix::Outbox::<DummyEvent>::init(
            &pool,
            obix::MailboxConfig::builder()
                .clock(clock.clone())
                .build()?,
        )
        .await?;
        let mut jobs = job::Jobs::init(
            job::JobSvcConfig::builder()
                .pool(pool.clone())
                .build()
                .unwrap(),
        )
        .await?;
        let authz = DummyPerms::new();
        let server = TestServer::spawn().await;

        let custody = CoreCustody::init(
            &pool,
            &authz,
            encryption::EncryptionConfig {
                encryption_key: encryption::EncryptionKey::new([9u8; 32]),
                ..Default::default()
            },
            CustodyConfig {
                custody_providers: CustodyProviderConfig {
                    self_custody_directory: SelfCustodyDirectoryConfig {
                        testnet4_url: Some(server.base_url.clone()),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
            &outbox,
            &mut jobs,
            clock,
        )
        .await?;
        let generated = self_custody::generate_account_keys(SelfCustodyNetwork::Testnet4)?;

        let custodian = custody
            .create_custodian(
                &authz::dummy::DummySubject,
                "Self Custody",
                CustodianConfig::SelfCustody(SelfCustodyConfig {
                    account_xpub: generated.account_xpub,
                    network: SelfCustodyNetwork::Testnet4,
                }),
            )
            .await?;

        let mut op = custody.custodians.begin_op().await?;
        let first = custody
            .create_wallet_in_op(&mut op, custodian.id, "Loan 1")
            .await?
            .unwrap();
        let second = custody
            .create_wallet_in_op(&mut op, custodian.id, "Loan 2")
            .await?
            .unwrap();
        op.commit().await?;

        assert_ne!(first.address, second.address);
        assert_eq!(first.external_wallet_id, "self-custody:0");
        assert_eq!(second.external_wallet_id, "self-custody:1");

        server.shutdown().await;
        Ok(())
    }

    struct TestServer {
        base_url: url::Url,
        shutdown: tokio::sync::oneshot::Sender<()>,
        handle: tokio::task::JoinHandle<()>,
    }

    impl TestServer {
        async fn spawn() -> Self {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
                .await
                .expect("listener binds");
            let addr: SocketAddr = listener.local_addr().expect("listener has local addr");
            let base_url = url::Url::parse(&format!("http://{addr}")).expect("url parses");
            let app = Router::new().route("/blocks/tip/height", get(|| async { "42" }));
            let (shutdown, shutdown_rx) = tokio::sync::oneshot::channel();
            let handle = tokio::spawn(async move {
                let _ = axum::serve(listener, app)
                    .with_graceful_shutdown(async {
                        let _ = shutdown_rx.await;
                    })
                    .await;
            });

            Self {
                base_url,
                shutdown,
                handle,
            }
        }

        async fn shutdown(self) {
            let _ = self.shutdown.send(());
            let _ = self.handle.await;
        }
    }
}
