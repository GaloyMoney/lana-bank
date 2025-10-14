mod chart_of_accounts_integration;
mod entity;
mod ledger;
mod primitives;
mod repo;

pub mod error;
pub use chart_of_accounts_integration::ChartOfAccountsIntegrationConfig;

use chrono::Utc;

use audit::AuditSvc;
use authz::PermissionCheck;
use cala_ledger::{CalaLedger, JournalId};
use ledger::{AnnualClosingTransactionLedger, AnnualClosingTransactionParams, EntryParams};

use crate::{
    Chart,
    chart_of_accounts::ChartOfAccounts,
    primitives::{
        AnnualClosingTransactionId, CalaAccountSetId, CalaTxId, ChartId, CoreAccountingAction,
        CoreAccountingObject,
    },
};

use error::*;
use repo::*;

pub use entity::AnnualClosingTransaction;
#[cfg(feature = "json-schema")]
pub use entity::AnnualClosingTransactionEvent;
pub(super) use entity::*;
pub use repo::annual_closing_transaction_cursor::AnnualClosingTransactionsByCreatedAtCursor;

pub(crate) const REVENUE_NAME: &str = "Revenue";
pub(crate) const EXPENSES_NAME: &str = "Expenses";
pub(crate) const COST_OF_REVENUE_NAME: &str = "Cost of Revenue";
pub(crate) const EQUITY_RETAINED_EARNINGS_NAME: &str = "Retained Earnings";
pub(crate) const EQUITY_RETAINED_LOSSES_NAME: &str = "Retained Losses";
#[derive(Clone, Copy)]
pub struct AnnualClosingIds {
    pub id: CalaAccountSetId,
    pub revenue: CalaAccountSetId,
    pub cost_of_revenue: CalaAccountSetId,
    pub expenses: CalaAccountSetId,
    pub equity_retained_earnings: CalaAccountSetId,
    pub equity_retained_losses: CalaAccountSetId,
}

#[derive(Clone)]
pub struct AnnualClosingTransactions<Perms>
where
    Perms: PermissionCheck,
{
    ledger: AnnualClosingTransactionLedger,
    authz: Perms,
    chart_of_accounts: ChartOfAccounts<Perms>,
    journal_id: JournalId,
    repo: AnnualClosingTransactionRepo,
}

impl<Perms> AnnualClosingTransactions<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreAccountingAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreAccountingObject>,
{
    pub fn new(
        pool: &sqlx::PgPool,
        authz: &Perms,
        chart_of_accounts: &ChartOfAccounts<Perms>,
        cala: &CalaLedger,
        journal_id: JournalId,
    ) -> Self {
        let repo = AnnualClosingTransactionRepo::new(pool);
        Self {
            ledger: AnnualClosingTransactionLedger::new(cala),
            authz: authz.clone(),
            chart_of_accounts: chart_of_accounts.clone(),
            journal_id,
            repo,
        }
    }

    pub async fn execute(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart_id: ChartId,
        reference: Option<String>,
        description: String,
    ) -> Result<AnnualClosingTransaction, AnnualClosingTransactionError> {
        // TODO: authz permissions (follow ManualTransactions).
        let effective = Utc::now();
        let ledger_tx_id: CalaTxId = CalaTxId::new();
        let closing_tx_id: AnnualClosingTransactionId = AnnualClosingTransactionId::new();

        let new_tx = NewAnnualClosingTransaction::builder()
            .id(closing_tx_id)
            .ledger_transaction_id(ledger_tx_id)
            .description(description.clone())
            .reference(reference)
            .build()
            .expect("Couldn't build new annual closing transaction");

        let mut db = self.repo.begin_op().await?;
        let annual_closing_transaction = self.repo.create_in_op(&mut db, new_tx).await?;
        let entries = self
            .chart_of_accounts
            .create_annual_closing_entries(effective, chart_id)
            .await?;

        let entry_params = entries.into_iter().map(EntryParams::from).collect();
        self.ledger
            .execute(
                db,
                ledger_tx_id,
                AnnualClosingTransactionParams {
                    journal_id: self.journal_id,
                    description,
                    entry_params,
                    effective: effective.date_naive(),
                },
            )
            .await?;

        Ok(annual_closing_transaction)
    }

    pub async fn get_chart_of_accounts_integration_config(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart: &Chart,
    ) -> Result<Option<ChartOfAccountsIntegrationConfig>, AnnualClosingTransactionError> {
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_annual_closing_transaction_configuration(),
                CoreAccountingAction::ANNUAL_CLOSING_TRANSACTION_CONFIGURATION_READ,
            )
            .await?;

        Ok(self
            .ledger
            .get_chart_of_accounts_integration_config(chart.id)
            .await?)
    }

    pub async fn set_chart_of_accounts_integration_config(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart: &Chart,
        config: ChartOfAccountsIntegrationConfig,
    ) -> Result<ChartOfAccountsIntegrationConfig, AnnualClosingTransactionError> {
        if chart.id != config.chart_of_accounts_id {
            return Err(AnnualClosingTransactionError::ChartIdMismatch);
        }

        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_annual_closing_transaction_configuration(),
                CoreAccountingAction::ANNUAL_CLOSING_TRANSACTION_CONFIGURATION_UPDATE,
            )
            .await?;

        if self
            .ledger
            .get_chart_of_accounts_integration_config(chart.id)
            .await?
            .is_some()
        {
            return Err(AnnualClosingTransactionError::AnnualClosingTransactionIntegrationConfigAlreadyExists);
        }

        todo!();

        // let revenue_child_account_set_id_from_chart =
        //     chart.account_set_id_from_code(&config.chart_of_accounts_revenue_code)?;
        // let cost_of_revenue_child_account_set_id_from_chart =
        //     chart.account_set_id_from_code(&config.chart_of_accounts_cost_of_revenue_code)?;
        // let expenses_child_account_set_id_from_chart =
        //     chart.account_set_id_from_code(&config.chart_of_accounts_expenses_code)?;
        //
        // let charts_integration_meta = ChartOfAccountsIntegrationMeta {
        //     audit_info,
        //     config: config.clone(),
        //
        //     revenue_child_account_set_id_from_chart,
        //     cost_of_revenue_child_account_set_id_from_chart,
        //     expenses_child_account_set_id_from_chart,
        // };
        //
        // self.pl_statement_ledger
        //     .attach_chart_of_accounts_account_sets(reference, charts_integration_meta)
        //     .await?;
        //
        // Ok(config)
    }
}
