use trial_balance::DebitOrCredit;

use crate::primitives::{LedgerDebitOrCredit, Satoshis, UsdCents};

use super::cala::graphql::*;

#[derive(Debug, Clone)]
pub struct BtcAccountBalance {
    pub debit: Satoshis,
    pub credit: Satoshis,
    pub net: Satoshis,
}

impl From<trial_balance::balances> for BtcAccountBalance {
    fn from(balances: trial_balance::balances) -> Self {
        Self {
            debit: Satoshis::from_btc(balances.dr_balance.units),
            credit: Satoshis::from_btc(balances.cr_balance.units),
            net: Satoshis::from_btc(balances.normal_balance.units),
        }
    }
}

impl Default for BtcAccountBalance {
    fn default() -> Self {
        Self {
            debit: Satoshis::ZERO,
            credit: Satoshis::ZERO,
            net: Satoshis::ZERO,
        }
    }
}

impl BtcAccountBalance {
    fn as_debit_normal(&self) -> Self {
        let debit_normal_balance = self.debit - self.credit;
        debit_normal_balance.assert_same_absolute_size(&self.net);

        BtcAccountBalance {
            debit: self.debit,
            credit: self.credit,
            net: debit_normal_balance,
        }
    }

    fn as_credit_normal(&self) -> Self {
        let credit_normal_balance = self.credit - self.debit;
        credit_normal_balance.assert_same_absolute_size(&self.net);

        BtcAccountBalance {
            debit: self.debit,
            credit: self.credit,
            net: credit_normal_balance,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UsdAccountBalance {
    pub debit: UsdCents,
    pub credit: UsdCents,
    pub net: UsdCents,
}

impl From<trial_balance::balances> for UsdAccountBalance {
    fn from(balances: trial_balance::balances) -> Self {
        Self {
            debit: UsdCents::from_usd(balances.dr_balance.units),
            credit: UsdCents::from_usd(balances.cr_balance.units),
            net: UsdCents::from_usd(balances.normal_balance.units),
        }
    }
}

impl Default for UsdAccountBalance {
    fn default() -> Self {
        Self {
            debit: UsdCents::ZERO,
            credit: UsdCents::ZERO,
            net: UsdCents::ZERO,
        }
    }
}

impl UsdAccountBalance {
    fn as_debit_normal(&self) -> Self {
        let debit_normal_balance = self.debit - self.credit;
        debit_normal_balance.assert_same_absolute_size(&self.net);

        UsdAccountBalance {
            debit: self.debit,
            credit: self.credit,
            net: debit_normal_balance,
        }
    }

    fn as_credit_normal(&self) -> Self {
        let credit_normal_balance = self.credit - self.debit;
        credit_normal_balance.assert_same_absolute_size(&self.net);

        UsdAccountBalance {
            debit: self.debit,
            credit: self.credit,
            net: credit_normal_balance,
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

impl LayeredBtcAccountBalances {
    fn as_debit_normal(&self) -> Self {
        LayeredBtcAccountBalances {
            settled: self.settled.as_debit_normal(),
            pending: self.pending.as_debit_normal(),
            encumbrance: self.encumbrance.as_debit_normal(),
            all_layers: self.all_layers.as_debit_normal(),
        }
    }

    fn as_credit_normal(&self) -> Self {
        LayeredBtcAccountBalances {
            settled: self.settled.as_credit_normal(),
            pending: self.pending.as_credit_normal(),
            encumbrance: self.encumbrance.as_credit_normal(),
            all_layers: self.all_layers.as_credit_normal(),
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

impl LayeredUsdAccountBalances {
    fn as_debit_normal(&self) -> Self {
        LayeredUsdAccountBalances {
            settled: self.settled.as_debit_normal(),
            pending: self.pending.as_debit_normal(),
            encumbrance: self.encumbrance.as_debit_normal(),
            all_layers: self.all_layers.as_debit_normal(),
        }
    }

    fn as_credit_normal(&self) -> Self {
        LayeredUsdAccountBalances {
            settled: self.settled.as_credit_normal(),
            pending: self.pending.as_credit_normal(),
            encumbrance: self.encumbrance.as_credit_normal(),
            all_layers: self.all_layers.as_credit_normal(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LedgerAccountBalancesByCurrency {
    pub btc: LayeredBtcAccountBalances,
    pub usd: LayeredUsdAccountBalances,
    pub usdt: LayeredUsdAccountBalances,
}

impl LedgerAccountBalancesByCurrency {
    pub fn as_debit_normal(&self) -> Self {
        LedgerAccountBalancesByCurrency {
            btc: self.btc.as_debit_normal(),
            usd: self.usd.as_debit_normal(),
            usdt: self.usdt.as_debit_normal(),
        }
    }

    pub fn as_credit_normal(&self) -> Self {
        LedgerAccountBalancesByCurrency {
            btc: self.btc.as_credit_normal(),
            usd: self.usd.as_credit_normal(),
            usdt: self.usdt.as_credit_normal(),
        }
    }
}

impl From<DebitOrCredit> for LedgerDebitOrCredit {
    fn from(debit_or_credit: DebitOrCredit) -> Self {
        match debit_or_credit {
            DebitOrCredit::DEBIT => LedgerDebitOrCredit::Debit,
            DebitOrCredit::CREDIT => LedgerDebitOrCredit::Credit,
            DebitOrCredit::Other(_) => todo!(),
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

impl LedgerAccountBalance {
    pub fn as_debit_normal(&self) -> Self {
        LedgerAccountBalance {
            name: self.name.to_owned(),
            normal_balance_type: self.normal_balance_type,
            balance: self.balance.as_debit_normal(),
        }
    }

    pub fn as_credit_normal(&self) -> Self {
        LedgerAccountBalance {
            name: self.name.to_owned(),
            normal_balance_type: self.normal_balance_type,
            balance: self.balance.as_credit_normal(),
        }
    }
}
