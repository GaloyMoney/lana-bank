use csv::Writer;
use rust_decimal::Decimal;

use audit::AuditSvc;
use authz::PermissionCheck;
use cala_ledger::DebitOrCredit;

use crate::{
    CoreAccountingAction, CoreAccountingObject, ledger_account::LedgerAccounts,
    primitives::LedgerAccountId,
};

use super::error::AccountingCsvError;

pub struct GenerateCsv<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreAccountingAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreAccountingObject>,
{
    ledger_accounts: LedgerAccounts<Perms>,
}

impl<Perms> GenerateCsv<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreAccountingAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreAccountingObject>,
{
    pub fn new(ledger_accounts: &LedgerAccounts<Perms>) -> Self {
        Self {
            ledger_accounts: ledger_accounts.clone(),
        }
    }

    pub async fn generate_ledger_account_csv(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        ledger_account_id: LedgerAccountId,
    ) -> Result<Vec<u8>, AccountingCsvError> {
        let history_result = self
            .ledger_accounts
            .history(sub, ledger_account_id, Default::default())
            .await
            .map_err(AccountingCsvError::LedgerAccountError)?;

        let mut wtr = Writer::from_writer(vec![]);
        wtr.write_record([
            "Recorded At",
            "Currency",
            "Debit Amount",
            "Credit Amount",
            "Description",
            "Entry Type",
        ])
        .map_err(|e| AccountingCsvError::CsvError(e.to_string()))?;

        for entry in history_result.entities {
            let formatted_amount = entry.amount.to_display_amount();
            let currency = entry.amount.currency_code();

            let (debit_amount, credit_amount) = match entry.direction {
                DebitOrCredit::Debit => (formatted_amount, Decimal::from(0).to_string()),
                DebitOrCredit::Credit => (Decimal::from(0).to_string(), formatted_amount),
            };

            wtr.write_record(&[
                entry.created_at.to_rfc3339(),
                currency,
                debit_amount,
                credit_amount,
                entry.description.unwrap_or_default(),
                entry.entry_type,
            ])
            .map_err(|e| AccountingCsvError::CsvError(e.to_string()))?;
        }
        let csv_data = wtr
            .into_inner()
            .map_err(|e| AccountingCsvError::CsvError(e.to_string()))?;

        Ok(csv_data)
    }
}
