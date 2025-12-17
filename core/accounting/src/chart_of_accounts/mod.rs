pub mod chart_node;
mod entity;
mod import;

pub mod error;
pub mod ledger;
mod repo;
pub mod tree;

use es_entity::Idempotent;
use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use tracing_macros::record_error_severity;

use cala_ledger::{CalaLedger, account::Account};

use crate::primitives::{
    AccountCode, AccountIdOrCode, AccountName, AccountSpec, CalaAccountSetId, CalaJournalId,
    ChartId, ClosingAccountCodes, ClosingTxDetails, CoreAccountingAction, CoreAccountingObject,
    LedgerAccountId,
};

#[cfg(feature = "json-schema")]
pub use chart_node::ChartNodeEvent;
pub use entity::Chart;
#[cfg(feature = "json-schema")]
pub use entity::ChartEvent;
pub(super) use entity::*;
use error::*;
use import::{
    BulkAccountImport, BulkImportResult,
    csv::{CsvParseError, CsvParser},
};
use ledger::*;
pub(super) use repo::*;

pub struct ChartOfAccounts<Perms>
where
    Perms: PermissionCheck,
{
    repo: ChartRepo,
    chart_ledger: ChartLedger,
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
            chart_ledger: self.chart_ledger.clone(),
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
        let chart_ledger = ChartLedger::new(cala, journal_id);

        Self {
            repo: chart_of_account,
            chart_ledger,
            cala: cala.clone(),
            authz: authz.clone(),
            journal_id,
        }
    }

    #[record_error_severity]
    #[instrument(name = "core_accounting.chart_of_accounts.create_chart", skip(self))]
    pub async fn create_chart(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        name: String,
        reference: String,
    ) -> Result<Chart, ChartOfAccountsError> {
        let id = ChartId::new();

        let mut op = self.repo.begin_op().await?;
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::chart(id),
                CoreAccountingAction::CHART_CREATE,
            )
            .await?;

        let new_chart = NewChart::builder()
            .id(id)
            .account_set_id(id)
            .name(name)
            .reference(reference)
            .build()
            .expect("Could not build new chart of accounts");

        let chart = self.repo.create_in_op(&mut op, new_chart).await?;

        self.chart_ledger
            .create_chart_root_account_set_in_op(op, &chart)
            .await?;

        Ok(chart)
    }

    #[record_error_severity]
    #[instrument(
        name = "core_accounting.chart_of_accounts.import_from_csv",
        skip(self, import_data)
    )]
    pub async fn import_from_csv(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart_ref: &str,
        import_data: impl AsRef<str>,
    ) -> Result<(Chart, Option<Vec<CalaAccountSetId>>), ChartOfAccountsError> {
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_charts(),
                CoreAccountingAction::CHART_IMPORT_ACCOUNTS,
            )
            .await?;
        let mut chart = self.find_by_reference(chart_ref).await?;

        let import_data = import_data.as_ref().to_string();
        let account_specs = CsvParser::new(import_data).account_specs()?;

        let BulkImportResult {
            new_account_sets,
            new_account_set_ids,
            new_connections,
        } = BulkAccountImport::new(&mut chart, self.journal_id).import(account_specs);

        let mut op = self.repo.begin_op().await?;
        self.repo.update_in_op(&mut op, &mut chart).await?;

        let mut op = self
            .cala
            .ledger_operation_from_db_op(op.with_db_time().await?);
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

    #[record_error_severity]
    #[instrument(name = "core_accounting.chart_of_accounts.add_root_node", skip(self,))]
    pub async fn add_root_node(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart_ref: &str,
        spec: impl Into<AccountSpec> + std::fmt::Debug,
    ) -> Result<(Chart, Option<CalaAccountSetId>), ChartOfAccountsError> {
        let spec = spec.into();

        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_charts(),
                CoreAccountingAction::CHART_UPDATE,
            )
            .await?;

        let mut chart = self.find_by_reference(chart_ref).await?;
        let es_entity::Idempotent::Executed(NewChartAccountDetails {
            parent_account_set_id,
            new_account_set,
        }) = chart.create_root_node(&spec, self.journal_id)
        else {
            return Ok((chart, None));
        };
        let account_set_id = new_account_set.id;

        let mut op = self.repo.begin_op().await?;
        self.repo.update_in_op(&mut op, &mut chart).await?;

        let mut op = self
            .cala
            .ledger_operation_from_db_op(op.with_db_time().await?);
        self.cala
            .account_sets()
            .create_in_op(&mut op, new_account_set)
            .await?;
        self.cala
            .account_sets()
            .add_member_in_op(&mut op, parent_account_set_id, account_set_id)
            .await?;

        op.commit().await?;

        let new_account_set_id = chart.trial_balance_account_id_from_new_account(account_set_id);
        Ok((chart, new_account_set_id))
    }

    #[record_error_severity]
    #[instrument(name = "core_accounting.chart_of_accounts.add_child_node", skip(self))]
    pub async fn add_child_node(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart_ref: &str,
        parent_code: AccountCode,
        code: AccountCode,
        name: AccountName,
    ) -> Result<(Chart, Option<CalaAccountSetId>), ChartOfAccountsError> {
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_charts(),
                CoreAccountingAction::CHART_UPDATE,
            )
            .await?;

        let mut chart = self.find_by_reference(chart_ref).await?;
        let es_entity::Idempotent::Executed(NewChartAccountDetails {
            parent_account_set_id,
            new_account_set,
        }) = chart.create_child_node(parent_code, code, name, self.journal_id)?
        else {
            return Ok((chart, None));
        };
        let account_set_id = new_account_set.id;

        let mut op = self.repo.begin_op().await?;
        self.repo.update_in_op(&mut op, &mut chart).await?;

        let mut op = self
            .cala
            .ledger_operation_from_db_op(op.with_db_time().await?);
        self.cala
            .account_sets()
            .create_in_op(&mut op, new_account_set)
            .await?;
        self.cala
            .account_sets()
            .add_member_in_op(&mut op, parent_account_set_id, account_set_id)
            .await?;

        op.commit().await?;

        let new_account_set_id = chart.trial_balance_account_id_from_new_account(account_set_id);
        Ok((chart, new_account_set_id))
    }

    #[record_error_severity]
    #[instrument(name = "core_accounting.chart_of_accounts.close_as_of", skip(self, op))]
    pub async fn close_as_of(
        &self,
        mut op: es_entity::DbOp<'_>,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart_id: ChartId,
        closed_as_of: chrono::NaiveDate,
    ) -> Result<(), ChartOfAccountsError> {
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::chart(chart_id),
                CoreAccountingAction::CHART_CLOSE_MONTHLY,
            )
            .await?;

        let mut chart = self.find_by_id(chart_id).await?;
        if let Idempotent::Executed(closing_date) = chart.close_as_of(closed_as_of) {
            self.repo.update_in_op(&mut op, &mut chart).await?;
            self.chart_ledger
                .close_by_chart_root_account_set_as_of(op, closing_date, chart.account_set_id)
                .await?;
        }
        Ok(())
    }

    #[record_error_severity]
    #[instrument(
        name = "core_accounting.chart_of_accounts.post_closing_transaction",
        skip(self)
    )]
    pub async fn post_closing_transaction(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart_id: ChartId,
        account_codes: ClosingAccountCodes,
        tx_details: ClosingTxDetails,
    ) -> Result<(), ChartOfAccountsError> {
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::chart(chart_id),
                CoreAccountingAction::CHART_POST_CLOSING_TRANSACTION,
            )
            .await?;
        let mut chart = self.find_by_id(chart_id).await?;
        if let Idempotent::Executed(closing_tx_parents_and_details) =
            chart.post_closing_tx_as_of(account_codes, tx_details)?
        {
            let mut op = self.repo.begin_op().await?;
            self.repo.update_in_op(&mut op, &mut chart).await?;
            self.chart_ledger
                .post_closing_transaction(op, closing_tx_parents_and_details)
                .await?;
        }
        Ok(())
    }

    #[record_error_severity]
    #[instrument(name = "core_accounting.chart_of_accounts.find_by_id", skip(self))]
    pub async fn find_by_id(
        &self,
        id: impl Into<ChartId> + std::fmt::Debug,
    ) -> Result<Chart, ChartOfAccountsError> {
        self.repo.find_by_id(id.into()).await
    }

    #[record_error_severity]
    #[instrument(
        name = "core_accounting.chart_of_accounts.maybe_find_by_reference",
        skip(self)
    )]
    pub async fn maybe_find_by_reference(
        &self,
        reference: &str,
    ) -> Result<Option<Chart>, ChartOfAccountsError> {
        self.repo.maybe_find_by_reference(reference).await
    }

    #[record_error_severity]
    #[instrument(
        name = "core_accounting.chart_of_accounts.find_by_reference_with_sub",
        skip(self)
    )]
    pub async fn find_by_reference_with_sub(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        reference: &str,
    ) -> Result<Chart, ChartOfAccountsError> {
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_charts(),
                CoreAccountingAction::CHART_LIST,
            )
            .await?;

        self.find_by_reference(reference).await
    }

    #[record_error_severity]
    #[instrument(
        name = "core_accounting.chart_of_accounts.find_by_reference",
        skip(self)
    )]
    pub async fn find_by_reference(&self, reference: &str) -> Result<Chart, ChartOfAccountsError> {
        self.maybe_find_by_reference(reference)
            .await?
            .ok_or_else(move || {
                ChartOfAccountsError::ChartOfAccountsNotFoundByReference(reference.to_string())
            })
    }

    #[record_error_severity]
    #[instrument(name = "core_accounting.chart_of_accounts.find_all", skip(self))]
    pub async fn find_all<T: From<Chart>>(
        &self,
        ids: &[ChartId],
    ) -> Result<std::collections::HashMap<ChartId, T>, ChartOfAccountsError> {
        self.repo.find_all(ids).await
    }

    #[record_error_severity]
    #[instrument(
        name = "core_accounting.chart_of_accounts.manual_transaction_account_id_for_account_id_or_code",
        skip(self)
    )]
    pub async fn manual_transaction_account_id_for_account_id_or_code(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart_ref: &str,
        account_id_or_code: AccountIdOrCode,
    ) -> Result<LedgerAccountId, ChartOfAccountsError> {
        let mut chart = self.find_by_reference(chart_ref).await?;

        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_charts(),
                CoreAccountingAction::CHART_UPDATE,
            )
            .await?;

        let manual_transaction_account_id = match chart
            .manual_transaction_account(account_id_or_code)?
        {
            ManualAccountFromChart::IdInChart(id) | ManualAccountFromChart::NonChartId(id) => id,
            ManualAccountFromChart::NewAccount((account_set_id, new_account)) => {
                let mut op = self.repo.begin_op().await?;
                self.repo.update_in_op(&mut op, &mut chart).await?;

                let mut op = self
                    .cala
                    .ledger_operation_from_db_op(op.with_db_time().await?);
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
