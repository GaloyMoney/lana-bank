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

impl From<trial_balance::balances> for BtcAccountBalance {
    fn from(balances: trial_balance::balances) -> Self {
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

impl From<account_set_and_sub_accounts_with_balance::balances> for BtcAccountBalance {
    fn from(balances: account_set_and_sub_accounts_with_balance::balances) -> Self {
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

impl From<trial_balance::balances> for UsdAccountBalance {
    fn from(balances: trial_balance::balances) -> Self {
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

impl From<account_set_and_sub_accounts_with_balance::balances> for UsdAccountBalance {
    fn from(balances: account_set_and_sub_accounts_with_balance::balances) -> Self {
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

impl From<trial_balance::TrialBalanceAccountSetBtcBalances> for LayeredBtcAccountBalances {
    fn from(btc_balances_by_layer: trial_balance::TrialBalanceAccountSetBtcBalances) -> Self {
        Self {
            settled: BtcAccountBalance::from(btc_balances_by_layer.settled),
            pending: BtcAccountBalance::from(btc_balances_by_layer.pending),
            encumbrance: BtcAccountBalance::from(btc_balances_by_layer.encumbrance),
            all_layers: BtcAccountBalance::from(btc_balances_by_layer.all_layers_available),
        }
    }
}

impl From<account_set_and_sub_accounts_with_balance::AccountBalancesBtcBalances> for LayeredBtcAccountBalances {
    fn from(btc_balances_by_layer: account_set_and_sub_accounts_with_balance::AccountBalancesBtcBalances) -> Self {
        Self {
            settled: BtcAccountBalance::from(btc_balances_by_layer.settled),
            pending: BtcAccountBalance::from(btc_balances_by_layer.pending),
            encumbrance: BtcAccountBalance::from(btc_balances_by_layer.encumbrance),
            all_layers: BtcAccountBalance::from(btc_balances_by_layer.all_layers_available),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct LayeredUsdAccountBalances {
    pub settled: UsdAccountBalance,
    pub pending: UsdAccountBalance,
    pub encumbrance: UsdAccountBalance,
    pub all_layers: UsdAccountBalance,
}

impl From<trial_balance::TrialBalanceAccountSetUsdBalances> for LayeredUsdAccountBalances {
    fn from(usd_balances_by_layer: trial_balance::TrialBalanceAccountSetUsdBalances) -> Self {
        Self {
            settled: UsdAccountBalance::from(usd_balances_by_layer.settled),
            pending: UsdAccountBalance::from(usd_balances_by_layer.pending),
            encumbrance: UsdAccountBalance::from(usd_balances_by_layer.encumbrance),
            all_layers: UsdAccountBalance::from(usd_balances_by_layer.all_layers_available),
        }
    }
}

impl From<account_set_and_sub_accounts_with_balance::AccountBalancesUsdBalances> for LayeredUsdAccountBalances {
    fn from(usd_balances_by_layer: account_set_and_sub_accounts_with_balance::AccountBalancesUsdBalances) -> Self {
        Self {
            settled: UsdAccountBalance::from(usd_balances_by_layer.settled),
            pending: UsdAccountBalance::from(usd_balances_by_layer.pending),
            encumbrance: UsdAccountBalance::from(usd_balances_by_layer.encumbrance),
            all_layers: UsdAccountBalance::from(usd_balances_by_layer.all_layers_available),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LedgerAccountBalancesByCurrency {
    pub btc: LayeredBtcAccountBalances,
    pub usd: LayeredUsdAccountBalances,
    pub usdt: LayeredUsdAccountBalances,
}

impl From<trial_balance::DebitOrCredit> for LedgerDebitOrCredit {
    fn from(debit_or_credit: trial_balance::DebitOrCredit) -> Self {
        match debit_or_credit {
            trial_balance::DebitOrCredit::DEBIT => LedgerDebitOrCredit::Debit,
            trial_balance::DebitOrCredit::CREDIT => LedgerDebitOrCredit::Credit,
            trial_balance::DebitOrCredit::Other(_) => todo!(),
        }
    }
}

impl From<account_set_and_sub_accounts_with_balance::DebitOrCredit> for LedgerDebitOrCredit {
    fn from(debit_or_credit: account_set_and_sub_accounts_with_balance::DebitOrCredit) -> Self {
        match debit_or_credit {
            account_set_and_sub_accounts_with_balance::DebitOrCredit::DEBIT => LedgerDebitOrCredit::Debit,
            account_set_and_sub_accounts_with_balance::DebitOrCredit::CREDIT => LedgerDebitOrCredit::Credit,
            account_set_and_sub_accounts_with_balance::DebitOrCredit::Other(_) => todo!(),
        }
    }
}

impl From<chart_of_accounts::DebitOrCredit> for LedgerDebitOrCredit {
    fn from(debit_or_credit: chart_of_accounts::DebitOrCredit) -> Self {
        match debit_or_credit {
            chart_of_accounts::DebitOrCredit::DEBIT => LedgerDebitOrCredit::Debit,
            chart_of_accounts::DebitOrCredit::CREDIT => LedgerDebitOrCredit::Credit,
            chart_of_accounts::DebitOrCredit::Other(_) => todo!(),
        }
    }
}

impl From<chart_of_accounts_category_account::DebitOrCredit> for LedgerDebitOrCredit {
    fn from(debit_or_credit: chart_of_accounts_category_account::DebitOrCredit) -> Self {
        match debit_or_credit {
            chart_of_accounts_category_account::DebitOrCredit::DEBIT => LedgerDebitOrCredit::Debit,
            chart_of_accounts_category_account::DebitOrCredit::CREDIT => {
                LedgerDebitOrCredit::Credit
            }
            chart_of_accounts_category_account::DebitOrCredit::Other(_) => todo!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LedgerAccountBalance {
    pub name: String,
    pub normal_balance_type: LedgerDebitOrCredit,
    pub balance: LedgerAccountBalancesByCurrency,
}

impl From<trial_balance::TrialBalanceAccountSetMembersEdgesNodeOnAccount> for LedgerAccountBalance {
    fn from(node: trial_balance::TrialBalanceAccountSetMembersEdgesNodeOnAccount) -> Self {
        LedgerAccountBalance {
            name: node.name,
            normal_balance_type: node.normal_balance_type.into(),
            balance: LedgerAccountBalancesByCurrency {
                btc: node.btc_balances.map_or_else(
                    LayeredBtcAccountBalances::default,
                    LayeredBtcAccountBalances::from,
                ),
                usd: node.usd_balances.map_or_else(
                    LayeredUsdAccountBalances::default,
                    LayeredUsdAccountBalances::from,
                ),
                usdt: node.usdt_balances.map_or_else(
                    LayeredUsdAccountBalances::default,
                    LayeredUsdAccountBalances::from,
                ),
            },
        }
    }
}

impl From<account_set_and_sub_accounts_with_balance::SubAccountOnAccount> for LedgerAccountBalance {
    fn from(node: account_set_and_sub_accounts_with_balance::SubAccountOnAccount) -> Self {
        let account = node.account_with_balance;
        LedgerAccountBalance {
            name: account.name,
            normal_balance_type: account.normal_balance_type.into(),
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

#[derive(Debug, Clone)]
pub struct LedgerChartOfAccountsAccount {
    pub id: LedgerAccountId,
    pub code: String,
    pub name: String,
    pub normal_balance_type: LedgerDebitOrCredit,
}

impl From<chart_of_accounts::accountDetails> for LedgerChartOfAccountsAccount {
    fn from(account: chart_of_accounts::accountDetails) -> Self {
        LedgerChartOfAccountsAccount {
            id: account.account_id.into(),
            code: account.code,
            name: account.name,
            normal_balance_type: account.normal_balance_type.into(),
        }
    }
}

impl From<chart_of_accounts_category_account::accountDetails> for LedgerChartOfAccountsAccount {
    fn from(account: chart_of_accounts_category_account::accountDetails) -> Self {
        LedgerChartOfAccountsAccount {
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
