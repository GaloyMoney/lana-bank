mod csv;
mod entity;
mod primitives;
mod repo;

use audit::AuditSvc;
use authz::PermissionCheck;

use cala_ledger::{account_set::NewAccountSet, CalaLedger};
use tracing::instrument;

use super::error::*;

pub(crate) use csv::CsvParseError;
use entity::*;
use primitives::*;
use repo::*;

pub struct CoreChartOfAccounts<Perms>
where
    Perms: PermissionCheck,
{
    repo: ChartRepo,
    cala: CalaLedger,
    authz: Perms,
    journal_id: LedgerJournalId,
}

impl<Perms> Clone for CoreChartOfAccounts<Perms>
where
    Perms: PermissionCheck,
{
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
            cala: self.cala.clone(),
            authz: self.authz.clone(),
            journal_id: self.journal_id,
        }
    }
}

impl<Perms> CoreChartOfAccounts<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreChartOfAccountsAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreChartOfAccountsObject>,
{
    pub async fn init(
        pool: &sqlx::PgPool,
        authz: &Perms,
        cala: &CalaLedger,
        journal_id: LedgerJournalId,
    ) -> Result<Self, CoreChartOfAccountsError> {
        let chart_of_account = ChartRepo::new(pool);
        let res = Self {
            repo: chart_of_account,
            cala: cala.clone(),
            authz: authz.clone(),
            journal_id,
        };
        Ok(res)
    }

    #[instrument(name = "chart_of_account.import_from_csv", skip(self))]
    pub async fn import_from_csv(
        &self,
        id: impl Into<ChartId> + std::fmt::Debug,
        data: String,
    ) -> Result<(), CoreChartOfAccountsError> {
        // Fix audit
        let id = id.into();
        let audit_info = self
            .authz
            .audit()
            .record_system_entry(
                CoreChartOfAccountsObject::chart(id),
                CoreChartOfAccountsAction::CHART_LIST,
            )
            .await?;
        let mut chart = self.repo.find_by_id(id).await?;

        let account_specs = csv::CsvParser::new(data).account_specs()?;
        for account_spec in account_specs {
            if !account_spec.has_parent() {
                chart.create_control_account(account_spec.category, audit_info.clone());
                // create account set for control account
            } else {
                chart.create_control_sub_account(account_spec.category, audit_info.clone());
                // get the control account for control sub account from cala
                // create account set for control sub account
                // add control sub account to control account
            }
        }

        Ok(())
    }

    // #[instrument(name = "chart_of_accounts.create_chart", skip(self))]
    // pub async fn create_chart( &self,
    //     id: impl Into<ChartId> + std::fmt::Debug,
    //     name: String,
    //     reference: String,
    // ) -> Result<Chart, CoreChartOfAccountsError> {
    //     let id = id.into();

    //     let mut op = self.repo.begin_op().await?;
    //     let audit_info = self
    //         .authz
    //         .audit()
    //         .record_system_entry_in_tx(
    //             op.tx(),
    //             CoreChartOfAccountsObject::chart(id),
    //             CoreChartOfAccountsAction::CHART_CREATE,
    //         )
    //         .await?;

    //     let new_chart_of_account = NewChart::builder()
    //         .id(id)
    //         .name(name)
    //         .reference(reference)
    //         .audit_info(audit_info)
    //         .build()
    //         .expect("Could not build new chart of accounts");

    //     let chart = self
    //         .repo
    //         .create_in_op(&mut op, new_chart_of_account)
    //         .await?;
    //     op.commit().await?;

    //     Ok(chart)
    // }
}
