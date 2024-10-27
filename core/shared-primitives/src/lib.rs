#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

es_entity::entity_id! { UserId }
es_entity::entity_id! { CommitteeId }
es_entity::entity_id! { CustomerId }
es_entity::entity_id! { LineOfCreditContractId }
es_entity::entity_id! { WithdrawId }
es_entity::entity_id! { DepositId }
es_entity::entity_id! { DocumentId }
es_entity::entity_id! { LoanId }
es_entity::entity_id! { CreditFacilityId }
es_entity::entity_id! { DisbursementId }
es_entity::entity_id! { InterestAccrualId }
es_entity::entity_id! { TermsTemplateId }
es_entity::entity_id! { ReportId }

pub use job::JobId;

impl From<LoanId> for JobId {
    fn from(id: LoanId) -> Self {
        JobId::from(id.0)
    }
}
impl From<CreditFacilityId> for JobId {
    fn from(id: CreditFacilityId) -> Self {
        JobId::from(id.0)
    }
}
impl From<ReportId> for JobId {
    fn from(id: ReportId) -> Self {
        JobId::from(id.0)
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum AllOrOne<T> {
    All,
    ById(T),
}

impl<T> std::str::FromStr for AllOrOne<T>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "*" => Ok(AllOrOne::All),
            _ => {
                let id = T::from_str(s).map_err(|e| format!("Invalid ID: {}", e))?;
                Ok(AllOrOne::ById(id))
            }
        }
    }
}

impl<T> std::fmt::Display for AllOrOne<T>
where
    T: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AllOrOne::All => write!(f, "*"),
            AllOrOne::ById(id) => write!(f, "{}", id),
        }
    }
}
