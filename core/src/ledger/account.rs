use crate::primitives::{
    LedgerAccountId, LedgerDebitOrCredit, Satoshis, SignedSatoshis, SignedUsdCents, UsdCents,
};

use super::{cala::graphql::*, error::*};

#[derive(Debug, Clone, PartialEq)]
pub struct BtcAccountBalance {
    pub debit: Satoshis,
    pub credit: Satoshis,
    pub net_normal: Satoshis,
    pub net_debit: SignedSatoshis,
    pub net_credit: SignedSatoshis,
}

impl Default for BtcAccountBalance {
    fn default() -> Self {
        Self {
            debit: Satoshis::ZERO,
            credit: Satoshis::ZERO,
            net_normal: Satoshis::ZERO,
            net_debit: SignedSatoshis::ZERO,
            net_credit: SignedSatoshis::ZERO,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsdAccountBalance {
    pub debit: UsdCents,
    pub credit: UsdCents,
    pub net_normal: UsdCents,
    pub net_debit: SignedUsdCents,
    pub net_credit: SignedUsdCents,
}

impl Default for UsdAccountBalance {
    fn default() -> Self {
        Self {
            debit: UsdCents::ZERO,
            credit: UsdCents::ZERO,
            net_normal: UsdCents::ZERO,
            net_debit: SignedUsdCents::ZERO,
            net_credit: SignedUsdCents::ZERO,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct LayeredBtcAccountBalances {
    pub settled: BtcAccountBalance,
    pub pending: BtcAccountBalance,
    pub encumbrance: BtcAccountBalance,
    pub all_layers: BtcAccountBalance,
}

#[derive(Debug, Clone, Default)]
pub struct LayeredUsdAccountBalances {
    pub settled: UsdAccountBalance,
    pub pending: UsdAccountBalance,
    pub encumbrance: UsdAccountBalance,
    pub all_layers: UsdAccountBalance,
}
#[derive(Debug, Clone)]
pub struct LedgerAccountBalancesByCurrency {
    pub btc: LayeredBtcAccountBalances,
    pub usd: LayeredUsdAccountBalances,
    pub usdt: LayeredUsdAccountBalances,
}

#[derive(Debug, Clone)]
pub struct LedgerAccountWithBalance {
    pub id: LedgerAccountId,
    pub name: String,
    pub normal_balance_type: LedgerDebitOrCredit,
    pub balance: LedgerAccountBalancesByCurrency,
}

macro_rules! impl_from_account_details_and_balances {
    ($($module:ident),+)  => {
        $(
            impl From<$module::DebitOrCredit> for LedgerDebitOrCredit {
                fn from(debit_or_credit: $module::DebitOrCredit) -> Self {
                    match debit_or_credit {
                        $module::DebitOrCredit::DEBIT => LedgerDebitOrCredit::Debit,
                        $module::DebitOrCredit::CREDIT => LedgerDebitOrCredit::Credit,
                        _ => todo!(),
                    }
                }
            }

            impl TryFrom<$module::balances> for BtcAccountBalance {
                type Error = LedgerError;

                fn try_from(balances: $module::balances) -> Result<Self, Self::Error> {
                    let net_normal = Satoshis::try_from_btc(balances.normal_balance.units)?;
                    let debit = Satoshis::try_from_btc(balances.dr_balance.units)?;
                    let credit = Satoshis::try_from_btc(balances.cr_balance.units)?;
                    let net_debit = SignedSatoshis::from(debit) - SignedSatoshis::from(credit);
                    let net_credit = SignedSatoshis::from(credit) - SignedSatoshis::from(debit);

                    Ok(Self {
                        debit,
                        credit,
                        net_normal,
                        net_debit,
                        net_credit,
                    })
                }
            }

            impl TryFrom<$module::balances> for UsdAccountBalance {
                type Error = LedgerError;

                fn try_from(balances: $module::balances) -> Result<Self, Self::Error> {
                    let net_normal = UsdCents::try_from_usd(balances.normal_balance.units)?;
                    let debit = UsdCents::try_from_usd(balances.dr_balance.units)?;
                    let credit = UsdCents::try_from_usd(balances.cr_balance.units)?;
                    let net_debit = SignedUsdCents::from(debit) - SignedUsdCents::from(credit);
                    let net_credit = SignedUsdCents::from(credit) - SignedUsdCents::from(debit);

                    Ok(Self {
                        debit,
                        credit,
                        net_normal,
                        net_debit,
                        net_credit,
                    })
                }
            }

            impl TryFrom<$module::balancesByLayer> for LayeredBtcAccountBalances {
                type Error = LedgerError;

                fn try_from(btc_balances_by_layer: $module::balancesByLayer) -> Result<Self, Self::Error> {
                    Ok(Self {
                        settled: BtcAccountBalance::try_from(btc_balances_by_layer.settled)?,
                        pending: BtcAccountBalance::try_from(btc_balances_by_layer.pending)?,
                        encumbrance: BtcAccountBalance::try_from(btc_balances_by_layer.encumbrance)?,
                        all_layers: BtcAccountBalance::try_from(btc_balances_by_layer.all_layers_available)?,
                    })
                }
            }

            impl TryFrom<$module::balancesByLayer> for LayeredUsdAccountBalances {
                type Error = LedgerError;

                fn try_from(usd_balances_by_layer: $module::balancesByLayer) -> Result<Self, Self::Error> {
                    Ok(Self {
                        settled: UsdAccountBalance::try_from(usd_balances_by_layer.settled)?,
                        pending: UsdAccountBalance::try_from(usd_balances_by_layer.pending)?,
                        encumbrance: UsdAccountBalance::try_from(usd_balances_by_layer.encumbrance)?,
                        all_layers: UsdAccountBalance::try_from(usd_balances_by_layer.all_layers_available)?,
                    })
                }
            }

            impl TryFrom<$module::accountSetBalances> for LedgerAccountBalancesByCurrency {
                type Error = LedgerError;

                fn try_from(balances: $module::accountSetBalances) -> Result<Self, Self::Error> {
                    Ok(LedgerAccountBalancesByCurrency {
                        btc: balances.btc_balances.map(
                            LayeredBtcAccountBalances::try_from
                        ).unwrap_or_else(|| Ok(LayeredBtcAccountBalances::default()))?,
                        usd: balances.usd_balances.map(
                            LayeredUsdAccountBalances::try_from
                        ).unwrap_or_else(|| Ok(LayeredUsdAccountBalances::default()))?,
                        usdt: balances.usdt_balances.map(
                            LayeredUsdAccountBalances::try_from
                        ).unwrap_or_else(|| Ok(LayeredUsdAccountBalances::default()))?,
                    })
                }
            }

            impl TryFrom<$module::accountDetailsAndBalances> for LedgerAccountWithBalance {
                type Error = LedgerError;

                fn try_from(account: $module::accountDetailsAndBalances) -> Result<Self, Self::Error> {
                    let account_details = account.account_details;
                    Ok(LedgerAccountWithBalance {
                        id: account_details.account_id.into(),
                        name: account_details.name,
                        normal_balance_type: account_details.normal_balance_type.into(),
                        balance: LedgerAccountBalancesByCurrency {
                            btc: account.account_balances.btc_balances.map(
                                LayeredBtcAccountBalances::try_from,
                            ).unwrap_or_else(|| Ok(LayeredBtcAccountBalances::default()))?,
                            usd: account.account_balances.usd_balances.map(
                                LayeredUsdAccountBalances::try_from,
                            ).unwrap_or_else(|| Ok(LayeredUsdAccountBalances::default()))?,
                            usdt: account.account_balances.usdt_balances.map(
                                LayeredUsdAccountBalances::try_from,
                            ).unwrap_or_else(|| Ok(LayeredUsdAccountBalances::default()))?,
                        },
                    })
                }
            }
        )+
    };
}

impl_from_account_details_and_balances!(
    chart_of_accounts,
    account_set_and_sub_accounts_with_balance,
    trial_balance,
    balance_sheet,
    profit_and_loss_statement
);

#[cfg(test)]
mod tests {

    use rust_decimal::Decimal;
    use rusty_money::{crypto, iso};
    use trial_balance::{BalancesCrBalance, BalancesDrBalance, BalancesNormalBalance};

    use crate::primitives::Currency;

    use super::*;

    #[test]
    fn calculate_debit_normal_btc_balance() {
        let currency = Currency::Crypto(crypto::BTC);

        let debit_amount = Decimal::new(50000, 8);
        let dr_balance = BalancesDrBalance {
            units: debit_amount,
            currency,
        };

        let credit_amount = Decimal::new(1000000, 8);
        let cr_balance = BalancesCrBalance {
            units: credit_amount,
            currency,
        };

        let net_amount_pos = Decimal::new(950000, 8);
        let net_amount_neg = Decimal::new(-950000, 8);
        let btc_balance = trial_balance::balances {
            dr_balance,
            cr_balance,
            normal_balance: BalancesNormalBalance {
                units: net_amount_pos,
                currency,
            },
        };
        let expected_debit_normal_balance = BtcAccountBalance {
            debit: Satoshis::try_from_btc(debit_amount).unwrap(),
            credit: Satoshis::try_from_btc(credit_amount).unwrap(),
            net_normal: Satoshis::try_from_btc(net_amount_pos).unwrap(),
            net_debit: SignedSatoshis::from_btc(net_amount_neg),
            net_credit: SignedSatoshis::from_btc(net_amount_pos),
        };

        let debit_normal_balance: BtcAccountBalance = btc_balance.try_into().unwrap();

        assert_eq!(debit_normal_balance, expected_debit_normal_balance);
    }

    #[test]
    fn calculate_debit_normal_usd_balance() {
        let currency = Currency::Iso(iso::USD);

        let debit_amount = Decimal::new(500, 2);
        let dr_balance = BalancesDrBalance {
            units: debit_amount,
            currency,
        };

        let credit_amount = Decimal::new(10000, 2);
        let cr_balance = BalancesCrBalance {
            units: credit_amount,
            currency,
        };

        let net_amount_pos = Decimal::new(9500, 2);
        let net_amount_neg = Decimal::new(-9500, 2);
        let usd_balance = trial_balance::balances {
            dr_balance,
            cr_balance,
            normal_balance: BalancesNormalBalance {
                units: net_amount_pos,
                currency,
            },
        };
        let expected_debit_normal_balance = UsdAccountBalance {
            debit: UsdCents::try_from_usd(debit_amount).unwrap(),
            credit: UsdCents::try_from_usd(credit_amount).unwrap(),
            net_normal: UsdCents::try_from_usd(net_amount_pos).unwrap(),
            net_debit: SignedUsdCents::from_usd(net_amount_neg),
            net_credit: SignedUsdCents::from_usd(net_amount_pos),
        };

        let debit_normal_balance: UsdAccountBalance = usd_balance.try_into().unwrap();

        assert_eq!(debit_normal_balance, expected_debit_normal_balance);
    }
}
