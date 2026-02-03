pub(crate) mod account_sets;
mod liquidation_payment;

use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use std::str::FromStr;

use authz::{ActionPermission, AllOrOne, action_description::*, map_action};

pub use cala_ledger::primitives::{
    AccountId as CalaAccountId, AccountSetId as CalaAccountSetId, Currency,
    DebitOrCredit as LedgerDebitOrCredit, JournalId as LedgerJournalId,
    TransactionId as LedgerTxId, TxTemplateId as LedgerTxTemplateId,
};
pub use core_custody::WalletId as CustodyWalletId;
pub use core_customer::{CustomerId, CustomerType};
pub use core_price::PriceOfOneBTC;
pub use governance::ApprovalProcessId;
pub use money::*;
pub use public_id::PublicId;

pub use core_credit_terms::{
    AnnualRatePct, CVLPct, DisbursalPolicy, EffectiveDate, FacilityDuration, FacilityDurationType,
    InterestInterval, InterestPeriod, ObligationDuration, OneTimeFeeRatePct, TermValues,
    TermValuesBuilder, TermsTemplateId,
    collateralization::{
        CollateralizationRatio, CollateralizationState, PendingCreditFacilityCollateralizationState,
    },
};

pub use core_credit_collection::{
    CoreCreditCollectionAction, CoreCreditCollectionEvent, CoreCreditCollectionObject,
    ObligationAction, ObligationAllOrOne, ObligationError, ObligationId, ObligationStatus,
    ObligationType, Obligations, ObligationsAmounts, PERMISSION_SET_COLLECTION_PAYMENT_DATE,
    PERMISSION_SET_COLLECTION_VIEWER, PERMISSION_SET_COLLECTION_WRITER, Payment, PaymentAllocation,
    PaymentAllocationError, PaymentAllocationId, PaymentError, PaymentId, PaymentSourceAccountId,
    Payments,
};

impl From<FacilityDurationType> for DisbursedReceivableAccountCategory {
    fn from(duration_type: FacilityDurationType) -> Self {
        match duration_type {
            FacilityDurationType::LongTerm => DisbursedReceivableAccountCategory::LongTerm,
            FacilityDurationType::ShortTerm => DisbursedReceivableAccountCategory::ShortTerm,
        }
    }
}

pub use liquidation_payment::LiquidationPayment;

es_entity::entity_id! {
    CreditFacilityProposalId,
    PendingCreditFacilityId,
    CreditFacilityId,
    DisbursalId,
    ChartOfAccountsIntegrationConfigId,
    CollateralId,
    LiquidationId,
    InterestAccrualCycleId,
    FiscalYearId;

    CreditFacilityProposalId => PendingCreditFacilityId,

    CreditFacilityProposalId => CreditFacilityId,
    PendingCreditFacilityId => CreditFacilityId,

    CreditFacilityId => governance::ApprovalProcessId,
    CreditFacilityProposalId => governance::ApprovalProcessId,
    DisbursalId => governance::ApprovalProcessId,

    CreditFacilityId => job::JobId,
    InterestAccrualCycleId => job::JobId,

    DisbursalId => LedgerTxId,

    CreditFacilityId => public_id::PublicIdTargetId,
    DisbursalId => public_id::PublicIdTargetId,

    CreditFacilityId => core_credit_collection::BeneficiaryId,
}

#[derive(Debug, Clone)]
pub struct LedgerOmnibusAccountIds {
    pub account_set_id: CalaAccountSetId,
    pub account_id: CalaAccountId,
}

pub const CREDIT_FACILITY_ENTITY_TYPE: core_accounting::EntityType =
    core_accounting::EntityType::new("CreditFacility");
pub const CREDIT_FACILITY_PROPOSAL_ENTITY_TYPE: core_accounting::EntityType =
    core_accounting::EntityType::new("CreditFacilityProposal");
pub const COLLATERAL_ENTITY_TYPE: core_accounting::EntityType =
    core_accounting::EntityType::new("Collateral");
pub const DISBURSAL_TRANSACTION_ENTITY_TYPE: core_accounting::EntityType =
    core_accounting::EntityType::new("Disbursal");

pub type CreditFacilityAllOrOne = AllOrOne<CreditFacilityId>;
pub type ChartOfAccountsIntegrationConfigAllOrOne = AllOrOne<ChartOfAccountsIntegrationConfigId>;
pub type CollateralAllOrOne = AllOrOne<CollateralId>;
pub type DisbursalAllOrOne = AllOrOne<DisbursalId>;
pub type LiquidationAllOrOne = AllOrOne<LiquidationId>;
pub const PERMISSION_SET_CREDIT_WRITER: &str = "credit_writer";
pub const PERMISSION_SET_CREDIT_VIEWER: &str = "credit_viewer";

pub const CREDIT_FACILITY_REF_TARGET: public_id::PublicIdTargetType =
    public_id::PublicIdTargetType::new("credit_facility");
pub const DISBURSAL_REF_TARGET: public_id::PublicIdTargetType =
    public_id::PublicIdTargetType::new("disbursal");

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreCreditObject {
    CreditFacility(CreditFacilityAllOrOne),
    ChartOfAccountsIntegrationConfig(ChartOfAccountsIntegrationConfigAllOrOne),
    Collateral(CollateralAllOrOne),
    Disbursal(DisbursalAllOrOne),
    Liquidation(LiquidationAllOrOne),
    Obligation(ObligationAllOrOne),
}

impl CoreCreditObject {
    pub fn all_credit_facilities() -> Self {
        CoreCreditObject::CreditFacility(AllOrOne::All)
    }

    pub fn credit_facility(id: CreditFacilityId) -> Self {
        CoreCreditObject::CreditFacility(AllOrOne::ById(id))
    }

    pub fn chart_of_accounts_integration() -> Self {
        CoreCreditObject::ChartOfAccountsIntegrationConfig(AllOrOne::All)
    }

    pub fn collateral(id: CollateralId) -> Self {
        CoreCreditObject::Collateral(AllOrOne::ById(id))
    }

    pub fn all_collaterals() -> Self {
        CoreCreditObject::Collateral(AllOrOne::All)
    }

    pub fn disbursal(id: DisbursalId) -> Self {
        CoreCreditObject::Disbursal(AllOrOne::ById(id))
    }

    pub fn all_disbursals() -> Self {
        CoreCreditObject::Disbursal(AllOrOne::All)
    }

    pub fn liquidation(id: LiquidationId) -> Self {
        CoreCreditObject::Liquidation(AllOrOne::ById(id))
    }

    pub fn all_liquidations() -> Self {
        CoreCreditObject::Liquidation(AllOrOne::All)
    }

    pub fn obligation(id: ObligationId) -> Self {
        CoreCreditObject::Obligation(AllOrOne::ById(id))
    }

    pub fn all_obligations() -> Self {
        CoreCreditObject::Obligation(AllOrOne::All)
    }
}

impl std::fmt::Display for CoreCreditObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let discriminant = CoreCreditObjectDiscriminants::from(self);
        use CoreCreditObject::*;
        match self {
            CreditFacility(obj_ref) => write!(f, "{discriminant}/{obj_ref}"),
            ChartOfAccountsIntegrationConfig(obj_ref) => write!(f, "{discriminant}/{obj_ref}"),
            Collateral(obj_ref) => write!(f, "{discriminant}/{obj_ref}"),
            Disbursal(obj_ref) => write!(f, "{discriminant}/{obj_ref}"),
            Liquidation(obj_ref) => write!(f, "{discriminant}/{obj_ref}"),
            Obligation(obj_ref) => write!(f, "{discriminant}/{obj_ref}"),
        }
    }
}

impl FromStr for CoreCreditObject {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, id) = s.split_once('/').expect("missing slash");
        use CoreCreditObjectDiscriminants::*;
        let res = match entity.parse().expect("invalid entity") {
            CreditFacility => {
                let obj_ref = id.parse().map_err(|_| "could not parse CoreCreditObject")?;
                CoreCreditObject::CreditFacility(obj_ref)
            }
            ChartOfAccountsIntegrationConfig => {
                let obj_ref = id.parse().map_err(|_| "could not parse CoreCreditObject")?;
                CoreCreditObject::ChartOfAccountsIntegrationConfig(obj_ref)
            }
            Collateral => {
                let obj_ref = id.parse().map_err(|_| "could not parse CoreCreditObject")?;
                CoreCreditObject::Collateral(obj_ref)
            }
            Obligation => {
                let obj_ref = id.parse().map_err(|_| "could not parse CoreCreditObject")?;
                CoreCreditObject::Obligation(obj_ref)
            }
            Disbursal => {
                let obj_ref = id.parse().map_err(|_| "could not parse CoreCreditObject")?;
                CoreCreditObject::Disbursal(obj_ref)
            }
            Liquidation => {
                let obj_ref = id.parse().map_err(|_| "could not parse CoreCreditObject")?;
                CoreCreditObject::Liquidation(obj_ref)
            }
        };
        Ok(res)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString, strum::VariantArray))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreCreditAction {
    CreditFacility(CreditFacilityAction),
    ChartOfAccountsIntegrationConfig(ChartOfAccountsIntegrationConfigAction),
    Collateral(CollateralAction),
    Disbursal(DisbursalAction),
    Liquidation(LiquidationAction),
    Obligation(ObligationAction),
}

impl CoreCreditAction {
    pub const CREDIT_FACILITY_CREATE: Self =
        CoreCreditAction::CreditFacility(CreditFacilityAction::Create);
    pub const CREDIT_FACILITY_READ: Self =
        CoreCreditAction::CreditFacility(CreditFacilityAction::Read);
    pub const CREDIT_FACILITY_LIST: Self =
        CoreCreditAction::CreditFacility(CreditFacilityAction::List);
    pub const CREDIT_FACILITY_CONCLUDE_APPROVAL_PROCESS: Self =
        CoreCreditAction::CreditFacility(CreditFacilityAction::ConcludeApprovalProcess);
    pub const CREDIT_FACILITY_CUSTOMER_APPROVE: Self =
        CoreCreditAction::CreditFacility(CreditFacilityAction::CustomerApprove);
    pub const CREDIT_FACILITY_ACTIVATE: Self =
        CoreCreditAction::CreditFacility(CreditFacilityAction::Activate);
    pub const CREDIT_FACILITY_RECORD_INTEREST: Self =
        CoreCreditAction::CreditFacility(CreditFacilityAction::RecordInterest);
    pub const CREDIT_FACILITY_COMPLETE: Self =
        CoreCreditAction::CreditFacility(CreditFacilityAction::Complete);
    pub const CREDIT_FACILITY_UPDATE_COLLATERAL: Self =
        CoreCreditAction::CreditFacility(CreditFacilityAction::UpdateCollateral);
    pub const CREDIT_FACILITY_UPDATE_COLLATERALIZATION_STATE: Self =
        CoreCreditAction::CreditFacility(CreditFacilityAction::UpdateCollateralizationState);

    pub const CHART_OF_ACCOUNTS_INTEGRATION_CONFIG_READ: Self =
        CoreCreditAction::ChartOfAccountsIntegrationConfig(
            ChartOfAccountsIntegrationConfigAction::Read,
        );
    pub const CHART_OF_ACCOUNTS_INTEGRATION_CONFIG_UPDATE: Self =
        CoreCreditAction::ChartOfAccountsIntegrationConfig(
            ChartOfAccountsIntegrationConfigAction::Update,
        );

    pub const COLLATERAL_RECORD_LIQUIDATION_UPDATE: Self =
        CoreCreditAction::Collateral(CollateralAction::RecordLiquidationUpdate);
    pub const COLLATERAL_RECORD_PAYMENT_RECEIVED_FROM_LIQUIDATION: Self =
        CoreCreditAction::Collateral(CollateralAction::RecordPaymentReceived);

    pub const DISBURSAL_INITIATE: Self = CoreCreditAction::Disbursal(DisbursalAction::Initiate);
    pub const DISBURSAL_SETTLE: Self = CoreCreditAction::Disbursal(DisbursalAction::Settle);
    pub const DISBURSAL_LIST: Self = CoreCreditAction::Disbursal(DisbursalAction::List);
    pub const DISBURSAL_READ: Self = CoreCreditAction::Disbursal(DisbursalAction::Read);

    pub const LIQUIDATION_LIST: Self = CoreCreditAction::Liquidation(LiquidationAction::List);
    pub const LIQUIDATION_READ: Self = CoreCreditAction::Liquidation(LiquidationAction::Read);

    pub const OBLIGATION_READ: Self = CoreCreditAction::Obligation(ObligationAction::Read);
    pub const OBLIGATION_UPDATE_STATUS: Self =
        CoreCreditAction::Obligation(ObligationAction::UpdateStatus);
    pub const OBLIGATION_RECORD_PAYMENT: Self =
        CoreCreditAction::Obligation(ObligationAction::RecordPaymentAllocation);
    pub const OBLIGATION_RECORD_PAYMENT_WITH_DATE: Self =
        CoreCreditAction::Obligation(ObligationAction::RecordPaymentAllocationWithDate);

    pub fn actions() -> Vec<ActionMapping> {
        use CoreCreditActionDiscriminants::*;
        use strum::VariantArray;

        CoreCreditActionDiscriminants::VARIANTS
            .iter()
            .flat_map(|&discriminant| match discriminant {
                CreditFacility => map_action!(credit, CreditFacility, CreditFacilityAction),
                ChartOfAccountsIntegrationConfig => map_action!(
                    credit,
                    ChartOfAccountsIntegrationConfig,
                    ChartOfAccountsIntegrationConfigAction
                ),
                Collateral => map_action!(credit, Collateral, CollateralAction),
                Disbursal => map_action!(credit, Disbursal, DisbursalAction),
                Liquidation => map_action!(credit, Liquidation, LiquidationAction),
                Obligation => map_action!(credit, Obligation, ObligationAction),
            })
            .collect()
    }
}

impl std::fmt::Display for CoreCreditAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:", CoreCreditActionDiscriminants::from(self))?;
        use CoreCreditAction::*;
        match self {
            CreditFacility(action) => action.fmt(f),
            ChartOfAccountsIntegrationConfig(action) => action.fmt(f),
            Collateral(action) => action.fmt(f),
            Disbursal(action) => action.fmt(f),
            Liquidation(action) => action.fmt(f),
            Obligation(action) => action.fmt(f),
        }
    }
}

impl FromStr for CoreCreditAction {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut elems = s.split(':');
        let entity = elems.next().expect("missing first element");
        let action = elems.next().expect("missing second element");
        use CoreCreditActionDiscriminants::*;
        let res = match entity.parse()? {
            CreditFacility => CoreCreditAction::from(action.parse::<CreditFacilityAction>()?),
            ChartOfAccountsIntegrationConfig => {
                CoreCreditAction::from(action.parse::<ChartOfAccountsIntegrationConfigAction>()?)
            }
            Collateral => CoreCreditAction::from(action.parse::<CollateralAction>()?),
            Disbursal => CoreCreditAction::from(action.parse::<DisbursalAction>()?),
            Liquidation => CoreCreditAction::from(action.parse::<LiquidationAction>()?),
            Obligation => CoreCreditAction::from(action.parse::<ObligationAction>()?),
        };
        Ok(res)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum CreditFacilityAction {
    Create,
    Read,
    List,
    ConcludeApprovalProcess,
    Activate,
    UpdateCollateral,
    RecordInterest,
    Complete,
    UpdateCollateralizationState,
    CustomerApprove,
}

impl ActionPermission for CreditFacilityAction {
    fn permission_set(&self) -> &'static str {
        match self {
            Self::Read | Self::List => PERMISSION_SET_CREDIT_VIEWER,
            Self::Create
            | Self::ConcludeApprovalProcess
            | Self::Activate
            | Self::UpdateCollateral
            | Self::RecordInterest
            | Self::Complete
            | Self::CustomerApprove
            | Self::UpdateCollateralizationState => PERMISSION_SET_CREDIT_WRITER,
        }
    }
}

impl From<CreditFacilityAction> for CoreCreditAction {
    fn from(action: CreditFacilityAction) -> Self {
        Self::CreditFacility(action)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum DisbursalAction {
    Initiate,
    Settle,
    List,
    Read,
}

impl ActionPermission for DisbursalAction {
    fn permission_set(&self) -> &'static str {
        match self {
            Self::List | Self::Read => PERMISSION_SET_CREDIT_VIEWER,
            Self::Initiate | Self::Settle => PERMISSION_SET_CREDIT_WRITER,
        }
    }
}

impl From<DisbursalAction> for CoreCreditAction {
    fn from(action: DisbursalAction) -> Self {
        Self::Disbursal(action)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum CollateralAction {
    RecordLiquidationUpdate,
    RecordPaymentReceived,
}

impl ActionPermission for CollateralAction {
    fn permission_set(&self) -> &'static str {
        match self {
            Self::RecordLiquidationUpdate | Self::RecordPaymentReceived => {
                PERMISSION_SET_CREDIT_WRITER
            }
        }
    }
}

impl From<CollateralAction> for CoreCreditAction {
    fn from(action: CollateralAction) -> Self {
        Self::Collateral(action)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum LiquidationAction {
    List,
    Read,
}

impl ActionPermission for LiquidationAction {
    fn permission_set(&self) -> &'static str {
        match self {
            Self::List | Self::Read => PERMISSION_SET_CREDIT_VIEWER,
        }
    }
}

impl From<LiquidationAction> for CoreCreditAction {
    fn from(action: LiquidationAction) -> Self {
        Self::Liquidation(action)
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
            Self::Read => PERMISSION_SET_CREDIT_VIEWER,
            Self::Update => PERMISSION_SET_CREDIT_WRITER,
        }
    }
}

impl From<ChartOfAccountsIntegrationConfigAction> for CoreCreditAction {
    fn from(action: ChartOfAccountsIntegrationConfigAction) -> Self {
        CoreCreditAction::ChartOfAccountsIntegrationConfig(action)
    }
}

impl From<ObligationAction> for CoreCreditAction {
    fn from(action: ObligationAction) -> Self {
        Self::Obligation(action)
    }
}

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    strum::Display,
    strum::EnumString,
)]
#[cfg_attr(feature = "graphql", derive(async_graphql::Enum))]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub enum CreditFacilityStatus {
    #[default]
    Active,
    Matured,
    Closed,
}

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    strum::Display,
    strum::EnumString,
)]
#[cfg_attr(feature = "graphql", derive(async_graphql::Enum))]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub enum CreditFacilityProposalStatus {
    #[default]
    PendingCustomerApproval,
    CustomerDenied,
    PendingApproval,
    Approved,
    Denied,
}

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    strum::Display,
    strum::EnumString,
)]
#[cfg_attr(feature = "graphql", derive(async_graphql::Enum))]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub enum PendingCreditFacilityStatus {
    #[default]
    PendingCollateralization,
    Completed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "graphql", derive(async_graphql::Enum))]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub enum DisbursalStatus {
    New,
    Approved,
    Denied,
    Confirmed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Hash, Deserialize, sqlx::Type)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct InterestAccrualCycleIdx(i32);
impl std::fmt::Display for InterestAccrualCycleIdx {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl InterestAccrualCycleIdx {
    pub const FIRST: Self = Self(1);
    pub const fn next(&self) -> Self {
        Self(self.0 + 1)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
#[cfg_attr(feature = "graphql", derive(async_graphql::Enum))]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub enum CollateralDirection {
    Add,
    Remove,
}

pub struct CollateralUpdate {
    pub tx_id: LedgerTxId,
    pub abs_diff: Satoshis,
    pub direction: CollateralDirection,
    pub effective: chrono::NaiveDate,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub enum DisbursedReceivableAccountType {
    Individual,
    GovernmentEntity,
    PrivateCompany,
    Bank,
    FinancialInstitution,
    ForeignAgencyOrSubsidiary,
    NonDomiciledCompany,
}

impl From<CustomerType> for DisbursedReceivableAccountType {
    fn from(customer_type: CustomerType) -> Self {
        match customer_type {
            CustomerType::Individual => Self::Individual,
            CustomerType::GovernmentEntity => Self::GovernmentEntity,
            CustomerType::PrivateCompany => Self::PrivateCompany,
            CustomerType::Bank => Self::Bank,
            CustomerType::FinancialInstitution => Self::FinancialInstitution,
            CustomerType::ForeignAgencyOrSubsidiary => Self::ForeignAgencyOrSubsidiary,
            CustomerType::NonDomiciledCompany => Self::NonDomiciledCompany,
        }
    }
}

pub enum InterestReceivableAccountType {
    Individual,
    GovernmentEntity,
    PrivateCompany,
    Bank,
    FinancialInstitution,
    ForeignAgencyOrSubsidiary,
    NonDomiciledCompany,
}

impl From<CustomerType> for InterestReceivableAccountType {
    fn from(customer_type: CustomerType) -> Self {
        match customer_type {
            CustomerType::Individual => Self::Individual,
            CustomerType::GovernmentEntity => Self::GovernmentEntity,
            CustomerType::PrivateCompany => Self::PrivateCompany,
            CustomerType::Bank => Self::Bank,
            CustomerType::FinancialInstitution => Self::FinancialInstitution,
            CustomerType::ForeignAgencyOrSubsidiary => Self::ForeignAgencyOrSubsidiary,
            CustomerType::NonDomiciledCompany => Self::NonDomiciledCompany,
        }
    }
}

pub enum DisbursedReceivableAccountCategory {
    LongTerm,
    ShortTerm,
    Overdue,
}
