mod credit_facility_accounts;
pub mod error;
// mod templates;

use cala_ledger::{account::NewAccount, CalaLedger, JournalId};

use crate::primitives::CreditFacilityId;

pub use credit_facility_accounts::*;
use error::*;

#[derive(Clone)]
pub struct CreditLedger {
    cala: CalaLedger,
    _journal_id: JournalId,
}

impl CreditLedger {
    pub async fn init(cala: &CalaLedger, journal_id: JournalId) -> Result<Self, CreditLedgerError> {
        Ok(Self {
            cala: cala.clone(),
            _journal_id: journal_id,
        })
    }

    pub async fn create_accounts_for_credit_facility(
        &self,
        op: es_entity::DbOp<'_>,
        credit_facility_id: CreditFacilityId,
        CreditFacilityAccountIds {
            facility_account_id,
            disbursed_receivable_account_id,
            collateral_account_id,
            interest_receivable_account_id,
            interest_account_id,
            fee_income_account_id,
        }: CreditFacilityAccountIds,
    ) -> Result<(), CreditLedgerError> {
        let mut op = self.cala.ledger_operation_from_db_op(op);
        let new_accounts = vec![
            NewAccount::builder()
                .id(collateral_account_id)
                .name("Credit Facility Collateral Account")
                .code(format!("CREDIT_FACILITY.COLLATERAL.{}", credit_facility_id))
                .build()
                .expect("new account"),
            NewAccount::builder()
                .id(facility_account_id)
                .name("Off-Balance-Sheet Facility Account for Credit Facility")
                .code(format!(
                    "CREDIT_FACILITY.OBS_FACILITY.{}",
                    credit_facility_id
                ))
                .build()
                .expect("new account"),
            NewAccount::builder()
                .id(disbursed_receivable_account_id)
                .name("Disbursed Receivable Account for Credit Facility")
                .code(format!(
                    "CREDIT_FACILITY.DISBURSED_RECEIVABLE.{}",
                    credit_facility_id
                ))
                .build()
                .expect("new account"),
            NewAccount::builder()
                .id(interest_receivable_account_id)
                .name("Interest Receivable Account for Credit Facility")
                .code(format!(
                    "CREDIT_FACILITY.INTEREST_RECEIVABLE.{}",
                    credit_facility_id
                ))
                .build()
                .expect("new account"),
            NewAccount::builder()
                .id(interest_account_id)
                .name("Interest Income for Credit Facility")
                .code(format!(
                    "CREDIT_FACILITY.INTEREST_INCOME.{}",
                    credit_facility_id
                ))
                .build()
                .expect("new account"),
            NewAccount::builder()
                .id(fee_income_account_id)
                .name("Fee Income for Credit Facility")
                .code(format!("CREDIT_FACILITY.FEE_INCOME.{}", credit_facility_id))
                .build()
                .expect("new account"),
        ];

        self.cala
            .accounts()
            .create_all_in_op(&mut op, new_accounts)
            .await?;

        op.commit().await?;

        Ok(())
    }
}
