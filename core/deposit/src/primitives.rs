use std::{fmt::Display, str::FromStr};

use authz::AllOrOne;

pub use chart_of_accounts::ChartId;
pub use governance::{ApprovalProcessId, GovernanceAction, GovernanceObject};

pub use cala_ledger::primitives::{
    AccountId as LedgerAccountId, AccountSetId as LedgerAccountSetId, EntryId as LedgerEntryId,
    JournalId as LedgerJournalId, TransactionId as LedgerTransactionId,
};

es_entity::entity_id! {
    DepositAccountHolderId,
    DepositAccountId,
    WithdrawalId,
    ChartOfAccountsIntegrationConfigId,
    DepositId;

    DepositAccountHolderId => core_customer::CustomerId,
    DepositAccountId => LedgerAccountId,
    DepositId => LedgerTransactionId,
    WithdrawalId => LedgerTransactionId,
    WithdrawalId => ApprovalProcessId
}

pub use core_money::UsdCents;

pub type DepositAccountAllOrOne = AllOrOne<DepositAccountId>;
pub type DepositAccountByHolderAllOrOne = AllOrOne<DepositAccountHolderId>;
pub type DepositAllOrOne = AllOrOne<DepositId>;
pub type ChartOfAccountsIntegrationConfigAllOrOne = AllOrOne<ChartOfAccountsIntegrationConfigId>;
pub type WithdrawalAllOrOne = AllOrOne<WithdrawalId>;

#[derive(Debug, Clone)]
pub struct LedgerOmnibusAccountIds {
    pub account_set_id: LedgerAccountSetId,
    pub account_id: LedgerAccountId,
}

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreDepositObject {
    DepositAccount(DepositAccountAllOrOne),
    Deposit(DepositAllOrOne),
    ChartOfAccountsIntegration(ChartOfAccountsIntegrationConfigAllOrOne),
    Withdrawal(WithdrawalAllOrOne),
}

impl CoreDepositObject {
    pub fn all_deposit_accounts() -> Self {
        CoreDepositObject::DepositAccount(AllOrOne::All)
    }

    pub fn deposit_account(id: DepositAccountId) -> Self {
        CoreDepositObject::DepositAccount(AllOrOne::ById(id))
    }

    pub fn all_deposits() -> Self {
        CoreDepositObject::Deposit(AllOrOne::All)
    }

    pub fn deposit(id: DepositId) -> Self {
        CoreDepositObject::Deposit(AllOrOne::ById(id))
    }

    pub fn all_withdrawals() -> Self {
        CoreDepositObject::Withdrawal(AllOrOne::All)
    }

    pub fn withdrawal(id: WithdrawalId) -> Self {
        CoreDepositObject::Withdrawal(AllOrOne::ById(id))
    }

    pub fn chart_of_accounts_integration() -> Self {
        CoreDepositObject::ChartOfAccountsIntegration(AllOrOne::All)
    }
}

impl Display for CoreDepositObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let discriminant = CoreDepositObjectDiscriminants::from(self);
        use CoreDepositObject::*;
        match self {
            DepositAccount(obj_ref) => write!(f, "{}/{}", discriminant, obj_ref),
            Deposit(obj_ref) => write!(f, "{}/{}", discriminant, obj_ref),
            Withdrawal(obj_ref) => write!(f, "{}/{}", discriminant, obj_ref),
            ChartOfAccountsIntegration(obj_ref) => write!(f, "{}/{}", discriminant, obj_ref),
        }
    }
}

impl FromStr for CoreDepositObject {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, id) = s.split_once('/').expect("missing slash");
        use CoreDepositObjectDiscriminants::*;
        let res = match entity.parse().expect("invalid entity") {
            DepositAccount => {
                let obj_ref = id
                    .parse()
                    .map_err(|_| "could not parse CoreDepositObject")?;
                CoreDepositObject::DepositAccount(obj_ref)
            }
            Deposit => {
                let obj_ref = id
                    .parse()
                    .map_err(|_| "could not parse CoreDepositObject")?;
                CoreDepositObject::Deposit(obj_ref)
            }
            Withdrawal => {
                let obj_ref = id
                    .parse()
                    .map_err(|_| "could not parse CoreDepositObject")?;
                CoreDepositObject::Withdrawal(obj_ref)
            }
            ChartOfAccountsIntegration => {
                let obj_ref = id
                    .parse()
                    .map_err(|_| "could not parse CoreDepositObject")?;
                CoreDepositObject::ChartOfAccountsIntegration(obj_ref)
            }
        };
        Ok(res)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreDepositAction {
    DepositAccount(DepositAccountAction),
    Deposit(DepositAction),
    ChartOfAccountsIntegrationConfig(ChartOfAccountsIntegrationConfigAction),
    Withdrawal(WithdrawalAction),
}

impl CoreDepositAction {
    pub const DEPOSIT_ACCOUNT_CREATE: Self =
        CoreDepositAction::DepositAccount(DepositAccountAction::Create);
    pub const DEPOSIT_ACCOUNT_READ_BALANCE: Self =
        CoreDepositAction::DepositAccount(DepositAccountAction::ReadBalance);
    pub const DEPOSIT_ACCOUNT_READ: Self =
        CoreDepositAction::DepositAccount(DepositAccountAction::Read);
    pub const DEPOSIT_ACCOUNT_LIST: Self =
        CoreDepositAction::DepositAccount(DepositAccountAction::List);

    pub const DEPOSIT_CREATE: Self = CoreDepositAction::Deposit(DepositAction::Create);
    pub const DEPOSIT_READ: Self = CoreDepositAction::Deposit(DepositAction::Read);
    pub const DEPOSIT_LIST: Self = CoreDepositAction::Deposit(DepositAction::List);

    pub const CHART_OF_ACCOUNTS_INTEGRATION_CONFIG_UPDATE: Self =
        CoreDepositAction::ChartOfAccountsIntegrationConfig(
            ChartOfAccountsIntegrationConfigAction::Update,
        );
    pub const CHART_OF_ACCOUNTS_INTEGRATION_CONFIG_READ: Self =
        CoreDepositAction::ChartOfAccountsIntegrationConfig(
            ChartOfAccountsIntegrationConfigAction::Read,
        );

    pub const WITHDRAWAL_INITIATE: Self = CoreDepositAction::Withdrawal(WithdrawalAction::Initiate);
    pub const WITHDRAWAL_CONCLUDE_APPROVAL_PROCESS: Self =
        CoreDepositAction::Withdrawal(WithdrawalAction::ConcludeApprovalProcess);
    pub const WITHDRAWAL_CANCEL: Self = CoreDepositAction::Withdrawal(WithdrawalAction::Cancel);
    pub const WITHDRAWAL_CONFIRM: Self = CoreDepositAction::Withdrawal(WithdrawalAction::Confirm);
    pub const WITHDRAWAL_READ: Self = CoreDepositAction::Withdrawal(WithdrawalAction::Read);
    pub const WITHDRAWAL_LIST: Self = CoreDepositAction::Withdrawal(WithdrawalAction::List);
}

impl Display for CoreDepositAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:", CoreDepositActionDiscriminants::from(self))?;
        use CoreDepositAction::*;
        match self {
            DepositAccount(action) => action.fmt(f),
            Deposit(action) => action.fmt(f),
            ChartOfAccountsIntegrationConfig(action) => action.fmt(f),
            Withdrawal(action) => action.fmt(f),
        }
    }
}

impl FromStr for CoreDepositAction {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, action) = s.split_once(':').expect("missing colon");
        use CoreDepositActionDiscriminants::*;
        let res = match entity.parse()? {
            DepositAccount => CoreDepositAction::from(action.parse::<DepositAccountAction>()?),
            Deposit => CoreDepositAction::from(action.parse::<DepositAction>()?),
            ChartOfAccountsIntegrationConfig => {
                CoreDepositAction::from(action.parse::<ChartOfAccountsIntegrationConfigAction>()?)
            }
            Withdrawal => CoreDepositAction::from(action.parse::<WithdrawalAction>()?),
        };

        Ok(res)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum DepositAccountAction {
    Create,
    ReadBalance,
    ReadTxHistory,
    Read,
    List,
}

impl From<DepositAccountAction> for CoreDepositAction {
    fn from(action: DepositAccountAction) -> Self {
        CoreDepositAction::DepositAccount(action)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum DepositAction {
    Create,
    Read,
    List,
}

impl From<DepositAction> for CoreDepositAction {
    fn from(action: DepositAction) -> Self {
        CoreDepositAction::Deposit(action)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum WithdrawalAction {
    Initiate,
    Cancel,
    Confirm,
    ConcludeApprovalProcess,
    Read,
    List,
}

impl From<WithdrawalAction> for CoreDepositAction {
    fn from(action: WithdrawalAction) -> Self {
        CoreDepositAction::Withdrawal(action)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum ChartOfAccountsIntegrationConfigAction {
    Read,
    Update,
}

impl From<ChartOfAccountsIntegrationConfigAction> for CoreDepositAction {
    fn from(action: ChartOfAccountsIntegrationConfigAction) -> Self {
        CoreDepositAction::ChartOfAccountsIntegrationConfig(action)
    }
}
