mod credit_facility_accounts;
pub mod error;
mod templates;

use cala_ledger::{account::NewAccount, CalaLedger, Currency, JournalId};

use crate::primitives::{CreditFacilityId, Satoshis, UsdCents};

pub use credit_facility_accounts::*;
use error::*;

#[derive(Clone)]
pub struct CreditLedger {
    cala: CalaLedger,
    journal_id: JournalId,
    usd: Currency,
    btc: Currency,
}

impl CreditLedger {
    pub async fn init(cala: &CalaLedger, journal_id: JournalId) -> Result<Self, CreditLedgerError> {
        templates::AddCollateral::init(cala).await?;
        templates::RemoveCollateral::init(cala).await?;

        Ok(Self {
            cala: cala.clone(),
            journal_id,
            usd: "USD".parse().expect("Could not parse 'USD'"),
            btc: "BTC".parse().expect("Could not parse 'BTC'"),
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

    pub async fn get_credit_facility_balance(
        &self,
        CreditFacilityAccountIds {
            facility_account_id,
            disbursed_receivable_account_id,
            collateral_account_id,
            interest_receivable_account_id,
            ..
        }: CreditFacilityAccountIds,
    ) -> Result<CreditFacilityLedgerBalance, CreditLedgerError> {
        let facility_id = (self.journal_id, facility_account_id, self.usd);
        let collateral_id = (self.journal_id, collateral_account_id, self.btc);
        let disbursed_receivable_id = (self.journal_id, disbursed_receivable_account_id, self.btc);
        let interest_receivable_id = (self.journal_id, interest_receivable_account_id, self.btc);
        let balances = self
            .cala
            .balances()
            .find_all(&[
                facility_id,
                collateral_id,
                disbursed_receivable_id,
                interest_receivable_id,
            ])
            .await?;
        let facility = if let Some(b) = balances.get(&facility_id) {
            UsdCents::try_from_usd(b.settled())?
        } else {
            UsdCents::ZERO
        };
        let disbursed = if let Some(b) = balances.get(&disbursed_receivable_id) {
            UsdCents::try_from_usd(b.details.settled.dr_balance)?
        } else {
            UsdCents::ZERO
        };
        let disbursed_receivable = if let Some(b) = balances.get(&disbursed_receivable_id) {
            UsdCents::try_from_usd(b.settled())?
        } else {
            UsdCents::ZERO
        };
        let interest = if let Some(b) = balances.get(&interest_receivable_id) {
            UsdCents::try_from_usd(b.details.settled.dr_balance)?
        } else {
            UsdCents::ZERO
        };
        let interest_receivable = if let Some(b) = balances.get(&interest_receivable_id) {
            UsdCents::try_from_usd(b.settled())?
        } else {
            UsdCents::ZERO
        };
        let collateral = if let Some(b) = balances.get(&collateral_id) {
            Satoshis::try_from_btc(b.settled())?
        } else {
            Satoshis::ZERO
        };
        Ok(CreditFacilityLedgerBalance {
            facility,
            collateral,
            disbursed,
            disbursed_receivable,
            interest,
            interest_receivable,
        })
    }

    pub async fn update_credit_facility_collateral(
        &self,
        // CreditFacilityCollateralUpdate {
        //     tx_id,
        //     credit_facility_account_ids,
        //     abs_diff,
        //     tx_ref,
        //     action,
        // }: CreditFacilityCollateralUpdate,
    ) -> Result<(), CreditLedgerError> {
        unimplemented!()
    }
}
