use crate::primitives::UsdCents;

use super::{cala::graphql::*, error::*, primitives::LayeredUsdBalance};

pub struct BankDepositsBalance {
    pub usd_balance: LayeredUsdBalance,
}

impl TryFrom<account_by_code::AccountByCodeAccountByCode> for BankDepositsBalance {
    type Error = LedgerError;

    fn try_from(data: account_by_code::AccountByCodeAccountByCode) -> Result<Self, Self::Error> {
        Ok(BankDepositsBalance {
            usd_balance: LayeredUsdBalance {
                settled: data
                    .usd_balance
                    .clone()
                    .map(|b| UsdCents::try_from_usd(b.settled.normal_balance.units))
                    .unwrap_or_else(|| Ok(UsdCents::ZERO))?,
                pending: data
                    .usd_balance
                    .map(|b| UsdCents::try_from_usd(b.pending.normal_balance.units))
                    .unwrap_or_else(|| Ok(UsdCents::ZERO))?,
            },
        })
    }
}

impl BankDepositsBalance {
    pub const ZERO: Self = BankDepositsBalance {
        usd_balance: LayeredUsdBalance::ZERO,
    };

    pub fn check_withdrawal_amount(&self, amount: UsdCents) -> Result<UsdCents, LedgerError> {
        if self.usd_balance.settled < amount {
            return Err(LedgerError::WithdrawalAmountTooLarge(
                amount,
                self.usd_balance.settled,
            ));
        }
        Ok(amount)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn check_withdrawal_amount() {
        let balance = BankDepositsBalance::ZERO;
        assert!(matches!(
            balance.check_withdrawal_amount(UsdCents::ONE),
            Err(LedgerError::WithdrawalAmountTooLarge(_, _))
        ));
    }
}
