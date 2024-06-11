use crate::primitives::{BfxAddressType, LedgerAccountId, LedgerAccountSetId};

use super::cala::graphql::*;

pub struct BfxIntegrationAccountsAndSets {
    pub omnibus_account_set_id: LedgerAccountSetId,
    pub withdrawal_account_id: LedgerAccountId,
}

impl From<bfx_integration_create::BfxIntegrationCreateBitfinexIntegrationCreateIntegration>
    for BfxIntegrationAccountsAndSets
{
    fn from(
        bfx_integration: bfx_integration_create::BfxIntegrationCreateBitfinexIntegrationCreateIntegration,
    ) -> Self {
        BfxIntegrationAccountsAndSets {
            omnibus_account_set_id: LedgerAccountSetId::from(
                bfx_integration.omnibus_account_set_id,
            ),
            withdrawal_account_id: LedgerAccountId::from(bfx_integration.withdrawal_account_id),
        }
    }
}

impl From<bfx_integration_by_id::BfxIntegrationByIdBitfinexIntegration>
    for BfxIntegrationAccountsAndSets
{
    fn from(bfx_integration: bfx_integration_by_id::BfxIntegrationByIdBitfinexIntegration) -> Self {
        BfxIntegrationAccountsAndSets {
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

impl From<bfx_address_backed_account_by_id::BfxAddressBackedAccountByIdBitfinexAddressBackedAccount>
    for String
{
    fn from(
        account: bfx_address_backed_account_by_id::BfxAddressBackedAccountByIdBitfinexAddressBackedAccount,
    ) -> Self {
        account.address
    }
}
