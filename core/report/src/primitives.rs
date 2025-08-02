use std::{fmt::Display, str::FromStr};

pub use audit::AuditInfo;
pub use authz::{ActionPermission, AllOrOne, action_description::*, auto_mappings};

es_entity::entity_id! {
    ReportId,
    ReportRunId
}

pub type ReportAllOrOne = AllOrOne<ReportId>;

pub const PERMISSION_SET_REPORT_VIEWER: &str = "report_viewer";
pub const PERMISSION_SET_REPORT_WRITER: &str = "report_writer";

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum ReportObject {
    Report(ReportAllOrOne),
}

impl ReportObject {
    pub fn all_reports() -> ReportObject {
        ReportObject::Report(AllOrOne::All)
    }
    pub fn report(id: impl Into<Option<ReportId>>) -> ReportObject {
        match id.into() {
            Some(id) => ReportObject::Report(AllOrOne::ById(id)),
            None => ReportObject::all_reports(),
        }
    }
}

impl Display for ReportObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let discriminant = ReportObjectDiscriminants::from(self);
        use ReportObject::*;
        match self {
            Report(obj_ref) => write!(f, "{discriminant}/{obj_ref}"),
        }
    }
}

impl FromStr for ReportObject {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, id) = s.split_once('/').expect("missing slash");
        use ReportObjectDiscriminants::*;
        let res = match entity.parse().expect("invalid entity") {
            Report => {
                let obj_ref = id.parse().map_err(|_| "could not parse ReportObject")?;
                ReportObject::Report(obj_ref)
            }
        };
        Ok(res)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString, strum::VariantArray))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreReportAction {
    Report(ReportEntityAction),
}

impl CoreReportAction {
    pub const REPORT_GENERATE: Self = CoreReportAction::Report(ReportEntityAction::Generate);
    pub const REPORT_READ: Self = CoreReportAction::Report(ReportEntityAction::Read);

    pub fn actions() -> Vec<ActionDescription> {
        use CoreReportActionDiscriminants::*;
        auto_mappings!(Report => ReportEntityAction)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum ReportEntityAction {
    Generate,
    Read,
}

impl ActionPermission for ReportEntityAction {
    fn permission_set(&self) -> &'static str {
        match self {
            Self::Read => PERMISSION_SET_REPORT_VIEWER,
            Self::Generate => PERMISSION_SET_REPORT_WRITER,
        }
    }
}

impl Display for CoreReportAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:", CoreReportActionDiscriminants::from(self))?;
        use CoreReportAction::*;
        match self {
            Report(action) => action.fmt(f),
        }
    }
}

impl FromStr for CoreReportAction {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, action) = s.split_once(':').expect("missing colon");
        use CoreReportActionDiscriminants::*;
        let res = match entity.parse()? {
            Report => CoreReportAction::from(action.parse::<ReportEntityAction>()?),
        };
        Ok(res)
    }
}

impl From<ReportEntityAction> for CoreReportAction {
    fn from(action: ReportEntityAction) -> Self {
        CoreReportAction::Report(action)
    }
}

impl From<airflow::reports_api_client::ReportRunState> for crate::report_run::ReportRunState {
    fn from(state: airflow::reports_api_client::ReportRunState) -> Self {
        match state {
            airflow::reports_api_client::ReportRunState::Queued => {
                crate::report_run::ReportRunState::Queued
            }
            airflow::reports_api_client::ReportRunState::Running => {
                crate::report_run::ReportRunState::Running
            }
            airflow::reports_api_client::ReportRunState::Success => {
                crate::report_run::ReportRunState::Success
            }
            airflow::reports_api_client::ReportRunState::Failed => {
                crate::report_run::ReportRunState::Failed
            }
        }
    }
}

impl From<airflow::reports_api_client::ReportRunType> for crate::report_run::ReportRunType {
    fn from(run_type: airflow::reports_api_client::ReportRunType) -> Self {
        match run_type {
            airflow::reports_api_client::ReportRunType::Scheduled => {
                crate::report_run::ReportRunType::Scheduled
            }
            airflow::reports_api_client::ReportRunType::Manual => {
                crate::report_run::ReportRunType::Manual
            }
        }
    }
}

impl From<airflow::reports_api_client::ReportFile> for crate::report::ReportFile {
    fn from(file: airflow::reports_api_client::ReportFile) -> Self {
        crate::report::ReportFile {
            extension: file.extension,
            path_in_bucket: file.path_in_bucket,
        }
    }
}
