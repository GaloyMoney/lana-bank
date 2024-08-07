use crate::primitives::{LedgerAccountId, Satoshis, UsdCents};
use serde::{Deserialize, Serialize};

use super::{cala::graphql::*, error::*};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct LoanAccountIds {
    pub collateral_account_id: LedgerAccountId,
    pub principal_receivable_account_id: LedgerAccountId,
    pub interest_receivable_account_id: LedgerAccountId,
    pub interest_account_id: LedgerAccountId,
}

impl LoanAccountIds {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            collateral_account_id: LedgerAccountId::new(),
            principal_receivable_account_id: LedgerAccountId::new(),
            interest_receivable_account_id: LedgerAccountId::new(),
            interest_account_id: LedgerAccountId::new(),
        }
    }
}

pub struct LoanBalance {
    pub collateral: Satoshis,
    pub principal_receivable: UsdCents,
    pub interest_receivable: UsdCents,
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
            principal_receivable: data
                .loan_principal_receivable
                .map(|b| UsdCents::try_from_usd(b.settled.normal_balance.units))
                .unwrap_or_else(|| Ok(UsdCents::ZERO))?,
            interest_receivable: data
                .loan_interest_receivable
                .map(|b| UsdCents::try_from_usd(b.settled.normal_balance.units))
                .unwrap_or_else(|| Ok(UsdCents::ZERO))?,
            interest_incurred: data
                .interest_income
                .map(|b| UsdCents::try_from_usd(b.settled.normal_balance.units))
                .unwrap_or_else(|| Ok(UsdCents::ZERO))?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LoanPayment {
    pub interest: Option<UsdCents>,
    pub principal: Option<UsdCents>,
}

impl LoanBalance {
    pub fn apply_payment(&self, amount: UsdCents) -> Result<LoanPayment, LedgerError> {
        let mut remaining = amount;

        let interest = std::cmp::min(amount, self.interest_receivable);
        remaining -= interest;

        let principal = std::cmp::min(remaining, self.principal_receivable);
        remaining -= principal;

        if remaining > UsdCents::ZERO {
            return Err(LedgerError::PaymentTooLarge(format!(
                "Amount '{}' too large for outstanding principal '{}' and interest '{}'",
                amount, self.principal_receivable, self.interest_receivable
            )));
        }

        Ok(LoanPayment {
            interest: Some(interest).filter(|&p| p > UsdCents::ZERO),
            principal: Some(principal).filter(|&p| p > UsdCents::ZERO),
        })
    }
}

#[cfg(test)]
mod test {

    mod apply_payment {
        use super::super::*;
        use crate::primitives::{Satoshis, UsdCents};

        fn default_balance() -> LoanBalance {
            LoanBalance {
                collateral: Satoshis::from(10000000),
                principal_receivable: UsdCents::from(200000),
                interest_receivable: UsdCents::from(1500),
                interest_incurred: UsdCents::from(2000),
            }
        }

        #[test]
        fn it_applies_partial_interest() {
            let balance = default_balance();
            let loan_payment = balance.apply_payment(UsdCents::from(1499)).unwrap();
            assert_eq!(
                loan_payment,
                LoanPayment {
                    interest: Some(UsdCents::from(1499)),
                    principal: None
                }
            );
        }

        #[test]
        fn it_applies_full_interest() {
            let balance = default_balance();
            let loan_payment = balance.apply_payment(UsdCents::from(1500)).unwrap();
            assert_eq!(
                loan_payment,
                LoanPayment {
                    interest: Some(UsdCents::from(1500)),
                    principal: None
                }
            );
        }

        #[test]
        fn it_applies_partial_principal() {
            let balance = default_balance();
            let loan_payment = balance.apply_payment(UsdCents::from(1501)).unwrap();
            assert_eq!(
                loan_payment,
                LoanPayment {
                    interest: Some(UsdCents::from(1500)),
                    principal: Some(UsdCents::from(1))
                }
            );
        }

        #[test]
        fn it_applies_full_principal() {
            let balance = default_balance();
            let loan_payment = balance.apply_payment(UsdCents::from(201500)).unwrap();
            assert_eq!(
                loan_payment,
                LoanPayment {
                    interest: Some(UsdCents::from(1500)),
                    principal: Some(UsdCents::from(200000))
                }
            );
        }

        #[test]
        fn it_errors_on_overpayment() {
            let balance = default_balance();
            let loan_payment = balance.apply_payment(UsdCents::from(201501));
            assert!(loan_payment.is_err());
        }
    }
}
