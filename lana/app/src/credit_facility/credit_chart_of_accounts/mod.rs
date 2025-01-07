use audit::AuditInfo;
use cala_ledger::CalaLedger;
use chart_of_accounts::TransactionAccountFactory;
use lana_ids::CreditFacilityId;

pub mod error;

use error::CreditChartOfAccountsError;

use super::CreditFacilityAccountIds;

#[derive(Clone)]
pub struct CreditChartOfAccounts {
    cala: CalaLedger,
    collateral_factory: TransactionAccountFactory,
    facility_factory: TransactionAccountFactory,
    disbursed_receivable_factory: TransactionAccountFactory,
    interest_receivable_factory: TransactionAccountFactory,
    interest_income_factory: TransactionAccountFactory,
    fee_income_factory: TransactionAccountFactory,
}

impl CreditChartOfAccounts {
    pub async fn init(
        cala: &CalaLedger,
        collateral_factory: TransactionAccountFactory,
        facility_factory: TransactionAccountFactory,
        disbursed_receivable_factory: TransactionAccountFactory,
        interest_receivable_factory: TransactionAccountFactory,
        interest_income_factory: TransactionAccountFactory,
        fee_income_factory: TransactionAccountFactory,
    ) -> Result<Self, CreditChartOfAccountsError> {
        Ok(Self {
            cala: cala.clone(),
            collateral_factory,
            facility_factory,
            disbursed_receivable_factory,
            interest_receivable_factory,
            interest_income_factory,
            fee_income_factory,
        })
    }

    pub async fn create_accounts_for_credit_facility(
        &self,
        op: es_entity::DbOp<'_>,
        credit_facility_id: CreditFacilityId,
        account_ids: CreditFacilityAccountIds,
        audit_info: AuditInfo,
    ) -> Result<(), CreditChartOfAccountsError> {
        let mut op = self.cala.ledger_operation_from_db_op(op);

        let collateral_name = &format!(
            "Credit Facility Collateral Account for {}",
            credit_facility_id
        );
        let _collateral_details = self
            .collateral_factory
            .create_transaction_account_in_op(
                &mut op,
                account_ids.collateral_account_id,
                collateral_name,
                collateral_name,
                audit_info.clone(),
            )
            .await?;

        let facility_name = &format!(
            "Off-Balance-Sheet Facility Account for Credit Facility {}",
            credit_facility_id
        );
        let _facility_details = self
            .facility_factory
            .create_transaction_account_in_op(
                &mut op,
                account_ids.facility_account_id,
                facility_name,
                facility_name,
                audit_info.clone(),
            )
            .await?;

        let disbursed_receivable_name = &format!(
            "Disbursed Receivable Account for Credit Facility {}",
            credit_facility_id
        );
        let _disbursed_receivable_details = self
            .disbursed_receivable_factory
            .create_transaction_account_in_op(
                &mut op,
                account_ids.disbursed_receivable_account_id,
                disbursed_receivable_name,
                disbursed_receivable_name,
                audit_info.clone(),
            )
            .await?;

        let interest_receivable_name = &format!(
            "Interest Receivable Account for Credit Facility {}",
            credit_facility_id
        );
        let _interest_receivable_details = self
            .interest_receivable_factory
            .create_transaction_account_in_op(
                &mut op,
                account_ids.interest_receivable_account_id,
                interest_receivable_name,
                interest_receivable_name,
                audit_info.clone(),
            )
            .await?;

        let interest_income_name = &format!(
            "Interest Income Account for Credit Facility {}",
            credit_facility_id
        );
        let _interest_income_details = self
            .interest_income_factory
            .create_transaction_account_in_op(
                &mut op,
                account_ids.interest_account_id,
                interest_income_name,
                interest_income_name,
                audit_info.clone(),
            )
            .await?;

        let fee_income_name = &format!(
            "Fee Income Account for Credit Facility {}",
            credit_facility_id
        );
        let _fee_income_details = self
            .fee_income_factory
            .create_transaction_account_in_op(
                &mut op,
                account_ids.fee_income_account_id,
                fee_income_name,
                fee_income_name,
                audit_info.clone(),
            )
            .await?;

        op.commit().await?;

        Ok(())
    }
}
