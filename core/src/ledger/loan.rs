use serde::{Deserialize, Serialize};

use crate::primitives::{CollateralAction, LedgerAccountId, LedgerTxId, Satoshis, UsdCents};

use super::{cala::graphql::*, customer::CustomerLedgerAccountIds, error::*};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct LoanAccountIds {
    pub collateral_account_id: LedgerAccountId,
    pub disbursed_receivable_account_id: LedgerAccountId,
    pub interest_receivable_account_id: LedgerAccountId,
    pub facility_account_id: LedgerAccountId,
    pub interest_account_id: LedgerAccountId,
}

impl LoanAccountIds {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            collateral_account_id: LedgerAccountId::new(),
            disbursed_receivable_account_id: LedgerAccountId::new(),
            interest_receivable_account_id: LedgerAccountId::new(),
            facility_account_id: LedgerAccountId::new(),
            interest_account_id: LedgerAccountId::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum LoanRepayment {
    Partial {
        tx_id: LedgerTxId,
        tx_ref: String,
        loan_account_ids: LoanAccountIds,
        customer_account_ids: CustomerLedgerAccountIds,
        amounts: LoanPaymentAmounts,
    },
    Final {
        payment_tx_id: LedgerTxId,
        payment_tx_ref: String,
        collateral_tx_id: LedgerTxId,
        collateral_tx_ref: String,
        collateral: Satoshis,
        loan_account_ids: LoanAccountIds,
        customer_account_ids: CustomerLedgerAccountIds,
        amounts: LoanPaymentAmounts,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LoanPaymentAmounts {
    pub interest: UsdCents,
    pub principal: UsdCents,
}

pub struct LoanBalance {
    pub collateral: Satoshis,
    pub disbursed_receivable: UsdCents,
    pub interest_receivable: UsdCents,
    pub facility_remaining: UsdCents,
    pub interest_incurred: UsdCents,
}

impl TryFrom<loan_balance::ResponseData> for LoanBalance {
    type Error = LedgerError;

    fn try_from(data: loan_balance::ResponseData) -> Result<Self, Self::Error> {
        Ok(LoanBalance {
            collateral: data
                .collateral
                .map(|b| Satoshis::try_from_btc(b.settled.normal_balance.units))
                .unwrap_or_else(|| Ok(Satoshis::ZERO))?,
            disbursed_receivable: data
                .loan_disbursed_receivable
                .map(|b| UsdCents::try_from_usd(b.settled.normal_balance.units))
                .unwrap_or_else(|| Ok(UsdCents::ZERO))?,
            interest_receivable: data
                .loan_interest_receivable
                .map(|b| UsdCents::try_from_usd(b.settled.normal_balance.units))
                .unwrap_or_else(|| Ok(UsdCents::ZERO))?,
            facility_remaining: data
                .loan_facility
                .map(|b| UsdCents::try_from_usd(b.settled.normal_balance.units))
                .unwrap_or_else(|| Ok(UsdCents::ZERO))?,
            interest_incurred: data
                .interest_income
                .map(|b| UsdCents::try_from_usd(b.settled.normal_balance.units))
                .unwrap_or_else(|| Ok(UsdCents::ZERO))?,
        })
    }
}

impl LoanBalance {
    pub fn check_disbursement_amount(&self, amount: UsdCents) -> Result<(), LedgerError> {
        if amount > self.facility_remaining {
            return Err(LedgerError::DisbursementAmountTooLarge(
                amount,
                self.facility_remaining,
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct LoanCollateralUpdate {
    pub tx_ref: String,
    pub tx_id: LedgerTxId,
    pub abs_diff: Satoshis,
    pub action: CollateralAction,
    pub loan_account_ids: LoanAccountIds,
}

#[derive(Debug, Clone)]
pub struct LoanInterestAccrual {
    pub interest: UsdCents,
    pub tx_ref: String,
    pub tx_id: LedgerTxId,
    pub loan_account_ids: LoanAccountIds,
}

#[derive(Debug, Clone)]
pub struct LoanApprovalData {
    pub initial_principal: UsdCents,
    pub tx_ref: String,
    pub tx_id: LedgerTxId,
    pub loan_account_ids: LoanAccountIds,
    pub customer_account_ids: CustomerLedgerAccountIds,
}
