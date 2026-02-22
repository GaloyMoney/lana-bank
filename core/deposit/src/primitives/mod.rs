pub(crate) mod account_sets;
pub use account_sets::{
    DEPOSIT_ACCOUNT_SET_CATALOG, DepositAccountSetCatalog, DepositOmnibusAccountSetSpec,
    DepositSummaryAccountSetSpec,
};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use std::{fmt::Display, str::FromStr};

use authz::{ActionPermission, AllOrOne, action_description::*, map_action};

pub const DEPOSIT_APPROVAL: audit::SystemActor = audit::SystemActor::new("deposit-approval");

pub use core_accounting_primitives::ChartId;
pub use core_customer::CustomerType;
pub use governance::{ApprovalProcessId, GovernanceAction, GovernanceObject};
pub use public_id::PublicId;

pub use cala_ledger::primitives::{
    AccountId as CalaAccountId, AccountSetId as CalaAccountSetId, EntryId as CalaEntryId,
    JournalId as CalaJournalId, TransactionId as CalaTransactionId,
};

es_entity::entity_id! {
    DepositAccountHolderId,
    DepositAccountId,
    WithdrawalId,
    ChartOfAccountsIntegrationConfigId,
    DepositId;

    DepositAccountHolderId => core_customer::CustomerId,
    DepositAccountId => CalaAccountId,
    DepositAccountId => public_id::PublicIdTargetId,
    DepositId => public_id::PublicIdTargetId,
    DepositId => CalaTransactionId,
    WithdrawalId => CalaTransactionId,
    WithdrawalId => public_id::PublicIdTargetId,
    WithdrawalId => ApprovalProcessId
}

pub use money::UsdCents;

pub const DEPOSIT_ACCOUNT_ENTITY_TYPE: core_accounting_primitives::EntityType =
    core_accounting_primitives::EntityType::new("DepositAccount");
pub const DEPOSIT_TRANSACTION_ENTITY_TYPE: core_accounting_primitives::EntityType =
    core_accounting_primitives::EntityType::new("Deposit");
pub const WITHDRAWAL_TRANSACTION_ENTITY_TYPE: core_accounting_primitives::EntityType =
    core_accounting_primitives::EntityType::new("Withdrawal");

pub type DepositAccountAllOrOne = AllOrOne<DepositAccountId>;
pub type DepositAccountByHolderAllOrOne = AllOrOne<DepositAccountHolderId>;
pub type DepositAllOrOne = AllOrOne<DepositId>;
pub type ChartOfAccountsIntegrationConfigAllOrOne = AllOrOne<ChartOfAccountsIntegrationConfigId>;
pub type WithdrawalAllOrOne = AllOrOne<WithdrawalId>;

permission_sets_macro::permission_sets! {
    DepositViewer("Can view deposit accounts, balances, transaction history, and withdrawal details"),
    DepositWriter("Can create and manage deposits, deposit accounts, and withdrawals, including initiating, confirming, cancelling, and approving withdrawals"),
    DepositFreeze("Can freeze deposit accounts"),
    DepositUnfreeze("Can unfreeze deposit accounts"),
}

pub const DEPOSIT_ACCOUNT_REF_TARGET: public_id::PublicIdTargetType =
    public_id::PublicIdTargetType::new("deposit_account");
pub const DEPOSIT_REF_TARGET: public_id::PublicIdTargetType =
    public_id::PublicIdTargetType::new("deposit");
pub const WITHDRAWAL_REF_TARGET: public_id::PublicIdTargetType =
    public_id::PublicIdTargetType::new("withdrawal");

#[derive(Debug, Clone)]
pub struct LedgerOmnibusAccountIds {
    pub account_set_id: CalaAccountSetId,
    pub account_id: CalaAccountId,
}

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreDepositObject {
    DepositAccount(DepositAccountAllOrOne),
    Deposit(DepositAllOrOne),
    ChartOfAccountsIntegrationConfig(ChartOfAccountsIntegrationConfigAllOrOne),
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
        CoreDepositObject::ChartOfAccountsIntegrationConfig(AllOrOne::All)
    }
}

impl Display for CoreDepositObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let discriminant = CoreDepositObjectDiscriminants::from(self);
        use CoreDepositObject::*;
        match self {
            DepositAccount(obj_ref) => write!(f, "{discriminant}/{obj_ref}"),
            Deposit(obj_ref) => write!(f, "{discriminant}/{obj_ref}"),
            Withdrawal(obj_ref) => write!(f, "{discriminant}/{obj_ref}"),
            ChartOfAccountsIntegrationConfig(obj_ref) => write!(f, "{discriminant}/{obj_ref}"),
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
            ChartOfAccountsIntegrationConfig => {
                let obj_ref = id
                    .parse()
                    .map_err(|_| "could not parse CoreDepositObject")?;
                CoreDepositObject::ChartOfAccountsIntegrationConfig(obj_ref)
            }
        };
        Ok(res)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString, strum::VariantArray))]
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
    pub const DEPOSIT_ACCOUNT_UPDATE_STATUS: Self =
        CoreDepositAction::DepositAccount(DepositAccountAction::UpdateStatus);
    pub const DEPOSIT_ACCOUNT_READ_BALANCE: Self =
        CoreDepositAction::DepositAccount(DepositAccountAction::ReadBalance);
    pub const DEPOSIT_ACCOUNT_READ: Self =
        CoreDepositAction::DepositAccount(DepositAccountAction::Read);
    pub const DEPOSIT_ACCOUNT_LIST: Self =
        CoreDepositAction::DepositAccount(DepositAccountAction::List);
    pub const DEPOSIT_ACCOUNT_FREEZE: Self =
        CoreDepositAction::DepositAccount(DepositAccountAction::Freeze);
    pub const DEPOSIT_ACCOUNT_UNFREEZE: Self =
        CoreDepositAction::DepositAccount(DepositAccountAction::Unfreeze);
    pub const DEPOSIT_ACCOUNT_CLOSE: Self =
        CoreDepositAction::DepositAccount(DepositAccountAction::Close);

    pub const DEPOSIT_CREATE: Self = CoreDepositAction::Deposit(DepositAction::Create);
    pub const DEPOSIT_READ: Self = CoreDepositAction::Deposit(DepositAction::Read);
    pub const DEPOSIT_LIST: Self = CoreDepositAction::Deposit(DepositAction::List);
    pub const DEPOSIT_REVERT: Self = CoreDepositAction::Deposit(DepositAction::Revert);

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
    pub const WITHDRAWAL_REVERT: Self = CoreDepositAction::Withdrawal(WithdrawalAction::Revert);

    pub fn actions() -> Vec<ActionMapping> {
        use CoreDepositActionDiscriminants::*;
        use strum::VariantArray;

        CoreDepositActionDiscriminants::VARIANTS
            .iter()
            .flat_map(|&discriminant| match discriminant {
                DepositAccount => {
                    map_action!(deposit, DepositAccount, DepositAccountAction)
                }
                Deposit => map_action!(deposit, Deposit, DepositAction),
                ChartOfAccountsIntegrationConfig => map_action!(
                    deposit,
                    ChartOfAccountsIntegrationConfig,
                    ChartOfAccountsIntegrationConfigAction
                ),
                Withdrawal => map_action!(deposit, Withdrawal, WithdrawalAction),
            })
            .collect()
    }
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

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum DepositAccountAction {
    Create,
    UpdateStatus,
    ReadBalance,
    ReadTxHistory,
    Read,
    List,
    Freeze,
    Unfreeze,
    Close,
}

impl ActionPermission for DepositAccountAction {
    fn permission_set(&self) -> &'static str {
        match self {
            Self::Read | Self::List | Self::ReadBalance | Self::ReadTxHistory => {
                PERMISSION_SET_DEPOSIT_VIEWER
            }
            Self::Create | Self::UpdateStatus | Self::Close => PERMISSION_SET_DEPOSIT_WRITER,
            Self::Freeze => PERMISSION_SET_DEPOSIT_FREEZE,
            Self::Unfreeze => PERMISSION_SET_DEPOSIT_UNFREEZE,
        }
    }
}

impl From<DepositAccountAction> for CoreDepositAction {
    fn from(action: DepositAccountAction) -> Self {
        CoreDepositAction::DepositAccount(action)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum DepositAction {
    Create,
    Read,
    List,
    Revert,
}

impl ActionPermission for DepositAction {
    fn permission_set(&self) -> &'static str {
        match self {
            Self::Read | Self::List => PERMISSION_SET_DEPOSIT_VIEWER,
            Self::Create | Self::Revert => PERMISSION_SET_DEPOSIT_WRITER,
        }
    }
}

impl From<DepositAction> for CoreDepositAction {
    fn from(action: DepositAction) -> Self {
        CoreDepositAction::Deposit(action)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum WithdrawalAction {
    Initiate,
    Cancel,
    Confirm,
    ConcludeApprovalProcess,
    Read,
    List,
    Revert,
}

impl ActionPermission for WithdrawalAction {
    fn permission_set(&self) -> &'static str {
        match self {
            Self::Read | Self::List => PERMISSION_SET_DEPOSIT_VIEWER,
            Self::Cancel
            | Self::Initiate
            | Self::ConcludeApprovalProcess
            | Self::Confirm
            | Self::Revert => PERMISSION_SET_DEPOSIT_WRITER,
        }
    }
}

impl From<WithdrawalAction> for CoreDepositAction {
    fn from(action: WithdrawalAction) -> Self {
        CoreDepositAction::Withdrawal(action)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum ChartOfAccountsIntegrationConfigAction {
    Read,
    Update,
}

impl ActionPermission for ChartOfAccountsIntegrationConfigAction {
    fn permission_set(&self) -> &'static str {
        match self {
            Self::Read => PERMISSION_SET_DEPOSIT_VIEWER,
            Self::Update => PERMISSION_SET_DEPOSIT_WRITER,
        }
    }
}

impl From<ChartOfAccountsIntegrationConfigAction> for CoreDepositAction {
    fn from(action: ChartOfAccountsIntegrationConfigAction) -> Self {
        CoreDepositAction::ChartOfAccountsIntegrationConfig(action)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "graphql", derive(async_graphql::Enum))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum DepositAccountStatus {
    Inactive,
    Active,
    Frozen,
    Closed,
}

#[derive(Debug, Clone, Copy)]
pub enum DepositAccountHolderStatus {
    Active,
    Inactive,
}

impl From<DepositAccountHolderStatus> for DepositAccountStatus {
    fn from(status: DepositAccountHolderStatus) -> Self {
        match status {
            DepositAccountHolderStatus::Active => DepositAccountStatus::Active,
            DepositAccountHolderStatus::Inactive => DepositAccountStatus::Inactive,
        }
    }
}

#[derive(Clone, Copy)]
pub enum DepositAccountType {
    Individual,
    GovernmentEntity,
    PrivateCompany,
    Bank,
    FinancialInstitution,
    NonDomiciledCompany,
}

impl From<CustomerType> for DepositAccountType {
    fn from(customer_type: CustomerType) -> Self {
        match customer_type {
            CustomerType::Individual => DepositAccountType::Individual,
            CustomerType::GovernmentEntity => DepositAccountType::GovernmentEntity,
            CustomerType::PrivateCompany => DepositAccountType::PrivateCompany,
            CustomerType::Bank => DepositAccountType::Bank,
            CustomerType::FinancialInstitution => DepositAccountType::FinancialInstitution,
            CustomerType::NonDomiciledCompany => DepositAccountType::NonDomiciledCompany,
            _ => panic!("Invalid customer type"),
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[cfg_attr(feature = "graphql", derive(async_graphql::Enum))]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub enum DepositStatus {
    Confirmed,
    Reverted,
}
