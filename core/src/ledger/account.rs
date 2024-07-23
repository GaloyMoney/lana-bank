use crate::primitives::{
    LedgerAccountId, LedgerDebitOrCredit, Satoshis, SignedSatoshis, SignedUsdCents, UsdCents,
};

use super::cala::graphql::*;

#[derive(Debug, Clone, PartialEq)]
pub struct BtcAccountBalance {
    pub debit: Satoshis,
    pub credit: Satoshis,
    pub net_normal: Satoshis,
    pub net_debit: SignedSatoshis,
}

impl Default for BtcAccountBalance {
    fn default() -> Self {
        Self {
            debit: Satoshis::ZERO,
            credit: Satoshis::ZERO,
            net_normal: Satoshis::ZERO,
            net_debit: SignedSatoshis::ZERO,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsdAccountBalance {
    pub debit: UsdCents,
    pub credit: UsdCents,
    pub net_normal: UsdCents,
    pub net_debit: SignedUsdCents,
}

impl Default for UsdAccountBalance {
    fn default() -> Self {
        Self {
            debit: UsdCents::ZERO,
            credit: UsdCents::ZERO,
            net_normal: UsdCents::ZERO,
            net_debit: SignedUsdCents::ZERO,
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

            impl From<$module::balances> for BtcAccountBalance {
                fn from(balances: $module::balances) -> Self {
                    let net_normal = Satoshis::from_btc(balances.normal_balance.units);
                    let debit = Satoshis::from_btc(balances.dr_balance.units);
                    let credit = Satoshis::from_btc(balances.cr_balance.units);
                    let net_debit = SignedSatoshis::from(debit) - SignedSatoshis::from(credit);

                    Self {
                        debit,
                        credit,
                        net_normal,
                        net_debit,
                    }
                }
            }

            impl From<$module::balances> for UsdAccountBalance {
                fn from(balances: $module::balances) -> Self {
                    let net_normal = UsdCents::from_usd(balances.normal_balance.units);
                    let debit = UsdCents::from_usd(balances.dr_balance.units);
                    let credit = UsdCents::from_usd(balances.cr_balance.units);
                    let net_debit = SignedUsdCents::from(debit) - SignedUsdCents::from(credit);

                    Self {
                        debit,
                        credit,
                        net_normal,
                        net_debit,
                    }
                }
            }

            impl From<$module::balancesByLayer> for LayeredBtcAccountBalances {
                fn from(btc_balances_by_layer: $module::balancesByLayer) -> Self {
                    Self {
                        settled: BtcAccountBalance::from(btc_balances_by_layer.settled),
                        pending: BtcAccountBalance::from(btc_balances_by_layer.pending),
                        encumbrance: BtcAccountBalance::from(btc_balances_by_layer.encumbrance),
                        all_layers: BtcAccountBalance::from(btc_balances_by_layer.all_layers_available),
                    }
                }
            }

            impl From<$module::balancesByLayer> for LayeredUsdAccountBalances {
                fn from(usd_balances_by_layer: $module::balancesByLayer) -> Self {
                    Self {
                        settled: UsdAccountBalance::from(usd_balances_by_layer.settled),
                        pending: UsdAccountBalance::from(usd_balances_by_layer.pending),
                        encumbrance: UsdAccountBalance::from(usd_balances_by_layer.encumbrance),
                        all_layers: UsdAccountBalance::from(usd_balances_by_layer.all_layers_available),
                    }
                }
            }

            impl From<$module::accountSetBalances> for LedgerAccountBalancesByCurrency {
                fn from(balances: $module::accountSetBalances) -> Self {
                    LedgerAccountBalancesByCurrency {
                        btc: balances.btc_balances.map_or_else(
                            LayeredBtcAccountBalances::default,
                            LayeredBtcAccountBalances::from,
                        ),
                        usd: balances.usd_balances.map_or_else(
                            LayeredUsdAccountBalances::default,
                            LayeredUsdAccountBalances::from,
                        ),
                        usdt: balances.usdt_balances.map_or_else(
                            LayeredUsdAccountBalances::default,
                            LayeredUsdAccountBalances::from,
                        ),
                    }
                }
            }

            impl From<$module::accountDetailsAndBalances> for LedgerAccountWithBalance {
                fn from(account: $module::accountDetailsAndBalances) -> Self {
                    let account_details = account.account_details;
                    LedgerAccountWithBalance {
                        id: account_details.account_id.into(),
                        name: account_details.name,
                        normal_balance_type: account_details.normal_balance_type.into(),
                        balance: LedgerAccountBalancesByCurrency {
                            btc: account.account_balances.btc_balances.map_or_else(
                                LayeredBtcAccountBalances::default,
                                LayeredBtcAccountBalances::from,
                            ),
                            usd: account.account_balances.usd_balances.map_or_else(
                                LayeredUsdAccountBalances::default,
                                LayeredUsdAccountBalances::from,
                            ),
                            usdt: account.account_balances.usdt_balances.map_or_else(
                                LayeredUsdAccountBalances::default,
                                LayeredUsdAccountBalances::from,
                            ),
                        },
                    }
                }
            }
        )+
    };
}

#[derive(Debug, Clone)]
pub struct LedgerAccountDetails {
    pub id: LedgerAccountId,
    pub code: String,
    pub name: String,
    pub normal_balance_type: LedgerDebitOrCredit,
}

macro_rules! impl_from_account_details_only {
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

            impl From<$module::accountDetails> for LedgerAccountDetails {
                fn from(account: $module::accountDetails) -> Self {
                    LedgerAccountDetails {
                        id: account.account_id.into(),
                        code: account.code,
                        name: account.name,
                        normal_balance_type: account.normal_balance_type.into(),
                    }
                }
            }
        )+
    };
}

impl_from_account_details_only!(account_set_and_sub_accounts, chart_of_accounts);

impl_from_account_details_and_balances!(
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
            debit: Satoshis::from_btc(debit_amount),
            credit: Satoshis::from_btc(credit_amount),
            net_normal: Satoshis::from_btc(net_amount_pos),
            net_debit: SignedSatoshis::from_btc(net_amount_neg),
        };

        let debit_normal_balance: BtcAccountBalance = btc_balance.into();

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
            debit: UsdCents::from_usd(debit_amount),
            credit: UsdCents::from_usd(credit_amount),
            net_normal: UsdCents::from_usd(net_amount_pos),
            net_debit: SignedUsdCents::from_usd(net_amount_neg),
        };

        let debit_normal_balance: UsdAccountBalance = usd_balance.into();

        assert_eq!(debit_normal_balance, expected_debit_normal_balance);
    }
}
