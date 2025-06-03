mod cvl;

use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};

use std::str::FromStr;

use authz::{action_description::*, AllOrOne};

pub use cala_ledger::primitives::{
    AccountId as CalaAccountId, AccountSetId as CalaAccountSetId, Currency,
    DebitOrCredit as LedgerDebitOrCredit, JournalId as LedgerJournalId,
    TransactionId as LedgerTxId, TxTemplateId as LedgerTxTemplateId,
};
pub use core_customer::{CustomerId, CustomerType};
pub use core_money::*;
pub use core_price::PriceOfOneBTC;
pub use governance::ApprovalProcessId;

pub use cvl::*;

es_entity::entity_id! {
    CreditFacilityId,
    DisbursalId,
    PaymentId,
    PaymentAllocationId,
    ChartOfAccountsIntegrationConfigId,
    CollateralId,
    ObligationId,
    InterestAccrualCycleId;

    CreditFacilityId => governance::ApprovalProcessId,
    DisbursalId => governance::ApprovalProcessId,

    CreditFacilityId => job::JobId,
    InterestAccrualCycleId => job::JobId,
    ObligationId => job::JobId,

    DisbursalId => LedgerTxId,
    PaymentAllocationId => LedgerTxId,
}

#[cfg(feature = "schemars")]
mod entity_id_schemas {
    use super::*;
    use schemars::{gen::SchemaGenerator, schema::Schema, JsonSchema};

    impl JsonSchema for CreditFacilityId {
        fn schema_name() -> String {
            "CreditFacilityId".to_string()
        }

        fn json_schema(gen: &mut SchemaGenerator) -> Schema {
            String::json_schema(gen)
        }
    }

    impl JsonSchema for DisbursalId {
        fn schema_name() -> String {
            "DisbursalId".to_string()
        }

        fn json_schema(gen: &mut SchemaGenerator) -> Schema {
            String::json_schema(gen)
        }
    }

    impl JsonSchema for PaymentId {
        fn schema_name() -> String {
            "PaymentId".to_string()
        }

        fn json_schema(gen: &mut SchemaGenerator) -> Schema {
            String::json_schema(gen)
        }
    }

    impl JsonSchema for PaymentAllocationId {
        fn schema_name() -> String {
            "PaymentAllocationId".to_string()
        }

        fn json_schema(gen: &mut SchemaGenerator) -> Schema {
            String::json_schema(gen)
        }
    }

    impl JsonSchema for ChartOfAccountsIntegrationConfigId {
        fn schema_name() -> String {
            "ChartOfAccountsIntegrationConfigId".to_string()
        }

        fn json_schema(gen: &mut SchemaGenerator) -> Schema {
            String::json_schema(gen)
        }
    }

    impl JsonSchema for CollateralId {
        fn schema_name() -> String {
            "CollateralId".to_string()
        }

        fn json_schema(gen: &mut SchemaGenerator) -> Schema {
            String::json_schema(gen)
        }
    }

    impl JsonSchema for ObligationId {
        fn schema_name() -> String {
            "ObligationId".to_string()
        }

        fn json_schema(gen: &mut SchemaGenerator) -> Schema {
            String::json_schema(gen)
        }
    }

    impl JsonSchema for InterestAccrualCycleId {
        fn schema_name() -> String {
            "InterestAccrualCycleId".to_string()
        }

        fn json_schema(gen: &mut SchemaGenerator) -> Schema {
            String::json_schema(gen)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub enum ObligationStatus {
    NotYetDue,
    Due,
    Overdue,
    Defaulted,
    Paid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub enum ObligationType {
    Disbursal,
    Interest,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub enum BalanceUpdatedType {
    Disbursal,
    InterestAccrual,
}

impl From<ObligationType> for BalanceUpdatedType {
    fn from(obligation_type: ObligationType) -> Self {
        match obligation_type {
            ObligationType::Disbursal => Self::Disbursal,
            ObligationType::Interest => Self::InterestAccrual,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub enum BalanceUpdatedSource {
    Obligation(ObligationId),
    PaymentAllocation(PaymentAllocationId),
}

impl From<ObligationId> for BalanceUpdatedSource {
    fn from(obligation_id: ObligationId) -> Self {
        Self::Obligation(obligation_id)
    }
}

impl From<PaymentAllocationId> for BalanceUpdatedSource {
    fn from(allocation_id: PaymentAllocationId) -> Self {
        Self::PaymentAllocation(allocation_id)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct BalanceUpdateData {
    pub source_id: BalanceUpdatedSource,
    pub ledger_tx_id: LedgerTxId,
    pub balance_type: ObligationType,
    pub amount: UsdCents,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct LedgerOmnibusAccountIds {
    pub account_set_id: CalaAccountSetId,
    pub account_id: CalaAccountId,
}

pub type CreditFacilityAllOrOne = AllOrOne<CreditFacilityId>;
pub type ChartOfAccountsIntegrationConfigAllOrOne = AllOrOne<ChartOfAccountsIntegrationConfigId>;
pub type DisbursalAllOrOne = AllOrOne<DisbursalId>;
pub type ObligationAllOrOne = AllOrOne<ObligationId>;

pub const PERMISSION_SET_CREDIT_WRITER: &str = "credit_writer";
pub const PERMISSION_SET_CREDIT_VIEWER: &str = "credit_viewer";

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreCreditObject {
    CreditFacility(CreditFacilityAllOrOne),
    ChartOfAccountsIntegrationConfig(ChartOfAccountsIntegrationConfigAllOrOne),
    Disbursal(DisbursalAllOrOne),
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

    pub fn disbursal(id: DisbursalId) -> Self {
        CoreCreditObject::Disbursal(AllOrOne::ById(id))
    }

    pub fn all_disbursals() -> Self {
        CoreCreditObject::Disbursal(AllOrOne::All)
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
            CreditFacility(obj_ref) => write!(f, "{}/{}", discriminant, obj_ref),
            ChartOfAccountsIntegrationConfig(obj_ref) => write!(f, "{}/{}", discriminant, obj_ref),
            Disbursal(obj_ref) => write!(f, "{}/{}", discriminant, obj_ref),
            Obligation(obj_ref) => write!(f, "{}/{}", discriminant, obj_ref),
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
            Obligation => {
                let obj_ref = id.parse().map_err(|_| "could not parse CoreCreditObject")?;
                CoreCreditObject::Obligation(obj_ref)
            }
            Disbursal => {
                let obj_ref = id.parse().map_err(|_| "could not parse CoreCreditObject")?;
                CoreCreditObject::Disbursal(obj_ref)
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
    Disbursal(DisbursalAction),
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

    pub const DISBURSAL_INITIATE: Self = CoreCreditAction::Disbursal(DisbursalAction::Initiate);
    pub const DISBURSAL_SETTLE: Self = CoreCreditAction::Disbursal(DisbursalAction::Settle);
    pub const DISBURSAL_LIST: Self = CoreCreditAction::Disbursal(DisbursalAction::List);
    pub const DISBURSAL_READ: Self = CoreCreditAction::Disbursal(DisbursalAction::Read);

    pub const OBLIGATION_READ: Self = CoreCreditAction::Obligation(ObligationAction::Read);
    pub const OBLIGATION_UPDATE_STATUS: Self =
        CoreCreditAction::Obligation(ObligationAction::UpdateStatus);
    pub const OBLIGATION_RECORD_PAYMENT: Self =
        CoreCreditAction::Obligation(ObligationAction::RecordPaymentAllocation);

    pub fn entities() -> Vec<(
        CoreCreditActionDiscriminants,
        Vec<ActionDescription<NoPath>>,
    )> {
        use CoreCreditActionDiscriminants::*;

        let mut result = vec![];

        for entity in <CoreCreditActionDiscriminants as strum::VariantArray>::VARIANTS {
            let actions = match entity {
                CreditFacility => CreditFacilityAction::describe(),
                ChartOfAccountsIntegrationConfig => {
                    ChartOfAccountsIntegrationConfigAction::describe()
                }
                Disbursal => DisbursalAction::describe(),
                Obligation => ObligationAction::describe(),
            };

            result.push((*entity, actions));
        }

        result
    }
}

impl std::fmt::Display for CoreCreditAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:", CoreCreditActionDiscriminants::from(self))?;
        use CoreCreditAction::*;
        match self {
            CreditFacility(action) => action.fmt(f),
            ChartOfAccountsIntegrationConfig(action) => action.fmt(f),
            Disbursal(action) => action.fmt(f),
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
            Disbursal => CoreCreditAction::from(action.parse::<DisbursalAction>()?),
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
}

impl CreditFacilityAction {
    pub fn describe() -> Vec<ActionDescription<NoPath>> {
        let mut res = vec![];

        for variant in <Self as strum::VariantArray>::VARIANTS {
            let action_description = match variant {
                Self::Create => ActionDescription::new(variant, &[PERMISSION_SET_CREDIT_WRITER]),
                Self::Read => ActionDescription::new(
                    variant,
                    &[PERMISSION_SET_CREDIT_VIEWER, PERMISSION_SET_CREDIT_WRITER],
                ),
                Self::List => ActionDescription::new(
                    variant,
                    &[PERMISSION_SET_CREDIT_VIEWER, PERMISSION_SET_CREDIT_WRITER],
                ),
                Self::ConcludeApprovalProcess => {
                    ActionDescription::new(variant, &[PERMISSION_SET_CREDIT_WRITER])
                }
                Self::Activate => ActionDescription::new(variant, &[PERMISSION_SET_CREDIT_WRITER]),
                Self::UpdateCollateral => {
                    ActionDescription::new(variant, &[PERMISSION_SET_CREDIT_WRITER])
                }
                Self::RecordInterest => {
                    ActionDescription::new(variant, &[PERMISSION_SET_CREDIT_WRITER])
                }
                Self::Complete => ActionDescription::new(variant, &[PERMISSION_SET_CREDIT_WRITER]),
                Self::UpdateCollateralizationState => {
                    ActionDescription::new(variant, &[PERMISSION_SET_CREDIT_WRITER])
                }
            };
            res.push(action_description);
        }

        res
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

impl DisbursalAction {
    pub fn describe() -> Vec<ActionDescription<NoPath>> {
        let mut res = vec![];

        for variant in <Self as strum::VariantArray>::VARIANTS {
            let action_description = match variant {
                Self::Initiate => ActionDescription::new(variant, &[PERMISSION_SET_CREDIT_WRITER]),
                Self::Settle => ActionDescription::new(variant, &[PERMISSION_SET_CREDIT_WRITER]),
                Self::List => ActionDescription::new(
                    variant,
                    &[PERMISSION_SET_CREDIT_VIEWER, PERMISSION_SET_CREDIT_WRITER],
                ),
                Self::Read => ActionDescription::new(
                    variant,
                    &[PERMISSION_SET_CREDIT_VIEWER, PERMISSION_SET_CREDIT_WRITER],
                ),
            };
            res.push(action_description);
        }

        res
    }
}

impl From<DisbursalAction> for CoreCreditAction {
    fn from(action: DisbursalAction) -> Self {
        Self::Disbursal(action)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum ChartOfAccountsIntegrationConfigAction {
    Read,
    Update,
}

impl ChartOfAccountsIntegrationConfigAction {
    pub fn describe() -> Vec<ActionDescription<NoPath>> {
        let mut res = vec![];

        for variant in <Self as strum::VariantArray>::VARIANTS {
            let action_description = match variant {
                Self::Read => ActionDescription::new(
                    variant,
                    &[PERMISSION_SET_CREDIT_WRITER, PERMISSION_SET_CREDIT_VIEWER],
                ),
                Self::Update => ActionDescription::new(variant, &[PERMISSION_SET_CREDIT_WRITER]),
            };
            res.push(action_description);
        }

        res
    }
}

impl From<ChartOfAccountsIntegrationConfigAction> for CoreCreditAction {
    fn from(action: ChartOfAccountsIntegrationConfigAction) -> Self {
        CoreCreditAction::ChartOfAccountsIntegrationConfig(action)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum ObligationAction {
    Read,
    UpdateStatus,
    RecordPaymentAllocation,
}

impl ObligationAction {
    pub fn describe() -> Vec<ActionDescription<NoPath>> {
        let mut res = vec![];

        for variant in <Self as strum::VariantArray>::VARIANTS {
            let action_description = match variant {
                Self::Read => ActionDescription::new(
                    variant,
                    &[PERMISSION_SET_CREDIT_VIEWER, PERMISSION_SET_CREDIT_WRITER],
                ),
                Self::UpdateStatus => {
                    ActionDescription::new(variant, &[PERMISSION_SET_CREDIT_WRITER])
                }
                Self::RecordPaymentAllocation => {
                    ActionDescription::new(variant, &[PERMISSION_SET_CREDIT_WRITER])
                }
            };
            res.push(action_description);
        }

        res
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
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub enum CreditFacilityStatus {
    #[default]
    PendingCollateralization,
    PendingApproval,
    Active,
    Matured,
    Closed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "graphql", derive(async_graphql::Enum))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub enum DisbursalStatus {
    New,
    Approved,
    Denied,
    Confirmed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Hash, Deserialize, sqlx::Type)]
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
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub enum CollateralAction {
    Add,
    Remove,
}

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Serialize,
    Deserialize,
    Eq,
    strum::Display,
    strum::EnumString,
)]
#[cfg_attr(feature = "graphql", derive(async_graphql::Enum))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub enum CollateralizationState {
    FullyCollateralized,
    UnderMarginCallThreshold,
    UnderLiquidationThreshold,
    #[default]
    NoCollateral,
}

pub struct CollateralUpdate {
    pub tx_id: LedgerTxId,
    pub abs_diff: Satoshis,
    pub action: CollateralAction,
    pub effective: chrono::NaiveDate,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
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

pub struct EffectiveDate(chrono::NaiveDate);

impl From<chrono::NaiveDate> for EffectiveDate {
    fn from(date: chrono::NaiveDate) -> Self {
        Self(date)
    }
}

impl EffectiveDate {
    pub fn end_of_day(&self) -> DateTime<Utc> {
        Utc.from_utc_datetime(
            &self
                .0
                .and_hms_opt(23, 59, 59)
                .expect("23:59:59 was invalid"),
        )
    }
}
