use crate::primitives::{
    LedgerAccountId, LedgerDebitOrCredit, Satoshis, SignedSatoshis, SignedUsdCents, UsdCents,
};

use super::cala::graphql::*;

macro_rules! impl_from_debit_or_credit {
    ($($t:ty),+) => {
        $(
            impl From<$t> for LedgerDebitOrCredit {
                fn from(debit_or_credit: $t) -> Self {
                    match debit_or_credit {
                        <$t>::DEBIT => LedgerDebitOrCredit::Debit,
                        <$t>::CREDIT => LedgerDebitOrCredit::Credit,
                        _ => todo!()
                    }
                }
            }
        )+
    };
}

impl_from_debit_or_credit!(
    trial_balance::DebitOrCredit,
    account_set_and_sub_accounts_with_balance::DebitOrCredit,
    chart_of_accounts::DebitOrCredit,
    balance_sheet::DebitOrCredit,
    profit_and_loss_statement::DebitOrCredit,
    account_set_and_sub_accounts::DebitOrCredit
);

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

macro_rules! impl_from_balances {
    ($($t:ty),+) => {
        $(
            impl From<$t> for BtcAccountBalance {
                fn from(balances: $t) -> Self {
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

            impl From<$t> for UsdAccountBalance {
                fn from(balances: $t) -> Self {
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
        )+
    };
}

impl_from_balances!(
    trial_balance::balances,
    balance_sheet::balances,
    profit_and_loss_statement::balances,
    account_set_and_sub_accounts_with_balance::balances
);

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

macro_rules! impl_from_layered_balances {
    ($($t:ty),+) => {
        $(
            impl From<$t> for LayeredBtcAccountBalances {
                fn from(btc_balances_by_layer: $t) -> Self {
                    Self {
                        settled: BtcAccountBalance::from(btc_balances_by_layer.settled),
                        pending: BtcAccountBalance::from(btc_balances_by_layer.pending),
                        encumbrance: BtcAccountBalance::from(btc_balances_by_layer.encumbrance),
                        all_layers: BtcAccountBalance::from(btc_balances_by_layer.all_layers_available),
                    }
                }
            }

            impl From<$t> for LayeredUsdAccountBalances {
                fn from(usd_balances_by_layer: $t) -> Self {
                    Self {
                        settled: UsdAccountBalance::from(usd_balances_by_layer.settled),
                        pending: UsdAccountBalance::from(usd_balances_by_layer.pending),
                        encumbrance: UsdAccountBalance::from(usd_balances_by_layer.encumbrance),
                        all_layers: UsdAccountBalance::from(usd_balances_by_layer.all_layers_available),
                    }
                            }
            }

        )+
    };
}

impl_from_layered_balances!(
    trial_balance::balancesByLayer,
    balance_sheet::balancesByLayer,
    profit_and_loss_statement::balancesByLayer,
    account_set_and_sub_accounts_with_balance::balancesByLayer
);

#[derive(Debug, Clone)]
pub struct LedgerAccountBalancesByCurrency {
    pub btc: LayeredBtcAccountBalances,
    pub usd: LayeredUsdAccountBalances,
    pub usdt: LayeredUsdAccountBalances,
}

macro_rules! impl_from_balances_by_currency {
    ($($t:ty),+) => {
        $(
            impl From<$t> for LedgerAccountBalancesByCurrency {
                fn from(balances: $t) -> Self {
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
        )+
    };
}

impl_from_balances_by_currency!(
    trial_balance::accountSetBalances,
    balance_sheet::accountSetBalances,
    profit_and_loss_statement::accountSetBalances,
    account_set_and_sub_accounts_with_balance::accountSetBalances
);

#[derive(Debug, Clone)]
pub struct LedgerAccountWithBalance {
    pub id: LedgerAccountId,
    pub name: String,
    pub normal_balance_type: LedgerDebitOrCredit,
    pub balance: LedgerAccountBalancesByCurrency,
}

macro_rules! impl_from_account_with_balance {
    ($($t:ty),+) => {
        $(
            impl From<$t> for LedgerAccountWithBalance {
                fn from(account: $t) -> Self {
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

impl_from_account_with_balance!(
    trial_balance::accountDetailsAndBalances,
    balance_sheet::accountDetailsAndBalances,
    profit_and_loss_statement::accountDetailsAndBalances,
    account_set_and_sub_accounts_with_balance::accountDetailsAndBalances
);

#[derive(Debug, Clone)]
pub struct LedgerAccountDetails {
    pub id: LedgerAccountId,
    pub code: String,
    pub name: String,
    pub normal_balance_type: LedgerDebitOrCredit,
}

impl From<chart_of_accounts::accountDetails> for LedgerAccountDetails {
    fn from(account: chart_of_accounts::accountDetails) -> Self {
        LedgerAccountDetails {
            id: account.account_id.into(),
            code: account.code,
            name: account.name,
            normal_balance_type: account.normal_balance_type.into(),
        }
    }
}

impl From<account_set_and_sub_accounts::accountDetails> for LedgerAccountDetails {
    fn from(account: account_set_and_sub_accounts::accountDetails) -> Self {
        LedgerAccountDetails {
            id: account.account_id.into(),
            code: account.code,
            name: account.name,
            normal_balance_type: account.normal_balance_type.into(),
        }
    }
}

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
