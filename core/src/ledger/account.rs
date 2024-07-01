use trial_balance::DebitOrCredit;

use crate::primitives::{LedgerDebitOrCredit, Satoshis, UsdCents};

use super::{cala::graphql::*, LedgerError};

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

#[derive(Debug, Clone)]
pub struct DebitNormalBtcAccountBalance {
    pub debit: Satoshis,
    pub credit: Satoshis,
    pub net_debit: Satoshis,
}

impl TryFrom<BtcAccountBalance> for DebitNormalBtcAccountBalance {
    type Error = LedgerError;

    fn try_from(btc_balance: BtcAccountBalance) -> Result<Self, LedgerError> {
        let debit_normal_balance = btc_balance.debit - btc_balance.credit;
        if !debit_normal_balance.check_same_absolute_size(&btc_balance.net) {
            return Err(LedgerError::CouldNotConvertAccountBalance);
        }

        Ok(DebitNormalBtcAccountBalance {
            debit: btc_balance.debit,
            credit: btc_balance.credit,
            net_debit: debit_normal_balance,
        })
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

#[derive(Debug, Clone)]
pub struct DebitNormalUsdAccountBalance {
    pub debit: UsdCents,
    pub credit: UsdCents,
    pub net_debit: UsdCents,
}

impl TryFrom<UsdAccountBalance> for DebitNormalUsdAccountBalance {
    type Error = LedgerError;

    fn try_from(usd_balance: UsdAccountBalance) -> Result<Self, LedgerError> {
        let debit_normal_balance = usd_balance.debit - usd_balance.credit;
        if !debit_normal_balance.check_same_absolute_size(&usd_balance.net) {
            return Err(LedgerError::CouldNotConvertAccountBalance);
        }

        Ok(DebitNormalUsdAccountBalance {
            debit: usd_balance.debit,
            credit: usd_balance.credit,
            net_debit: debit_normal_balance,
        })
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

#[derive(Debug, Clone)]
pub struct DebitNormalLayeredBtcAccountBalances {
    pub settled: DebitNormalBtcAccountBalance,
    pub pending: DebitNormalBtcAccountBalance,
    pub encumbrance: DebitNormalBtcAccountBalance,
    pub all_layers: DebitNormalBtcAccountBalance,
}

impl TryFrom<LayeredBtcAccountBalances> for DebitNormalLayeredBtcAccountBalances {
    type Error = LedgerError;

    fn try_from(balances: LayeredBtcAccountBalances) -> Result<Self, LedgerError> {
        Ok(DebitNormalLayeredBtcAccountBalances {
            settled: balances.settled.try_into()?,
            pending: balances.pending.try_into()?,
            encumbrance: balances.encumbrance.try_into()?,
            all_layers: balances.all_layers.try_into()?,
        })
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

#[derive(Debug, Clone)]
pub struct DebitNormalLayeredUsdAccountBalances {
    pub settled: DebitNormalUsdAccountBalance,
    pub pending: DebitNormalUsdAccountBalance,
    pub encumbrance: DebitNormalUsdAccountBalance,
    pub all_layers: DebitNormalUsdAccountBalance,
}

impl TryFrom<LayeredUsdAccountBalances> for DebitNormalLayeredUsdAccountBalances {
    type Error = LedgerError;

    fn try_from(balances: LayeredUsdAccountBalances) -> Result<Self, LedgerError> {
        Ok(DebitNormalLayeredUsdAccountBalances {
            settled: balances.settled.try_into()?,
            pending: balances.pending.try_into()?,
            encumbrance: balances.encumbrance.try_into()?,
            all_layers: balances.all_layers.try_into()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct LedgerAccountBalancesByCurrency {
    pub btc: LayeredBtcAccountBalances,
    pub usd: LayeredUsdAccountBalances,
    pub usdt: LayeredUsdAccountBalances,
}

#[derive(Debug, Clone)]
pub struct DebitNormalLedgerAccountBalancesByCurrency {
    pub btc: DebitNormalLayeredBtcAccountBalances,
    pub usd: DebitNormalLayeredUsdAccountBalances,
    pub usdt: DebitNormalLayeredUsdAccountBalances,
}

impl TryFrom<LedgerAccountBalancesByCurrency> for DebitNormalLedgerAccountBalancesByCurrency {
    type Error = LedgerError;

    fn try_from(balances: LedgerAccountBalancesByCurrency) -> Result<Self, LedgerError> {
        Ok(DebitNormalLedgerAccountBalancesByCurrency {
            btc: balances.btc.try_into()?,
            usd: balances.usd.try_into()?,
            usdt: balances.usdt.try_into()?,
        })
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

#[derive(Debug, Clone)]
pub struct DebitNormalLedgerAccountBalance {
    pub name: String,
    pub normal_balance_type: LedgerDebitOrCredit,
    pub balance: DebitNormalLedgerAccountBalancesByCurrency,
}

impl TryFrom<LedgerAccountBalance> for DebitNormalLedgerAccountBalance {
    type Error = LedgerError;

    fn try_from(balance: LedgerAccountBalance) -> Result<Self, LedgerError> {
        Ok(DebitNormalLedgerAccountBalance {
            name: balance.name,
            normal_balance_type: balance.normal_balance_type,
            balance: balance.balance.try_into()?,
        })
    }
}
