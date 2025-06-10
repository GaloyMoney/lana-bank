#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod custodian;
pub mod error;
mod event;
mod primitives;
mod publisher;
pub mod wallet;

use es_entity::DbOp;
pub use event::CoreCustodyEvent;
use outbox::OutboxEventMarker;
pub use publisher::CustodyPublisher;
use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;

pub use custodian::*;
pub use wallet::*;

use error::CoreCustodyError;
pub use primitives::*;

pub struct CoreCustody<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustodyEvent>,
{
    authz: Perms,
    custodians: CustodianRepo,
    wallets: WalletRepo<E>,
}

impl<Perms, E> CoreCustody<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCustodyEvent>,
{
    pub fn new(pool: &sqlx::PgPool, authz: &Perms, publisher: &CustodyPublisher<E>) -> Self {
        Self {
            authz: authz.clone(),
            custodians: CustodianRepo::new(pool),
            wallets: WalletRepo::new(pool, publisher),
        }
    }

    #[instrument(
        name = "core_custody.create_custodian_config",
        skip(self, custodian),
        err
    )]
    pub async fn create_custodian_config(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        name: impl AsRef<str> + std::fmt::Debug,
        custodian: CustodianConfig,
    ) -> Result<Custodian, CoreCustodyError> {
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreCustodyObject::all_custodians(),
                CoreCustodyAction::CUSTODIAN_CREATE,
            )
            .await?;

        let new_custodian = NewCustodian::builder()
            .name(name.as_ref().to_owned())
            .custodian(custodian)
            .audit_info(audit_info)
            .build()
            .expect("all fields provided");

        Ok(self.custodians.create(new_custodian).await?)
    }

    #[instrument(name = "core_custody.find_all_custodians", skip(self), err)]
    pub async fn find_all_custodians<T: From<Custodian>>(
        &self,
        ids: &[CustodianId],
    ) -> Result<std::collections::HashMap<CustodianId, T>, CoreCustodyError> {
        Ok(self.custodians.find_all(ids).await?)
    }

    #[instrument(name = "core_custody.list_custodians", skip(self), err)]
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

        Ok(self.wallets.create_in_op(db, new_wallet).await?)
    }

    pub async fn generate_wallet_address_in_op(
        &self,
        db: &mut DbOp<'_>,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        wallet_id: WalletId,
        label: &str,
    ) -> Result<(), CoreCustodyError> {
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreCustodyObject::wallet(wallet_id),
                CoreCustodyAction::CUSTODIAN_CREATE_WALLET,
            )
            .await?;

        let mut wallet = self.wallets.find_by_id_in_tx(db.tx(), &wallet_id).await?;
        let custodian = self
            .custodians
            .find_by_id_in_tx(db.tx(), &wallet.custodian_id)
            .await?;

        let client = custodian.custodian_client().await?;
        let address = client.create_address(label).await?;

        if wallet
            .allocate_address(
                address.address,
                address.label,
                address.full_response,
                &audit_info,
            )
            .did_execute()
        {
            self.wallets.update_in_op(db, &mut wallet).await?;
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
        }
    }
}
