mod csv;
mod entity;
pub mod error;
mod repo;
pub mod tree;

use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;

use cala_ledger::{CalaLedger, account::Account};

use crate::primitives::{
    AccountIdOrCode, AccountSpec, CalaAccountSetId, CalaJournalId, ChartId, CoreAccountingAction,
    CoreAccountingObject, LedgerAccountId,
};

pub(super) use csv::{CsvParseError, CsvParser};
pub use entity::Chart;
#[cfg(feature = "json-schema")]
pub use entity::ChartEvent;
pub(super) use entity::*;
use error::*;
pub(super) use repo::*;

pub struct ChartOfAccounts<Perms>
where
    Perms: PermissionCheck,
{
    repo: ChartRepo,
    cala: CalaLedger,
    authz: Perms,
    journal_id: CalaJournalId,
}

impl<Perms> Clone for ChartOfAccounts<Perms>
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

impl<Perms> ChartOfAccounts<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreAccountingAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreAccountingObject>,
{
    pub fn new(
        pool: &sqlx::PgPool,
        authz: &Perms,
        cala: &CalaLedger,
        journal_id: CalaJournalId,
    ) -> Self {
        let chart_of_account = ChartRepo::new(pool);
        Self {
            repo: chart_of_account,
            cala: cala.clone(),
            authz: authz.clone(),
            journal_id,
        }
    }

    #[instrument(
        name = "core_accounting.chart_of_accounts.create_chart",
        skip(self),
        err
    )]
    pub async fn create_chart(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        name: String,
        reference: String,
    ) -> Result<Chart, ChartOfAccountsError> {
        let id = ChartId::new();

        let mut op = self.repo.begin_op().await?;
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreAccountingObject::chart(id),
                CoreAccountingAction::CHART_CREATE,
            )
            .await?;

        let new_chart = NewChart::builder()
            .id(id)
            .name(name)
            .reference(reference)
            .audit_info(audit_info)
            .build()
            .expect("Could not build new chart of accounts");

        let chart = self.repo.create_in_op(&mut op, new_chart).await?;
        op.commit().await?;

        Ok(chart)
    }

    #[instrument(
        name = "core_accounting.chart_of_accounts.import_from_csv",
        skip(self, data),
        err
    )]
    pub async fn import_from_csv(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        id: impl Into<ChartId> + std::fmt::Debug,
        data: impl AsRef<str>,
    ) -> Result<(Chart, Option<Vec<CalaAccountSetId>>), ChartOfAccountsError> {
        let id = id.into();
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreAccountingObject::chart(id),
                CoreAccountingAction::CHART_IMPORT_ACCOUNTS,
            )
            .await?;
        let mut chart = self.repo.find_by_id(id).await?;

        let data = data.as_ref().to_string();
        let account_specs = CsvParser::new(data).account_specs()?;
        let mut new_account_sets = Vec::new();
        let mut new_connections = Vec::new();
        for spec in account_specs {
            if let es_entity::Idempotent::Executed(NewChartAccountDetails {
                parent_account_set_id,
                new_account_set,
            }) = chart.create_node(&spec, self.journal_id, audit_info.clone())
            {
                let account_set_id = new_account_set.id;
                new_account_sets.push(new_account_set);
                if let Some(parent) = parent_account_set_id {
                    new_connections.push((parent, account_set_id));
                }
            }
        }
        let new_account_set_ids = new_account_sets.iter().map(|a| a.id).collect::<Vec<_>>();
        if new_account_sets.is_empty() {
            return Ok((chart, None));
        }

        let mut op = self.repo.begin_op().await?;
        self.repo.update_in_op(&mut op, &mut chart).await?;

        let mut op = self.cala.ledger_operation_from_db_op(op);
        self.cala
            .account_sets()
            .create_all_in_op(&mut op, new_account_sets)
            .await?;

        for (parent, child) in new_connections {
            self.cala
                .account_sets()
                .add_member_in_op(&mut op, parent, child)
                .await?;
        }
        op.commit().await?;

        let new_account_set_ids = &chart
            .trial_balance_account_ids_from_new_accounts(&new_account_set_ids)
            .collect::<Vec<_>>();

        Ok((chart, Some(new_account_set_ids.clone())))
    }

    #[instrument(name = "core_accounting.chart_of_accounts.add_node", skip(self,), err)]
    pub async fn add_node(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        id: impl Into<ChartId> + std::fmt::Debug,
        spec: impl Into<AccountSpec> + std::fmt::Debug,
    ) -> Result<(Chart, Option<CalaAccountSetId>), ChartOfAccountsError> {
        let id = id.into();
        let spec = spec.into();
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreAccountingObject::chart(id),
                CoreAccountingAction::CHART_UPDATE,
            )
            .await?;
        let mut chart = self.repo.find_by_id(id).await?;

        let es_entity::Idempotent::Executed(NewChartAccountDetails {
            parent_account_set_id,
            new_account_set,
        }) = chart.create_node(&spec, self.journal_id, audit_info.clone())
        else {
            return Ok((chart, None));
        };
        let account_set_id = new_account_set.id;

        let mut op = self.repo.begin_op().await?;
        self.repo.update_in_op(&mut op, &mut chart).await?;

        let mut op = self.cala.ledger_operation_from_db_op(op);
        self.cala
            .account_sets()
            .create_in_op(&mut op, new_account_set)
            .await?;
        if let Some(parent) = parent_account_set_id {
            self.cala
                .account_sets()
                .add_member_in_op(&mut op, parent, account_set_id)
                .await?;
        }

        op.commit().await?;

        let new_account_set_id = chart.trial_balance_account_id_from_new_account(account_set_id);
        Ok((chart, new_account_set_id))
    }

    #[instrument(name = "core_accounting.chart_of_accounts.find_by_id", skip(self), err)]
    pub async fn find_by_id(
        &self,
        id: impl Into<ChartId> + std::fmt::Debug,
    ) -> Result<Chart, ChartOfAccountsError> {
        self.repo.find_by_id(id.into()).await
    }

    #[instrument(
        name = "core_accounting.chart_of_accounts.find_by_reference_with_sub",
        skip(self),
        err
    )]
    pub async fn find_by_reference_with_sub(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        reference: &str,
    ) -> Result<Option<Chart>, ChartOfAccountsError> {
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_charts(),
                CoreAccountingAction::CHART_LIST,
            )
            .await?;

        self.find_by_reference(reference).await
    }

    #[instrument(
        name = "core_accounting.chart_of_accounts.find_by_reference",
        skip(self),
        err
    )]
    pub async fn find_by_reference(
        &self,
        reference: &str,
    ) -> Result<Option<Chart>, ChartOfAccountsError> {
        let reference = reference.to_string();
        let chart = match self.repo.find_by_reference(reference).await {
            Ok(chart) => Some(chart),
            Err(e) if e.was_not_found() => None,
            Err(e) => return Err(e),
        };

        Ok(chart)
    }

    #[instrument(name = "core_accounting.chart_of_accounts.find_all", skip(self), err)]
    pub async fn find_all<T: From<Chart>>(
        &self,
        ids: &[ChartId],
    ) -> Result<std::collections::HashMap<ChartId, T>, ChartOfAccountsError> {
        self.repo.find_all(ids).await
    }

    #[instrument(
        name = "core_accounting.chart_of_accounts.manual_transaction_account_id_for_account_id_or_code",
        skip(self),
        err
    )]
    pub async fn manual_transaction_account_id_for_account_id_or_code(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart_ref: &str,
        account_id_or_code: AccountIdOrCode,
    ) -> Result<LedgerAccountId, ChartOfAccountsError> {
        let mut chart = self.repo.find_by_reference(chart_ref.to_string()).await?;

        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_charts(),
                CoreAccountingAction::CHART_UPDATE,
            )
            .await?;

        let manual_transaction_account_id = match chart
            .manual_transaction_account(account_id_or_code, audit_info)?
        {
            ManualAccountFromChart::IdInChart(id) | ManualAccountFromChart::NonChartId(id) => id,
            ManualAccountFromChart::NewAccount((account_set_id, new_account)) => {
                let mut op = self.repo.begin_op().await?;
                self.repo.update_in_op(&mut op, &mut chart).await?;

                let mut op = self.cala.ledger_operation_from_db_op(op);
                let Account {
                    id: manual_transaction_account_id,
                    ..
                } = self
                    .cala
                    .accounts()
                    .create_in_op(&mut op, new_account)
                    .await?;

                self.cala
                    .account_sets()
                    .add_member_in_op(&mut op, account_set_id, manual_transaction_account_id)
                    .await?;

                op.commit().await?;

                manual_transaction_account_id.into()
            }
        };

        Ok(manual_transaction_account_id)
    }
}
