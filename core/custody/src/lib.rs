#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod config;
pub mod custodian;
pub mod error;
mod event;
mod primitives;
mod publisher;
pub mod wallet;
mod webhook_notification_repo;

use strum::IntoDiscriminant as _;
use tracing::instrument;

use es_entity::DbOp;
pub use event::CoreCustodyEvent;
use outbox::{Outbox, OutboxEventMarker};
pub use publisher::CustodyPublisher;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_money::Satoshis;

pub use custodian::*;
pub use wallet::*;
use webhook_notification_repo::*;

pub use config::*;
use error::CoreCustodyError;
pub use primitives::*;

#[cfg(feature = "json-schema")]
pub mod event_schema {
    pub use crate::custodian::CustodianEvent;
}

pub struct CoreCustody<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustodyEvent>,
{
    authz: Perms,
    custodians: CustodianRepo,
    webhooks: WebhookNotificationRepo,
    config: CustodyConfig,
    wallets: WalletRepo<E>,
    pool: sqlx::PgPool,
    outbox: Outbox<E>,
}

impl<Perms, E> CoreCustody<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCustodyEvent>,
{
    pub async fn init(
        pool: &sqlx::PgPool,
        authz: &Perms,
        config: CustodyConfig,
        outbox: &Outbox<E>,
    ) -> Result<Self, CoreCustodyError> {
        let custody = Self {
            authz: authz.clone(),
            custodians: CustodianRepo::new(pool),
            webhooks: WebhookNotificationRepo::new(pool),
            config,
            wallets: WalletRepo::new(pool, &CustodyPublisher::new(outbox)),
            pool: pool.clone(),
            outbox: outbox.clone(),
        };

        if let Some(deprecated_encryption_key) = custody.config.deprecated_encryption_key.as_ref() {
            custody
                .rotate_encryption_key(deprecated_encryption_key)
                .await?;
        }

        Ok(custody)
    }

    #[cfg(feature = "mock-custodian")]
    #[instrument(
        name = "credit_facility.ensure_mock_custodian_in_op",
        skip(self, db),
        err
    )]
    pub async fn ensure_mock_custodian_in_op(
        &self,
        db: &mut DbOp<'_>,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
    ) -> Result<(), CoreCustodyError> {
        let mock_custodian = self
            .find_custodian_by_id(sub, CustodianId::mock_custodian_id())
            .await;

        match mock_custodian {
            Err(CoreCustodyError::Custodian(e)) if e.was_not_found() => {
                let _ = self
                    .create_custodian_in_op(db, sub, "Mock Custodian", CustodianConfig::Mock)
                    .await?;

                Ok(())
            }
            Err(e) => Err(e),
            Ok(_) => Ok(()),
        }
    }

    #[instrument(name = "core_custody.created_custodian_in_op", skip(self, db), err)]
    pub async fn create_custodian_in_op(
        &self,
        db: &mut DbOp<'_>,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        name: impl AsRef<str> + std::fmt::Debug,
        custodian_config: CustodianConfig,
    ) -> Result<Custodian, CoreCustodyError> {
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreCustodyObject::all_custodians(),
                CoreCustodyAction::CUSTODIAN_CREATE,
            )
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
            .name(name.as_ref().to_owned())
            .provider(custodian_config.discriminant().to_string())
            .encrypted_custodian_config(custodian_config, &self.config.encryption.key)
            .audit_info(audit_info.clone())
            .build()
            .expect("should always build a new custodian");

        let custodian = self.custodians.create_in_op(db, new_custodian).await?;

        Ok(custodian)
    }

    #[instrument(
        name = "core_custody.create_custodian",
        skip(self, custodian_config),
        err
    )]
    pub async fn create_custodian(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        name: impl AsRef<str> + std::fmt::Debug,
        custodian_config: CustodianConfig,
    ) -> Result<Custodian, CoreCustodyError> {
        let mut db = self.custodians.begin_op().await?;

        let custodian = self
            .create_custodian_in_op(&mut db, sub, name, custodian_config)
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
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreCustodyObject::custodian(id),
                CoreCustodyAction::CUSTODIAN_UPDATE,
            )
            .await?;
        let mut custodian = self.custodians.find_by_id(id).await?;

        custodian.update_custodian_config(config, &self.config.encryption.key, audit_info);

        let mut op = self.custodians.begin_op().await?;
        self.custodians
            .update_config_in_op(&mut op, &mut custodian)
            .await?;
        op.commit().await?;

        Ok(custodian)
    }

    async fn rotate_encryption_key(
        &self,
        deprecated_encryption_key: &DeprecatedEncryptionKey,
    ) -> Result<(), CoreCustodyError> {
        let audit_info = self
            .authz
            .audit()
            .record_system_entry(
                CoreCustodyObject::all_custodians(),
                CoreCustodyAction::CUSTODIAN_UPDATE,
            )
            .await?;

        let mut custodians = self.custodians.list_all().await?;

        let mut op = self.custodians.begin_op().await?;

        for custodian in custodians.iter_mut() {
            custodian.rotate_encryption_key(
                &self.config.encryption.key,
                deprecated_encryption_key,
                &audit_info,
            )?;

            self.custodians
                .update_config_in_op(&mut op, custodian)
                .await?;
        }

        op.commit().await?;

        Ok(())
    }

    #[instrument(name = "core_custody.find_all_wallets", skip(self), err)]
    pub async fn find_all_wallets<T: From<Wallet>>(
        &self,
        ids: &[WalletId],
    ) -> Result<std::collections::HashMap<WalletId, T>, CoreCustodyError> {
        Ok(self.wallets.find_all(ids).await?)
    }

    #[instrument(name = "core_custody.find_all_custodians", skip(self), err)]
    pub async fn find_all_custodians<T: From<Custodian>>(
        &self,
        ids: &[CustodianId],
    ) -> Result<std::collections::HashMap<CustodianId, T>, CoreCustodyError> {
        Ok(self.custodians.find_all(ids).await?)
    }

    #[instrument(name = "core_custody.find_custodian_by_id", skip(self), err)]
    pub async fn find_custodian_by_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        id: CustodianId,
    ) -> Result<Custodian, CoreCustodyError> {
        self.authz
            .enforce_permission(
                sub,
                CoreCustodyObject::all_custodians(),
                CoreCustodyAction::CUSTODIAN_LIST,
            )
            .await?;

        Ok(self.custodians.find_by_id(id).await?)
    }

    #[instrument(name = "core_custody.list_custodians", skip(self), err)]
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

    #[instrument(name = "custody.create_wallet_in_op", skip(self, db), err)]
    pub async fn create_wallet_in_op(
        &self,
        db: &mut DbOp<'_>,
        audit_info: audit::AuditInfo,
        custodian_id: CustodianId,
    ) -> Result<Wallet, CoreCustodyError> {
        let new_wallet = NewWallet::builder()
            .id(WalletId::new())
            .custodian_id(custodian_id)
            .audit_info(audit_info.clone())
            .build()
            .expect("all fields for new wallet provided");

        let mut wallet = self.wallets.create_in_op(db, new_wallet).await?;

        self.generate_wallet_address_in_op(db, audit_info, &mut wallet)
            .await?;

        Ok(wallet)
    }

    #[instrument(name = "custody.handle_webhook", skip(self), err)]
    pub async fn handle_webhook(
        &self,
        provider: String,
        uri: http::Uri,
        headers: http::HeaderMap,
        payload: bytes::Bytes,
    ) -> Result<(), CoreCustodyError> {
        let custodian = self.custodians.find_by_provider(provider).await;

        let custodian_id = match custodian {
            Err(ref e) if e.was_not_found() => None,
            Ok(ref custodian) => Some(custodian.id),
            Err(e) => return Err(e.into()),
        };

        self.webhooks
            .persist(custodian_id, &uri, &headers, &payload)
            .await?;

        if let Ok(custodian) = custodian {
            if let Some(notification) = custodian
                .custodian_client(self.config.encryption.key)
                .await?
                .process_webhook(&headers, payload)
                .await?
            {
                match notification {
                    CustodianNotification::WalletBalanceChanged {
                        external_wallet_id,
                        new_balance,
                    } => {
                        self.update_wallet_balance(external_wallet_id, new_balance)
                            .await?;
                    }
                }
            }
        }

        Ok(())
    }

    #[instrument(name = "custody.update_wallet_balance", skip(self), err)]
    async fn update_wallet_balance(
        &self,
        external_wallet_id: String,
        new_balance: Satoshis,
    ) -> Result<(), CoreCustodyError> {
        let mut db = self.wallets.begin_op().await?;

        let mut wallet = self
            .wallets
            .find_by_external_wallet_id_in_tx(db.tx(), Some(external_wallet_id))
            .await?;

        let audit_info = self
            .authz
            .audit()
            .record_system_entry_in_tx(
                db.tx(),
                CoreCustodyObject::wallet(wallet.id),
                CoreCustodyAction::WALLET_UPDATE,
            )
            .await?;

        if wallet
            .update_balance(new_balance, &audit_info)
            .did_execute()
        {
            self.wallets.update_in_op(&mut db, &mut wallet).await?;
        }

        db.commit().await?;

        Ok(())
    }

    #[instrument(
        name = "custody.generate_wallet_address_in_op",
        skip(self, db, wallet),
        err
    )]
    async fn generate_wallet_address_in_op(
        &self,
        db: &mut DbOp<'_>,
        audit_info: audit::AuditInfo,
        wallet: &mut Wallet,
    ) -> Result<(), CoreCustodyError> {
        let custodian = self
            .custodians
            .find_by_id_in_tx(db.tx(), &wallet.custodian_id)
            .await?;

        let client = custodian
            .custodian_client(self.config.encryption.key)
            .await?;

        let external_wallet = client.initialize_wallet("label").await?;

        if wallet
            .attach_external_wallet(
                external_wallet.external_id.clone(),
                external_wallet.address,
                external_wallet.full_response,
                &audit_info,
            )
            .did_execute()
        {
            self.wallets.update_in_op(db, wallet).await?;
        };

        Ok(())
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
            webhooks: self.webhooks.clone(),
            wallets: self.wallets.clone(),
            pool: self.pool.clone(),
            config: self.config.clone(),
            outbox: self.outbox.clone(),
        }
    }
}
