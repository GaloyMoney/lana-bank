use crate::primitives::{
    BfxAddressType, BfxIntegrationId, BfxWithdrawalMethod, LedgerAccountId, LedgerAccountSetId,
};

use super::cala::graphql::*;

pub struct BfxIntegration {
    pub id: BfxIntegrationId,
    pub omnibus_account_set_id: LedgerAccountSetId,
    pub withdrawal_account_id: LedgerAccountId,
}

impl From<bfx_integration_create::BfxIntegrationCreateBitfinexIntegrationCreateIntegration>
    for BfxIntegration
{
    fn from(
        bfx_integration: bfx_integration_create::BfxIntegrationCreateBitfinexIntegrationCreateIntegration,
    ) -> Self {
        BfxIntegration {
            id: BfxIntegrationId::from(bfx_integration.integration_id),
            omnibus_account_set_id: LedgerAccountSetId::from(
                bfx_integration.omnibus_account_set_id,
            ),
            withdrawal_account_id: LedgerAccountId::from(bfx_integration.withdrawal_account_id),
        }
    }
}

impl From<bfx_integration_by_id::BfxIntegrationByIdBitfinexIntegration> for BfxIntegration {
    fn from(bfx_integration: bfx_integration_by_id::BfxIntegrationByIdBitfinexIntegration) -> Self {
        BfxIntegration {
            id: BfxIntegrationId::from(bfx_integration.integration_id),
            omnibus_account_set_id: LedgerAccountSetId::from(
                bfx_integration.omnibus_account_set_id,
            ),
            withdrawal_account_id: LedgerAccountId::from(bfx_integration.withdrawal_account_id),
        }
    }
}

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
