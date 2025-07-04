#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod config;
pub mod custodian;
pub mod error;
mod event;
mod primitives;
mod publisher;
pub mod wallet;

use es_entity::DbOp;
pub use event::CoreCustodyEvent;
use outbox::{Outbox, OutboxEventMarker};
pub use publisher::CustodyPublisher;
use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;

use custodian::state::repo::CustodianStateRepo;
pub use custodian::*;
pub use wallet::*;

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
    config: CustodyConfig,
    wallets: WalletRepo<E>,
    pool: sqlx::PgPool,
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
            config,
            wallets: WalletRepo::new(pool, &CustodyPublisher::new(outbox)),
            pool: pool.clone(),
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
            .audit_info(audit_info.clone())
            .encrypted_custodian_config(custodian_config, &self.config.custodian_encryption.key)
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

        custodian.update_custodian_config(
            config,
            &self.config.custodian_encryption.key,
            audit_info,
        );

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
                &self.config.custodian_encryption.key,
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

    #[instrument(name = "core_custody.find_all_custodians", skip(self), err)]
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

    pub async fn create_new_wallet_in_op(
        &self,
        db: &mut DbOp<'_>,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        custodian_id: CustodianId,
    ) -> Result<Wallet, CoreCustodyError> {
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreCustodyObject::custodian(custodian_id),
                CoreCustodyAction::CUSTODIAN_CREATE_WALLET,
            )
            .await?;

        let new_wallet = NewWallet::builder()
            .id(WalletId::new())
            .custodian_id(custodian_id)
            .audit_info(audit_info)
            .build()
            .expect("all fields for new wallet provided");

        let mut wallet = self.wallets.create_in_op(db, new_wallet).await?;

        self.generate_wallet_address_in_op(db, sub, &mut wallet)
            .await?;

        Ok(wallet)
    }

    async fn generate_wallet_address_in_op(
        &self,
        db: &mut DbOp<'_>,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        wallet: &mut Wallet,
    ) -> Result<(), CoreCustodyError> {
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreCustodyObject::wallet(wallet.id),
                CoreCustodyAction::WALLET_GENERATE_ADDRESS,
            )
            .await?;

        let custodian = self
            .custodians
            .find_by_id_in_tx(db.tx(), &wallet.custodian_id)
            .await?;

        let custodian_id = custodian.id;

        let client = custodian
            .custodian_client(self.config.custodian_encryption.key)
            .await?;
        let address = client
            .create_address(
                &format!("Wallet {}", wallet.id),
                CustodianStateRepo::new(custodian_id, &self.pool),
            )
            .await?;

        if wallet
            .allocate_address(
                address.address,
                address.label,
                address.full_response,
                &audit_info,
            )
            .did_execute()
        {
            self.wallets.update_in_op(db, wallet).await?;
        }

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
            wallets: self.wallets.clone(),
            pool: self.pool.clone(),
            config: self.config.clone(),
        }
    }
}
