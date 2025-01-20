use cala_ledger::LedgerOperation;
use chart_of_accounts::TransactionAccountFactory;
use lana_ids::CreditFacilityId;

pub mod error;

use error::CreditChartOfAccountsError;

use super::CreditFacilityAccountIds;

#[derive(Clone)]
pub struct CreditChartOfAccounts {
    collateral_factory: TransactionAccountFactory,
    facility_factory: TransactionAccountFactory,
    disbursed_receivable_factory: TransactionAccountFactory,
    interest_receivable_factory: TransactionAccountFactory,
    interest_income_factory: TransactionAccountFactory,
    fee_income_factory: TransactionAccountFactory,
}

impl CreditChartOfAccounts {
    pub fn new(
        collateral_factory: TransactionAccountFactory,
        facility_factory: TransactionAccountFactory,
        disbursed_receivable_factory: TransactionAccountFactory,
        interest_receivable_factory: TransactionAccountFactory,
        interest_income_factory: TransactionAccountFactory,
        fee_income_factory: TransactionAccountFactory,
    ) -> Self {
        Self {
            collateral_factory,
            facility_factory,
            disbursed_receivable_factory,
            interest_receivable_factory,
            interest_income_factory,
            fee_income_factory,
        }
    }

    // TODO: move to CreditLedger
    pub async fn create_accounts_for_credit_facility(
        &self,
        op: &mut LedgerOperation<'_>,
        credit_facility_id: CreditFacilityId,
        account_ids: CreditFacilityAccountIds,
    ) -> Result<(), CreditChartOfAccountsError> {
        let collateral_name = &format!(
            "Credit Facility Collateral Account for {}",
            credit_facility_id
        );
        self.collateral_factory
            .create_transaction_account_in_op(
                op,
                account_ids.collateral_account_id,
                collateral_name,
                collateral_name,
            )
            .await?;

        let facility_name = &format!(
            "Off-Balance-Sheet Facility Account for Credit Facility {}",
            credit_facility_id
        );
        self.facility_factory
            .create_transaction_account_in_op(
                op,
                account_ids.facility_account_id,
                facility_name,
                facility_name,
            )
            .await?;

        let disbursed_receivable_name = &format!(
            "Disbursed Receivable Account for Credit Facility {}",
            credit_facility_id
        );
        self.disbursed_receivable_factory
            .create_transaction_account_in_op(
                op,
                account_ids.disbursed_receivable_account_id,
                disbursed_receivable_name,
                disbursed_receivable_name,
            )
            .await?;

        let interest_receivable_name = &format!(
            "Interest Receivable Account for Credit Facility {}",
            credit_facility_id
        );
        self.interest_receivable_factory
            .create_transaction_account_in_op(
                op,
                account_ids.interest_receivable_account_id,
                interest_receivable_name,
                interest_receivable_name,
            )
            .await?;

        let interest_income_name = &format!(
            "Interest Income Account for Credit Facility {}",
            credit_facility_id
        );
        self.interest_income_factory
            .create_transaction_account_in_op(
                op,
                account_ids.interest_account_id,
                interest_income_name,
                interest_income_name,
            )
            .await?;

        let fee_income_name = &format!(
            "Fee Income Account for Credit Facility {}",
            credit_facility_id
        );
        self.fee_income_factory
            .create_transaction_account_in_op(
                op,
                account_ids.fee_income_account_id,
                fee_income_name,
                fee_income_name,
            )
            .await?;

        Ok(())
    }
}
