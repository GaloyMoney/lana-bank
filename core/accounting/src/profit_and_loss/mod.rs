pub mod error;
pub mod ledger;

use chrono::NaiveDate;
use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use cala_ledger::CalaLedger;
use tracing_macros::record_error_severity;

use crate::{
    LedgerAccountId,
    primitives::{
        BalanceRange, CalaAccountSetId, CoreAccountingAction, CoreAccountingObject,
        ResolvedAccountingBaseConfig,
    },
};
use error::*;
use ledger::*;

pub(crate) const REVENUE_NAME: &str = "Revenue";
pub(crate) const EXPENSES_NAME: &str = "Expenses";
pub(crate) const COST_OF_REVENUE_NAME: &str = "Cost of Revenue";
#[derive(Clone, Copy)]
pub struct ProfitAndLossStatementIds {
    pub id: CalaAccountSetId,
    pub revenue: CalaAccountSetId,
    pub cost_of_revenue: CalaAccountSetId,
    pub expenses: CalaAccountSetId,
}

#[derive(Clone)]
pub struct ProfitAndLossStatements<Perms>
where
    Perms: PermissionCheck,
{
    pool: sqlx::PgPool,
    authz: Perms,
    pl_statement_ledger: ProfitAndLossStatementLedger,
}

impl<Perms> ProfitAndLossStatements<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreAccountingAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreAccountingObject>,
{
    pub fn new(
        pool: &sqlx::PgPool,
        authz: &Perms,
        cala: &CalaLedger,
        journal_id: cala_ledger::JournalId,
    ) -> Self {
        let pl_statement_ledger = ProfitAndLossStatementLedger::new(cala, journal_id);

        Self {
            pool: pool.clone(),
            pl_statement_ledger,
            authz: authz.clone(),
        }
    }

    #[record_error_severity]
    #[instrument(name = "core_accounting.profit_and_loss.create_pl_statement", skip(self, name), fields(pl_statement_name = %name))]
    pub async fn create_pl_statement(
        &self,
        name: String,
    ) -> Result<(), ProfitAndLossStatementError> {
        let mut op = es_entity::DbOp::init(&self.pool).await?;

        self.authz
            .audit()
            .record_system_entry_in_op(
                &mut op,
                CoreAccountingObject::all_profit_and_loss(),
                CoreAccountingAction::PROFIT_AND_LOSS_CREATE,
            )
            .await?;

        match self.pl_statement_ledger.create_in_op(&mut op, &name).await {
            Ok(_) => {
                op.commit().await?;
                Ok(())
            }
            Err(e) if e.account_set_exists() => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    #[record_error_severity]
    #[instrument(
        name = "core_accounting.profit_and_loss.link_chart_account_sets_in_op",
        skip(self, op, resolved)
    )]
    pub(crate) async fn link_chart_account_sets_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        reference: String,
        resolved: &ResolvedAccountingBaseConfig,
    ) -> Result<(), ledger::error::ProfitAndLossStatementLedgerError> {
        self.pl_statement_ledger
            .attach_chart_of_accounts_account_sets_in_op(op, reference, resolved)
            .await
    }

    #[record_error_severity]
    #[instrument(name = "core_accounting.profit_and_loss.pl_statement", skip(self))]
    pub async fn pl_statement(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        reference: String,
        from: NaiveDate,
        until: Option<NaiveDate>,
    ) -> Result<ProfitAndLossStatement, ProfitAndLossStatementError> {
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_profit_and_loss(),
                CoreAccountingAction::PROFIT_AND_LOSS_READ,
            )
            .await?;

        Ok(self
            .pl_statement_ledger
            .get_pl_statement(reference, from, until)
            .await?)
    }
}

#[derive(Clone)]
pub struct ProfitAndLossStatement {
    pub id: LedgerAccountId,
    pub name: String,
    pub usd_balance_range: Option<BalanceRange>,
    pub btc_balance_range: Option<BalanceRange>,
    pub category_ids: Vec<LedgerAccountId>,
}
