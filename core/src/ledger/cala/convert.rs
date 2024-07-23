use super::graphql::*;
use crate::primitives::{LedgerAccountId, LedgerAccountSetMemberType, LedgerTxTemplateId};

impl From<account_by_external_id::AccountByExternalIdAccountByExternalId> for LedgerAccountId {
    fn from(account: account_by_external_id::AccountByExternalIdAccountByExternalId) -> Self {
        LedgerAccountId::from(account.account_id)
    }
}

impl From<account_by_code::AccountByCodeAccountByCode> for LedgerAccountId {
    fn from(account: account_by_code::AccountByCodeAccountByCode) -> Self {
        LedgerAccountId::from(account.account_id)
    }
}

impl From<tx_template_by_code::TxTemplateByCodeTxTemplateByCode> for LedgerTxTemplateId {
    fn from(tx_template_by_code: tx_template_by_code::TxTemplateByCodeTxTemplateByCode) -> Self {
        LedgerTxTemplateId::from(tx_template_by_code.tx_template_id)
    }
}

impl From<LedgerAccountSetMemberType> for add_to_account_set::AccountSetMemberType {
    fn from(member_type: LedgerAccountSetMemberType) -> Self {
        match member_type {
            LedgerAccountSetMemberType::Account => Self::ACCOUNT,
            LedgerAccountSetMemberType::AccountSet => Self::ACCOUNT_SET,
        }
    }
}
