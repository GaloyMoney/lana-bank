mod entity;
pub mod error;
mod repo;

use crate::{
    entity::*,
    ledger::Ledger,
    primitives::{BfxIntegrationId, LedgerAccountId, LedgerAccountSetId},
};

pub use entity::*;
use error::BfxIntegrationError;
pub use repo::BfxIntegrationRepo;

pub struct BfxIntegrationOmnibusAccountSetIdsForLedger {
    pub off_balance_sheet: LedgerAccountSetId,
    pub usdt_cash: LedgerAccountSetId,
}

#[derive(Clone)]
pub struct BfxIntegrations {
    _pool: sqlx::PgPool,
    repo: BfxIntegrationRepo,
    ledger: Ledger,
}

impl BfxIntegrations {
    pub fn new(pool: &sqlx::PgPool, ledger: &Ledger) -> Self {
        let repo = BfxIntegrationRepo::new(pool);
        Self {
            _pool: pool.clone(),
            repo,
            ledger: ledger.clone(),
        }
    }

    pub fn repo(&self) -> &BfxIntegrationRepo {
        &self.repo
    }

    pub async fn create_bfx_integration(
        &self,
        id: BfxIntegrationId,
        omnibus_account_set_id: LedgerAccountSetId,
        withdrawal_account_id: LedgerAccountId,
    ) -> Result<BfxIntegration, BfxIntegrationError> {
        let new_bfx_integration = NewBfxIntegration::builder()
            .id(id)
            .omnibus_account_set_id(omnibus_account_set_id)
            .withdrawal_account_id(withdrawal_account_id)
            .build()
            .expect("Could not build BfxIntegration");

        let EntityUpdate {
            entity: bfx_integration,
            ..
        } = self.repo.create(new_bfx_integration).await?;
        Ok(bfx_integration)
    }

    pub async fn get_omnibus_account_set_ids_for_ledger(
        &self,
    ) -> Result<BfxIntegrationOmnibusAccountSetIdsForLedger, BfxIntegrationError> {
        let integration_ids = self.ledger.bfx_integration_ids();
        let bfx_off_balance_sheet_integration = self
            .repo
            .find_by_id(integration_ids.off_balance_sheet)
            .await?;
        let bfx_usdt_cash_integration = self.repo.find_by_id(integration_ids.usdt_cash).await?;

        Ok(BfxIntegrationOmnibusAccountSetIdsForLedger {
            off_balance_sheet: bfx_off_balance_sheet_integration.omnibus_account_set_id,
            usdt_cash: bfx_usdt_cash_integration.omnibus_account_set_id,
        })
    }
}
