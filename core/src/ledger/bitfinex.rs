use crate::primitives::{BfxAddressType, BfxWithdrawalMethod};

use super::cala::graphql::*;

impl From<BfxAddressType> for bfx_address_backed_account_create::BfxAddressType {
    fn from(address_type: BfxAddressType) -> Self {
        match address_type {
            BfxAddressType::Bitcoin => bfx_address_backed_account_create::BfxAddressType::BTC,
            BfxAddressType::Tron => bfx_address_backed_account_create::BfxAddressType::TRX,
        }
    }
}

impl From<BfxWithdrawalMethod> for bfx_withdrawal_execute::BfxWithdrawalMethod {
    fn from(address_type: BfxWithdrawalMethod) -> Self {
        match address_type {
            BfxWithdrawalMethod::Bitcoin => bfx_withdrawal_execute::BfxWithdrawalMethod::BITCOIN,
            BfxWithdrawalMethod::TronUsdt => {
                bfx_withdrawal_execute::BfxWithdrawalMethod::TETHER_USX
            }
        }
    }
}

impl From<bfx_address_backed_account_by_id::BfxAddressBackedAccountByIdBitfinexAddressBackedAccount>
    for String
{
    fn from(
        account: bfx_address_backed_account_by_id::BfxAddressBackedAccountByIdBitfinexAddressBackedAccount,
    ) -> Self {
        account.address
    }
}
