use std::{fmt::Display, str::FromStr};

pub use audit::AuditInfo;
pub use authz::{AllOrOne, action_description::*};

es_entity::entity_id! {
    ReportId
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
    pub const REPORT_GENERATION_STATUS_READ: Self = CoreReportAction::Report(ReportEntityAction::GenerationStatusRead);

    pub fn entities() -> Vec<(
        CoreReportActionDiscriminants,
        Vec<ActionDescription<NoPath>>,
    )> {
        use CoreReportActionDiscriminants::*;

        let mut result = vec![];

        for entity in <CoreReportActionDiscriminants as strum::VariantArray>::VARIANTS {
            let actions = match entity {
                Report => ReportEntityAction::describe(),
            };

            result.push((*entity, actions));
        }

        result
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum ReportEntityAction {
    Generate,
    GenerationStatusRead,
}

impl ReportEntityAction {
    pub fn describe() -> Vec<ActionDescription<NoPath>> {
        let mut res = vec![];

        for variant in <Self as strum::VariantArray>::VARIANTS {
            let action_description = match variant {
                Self::Generate => ActionDescription::new(variant, &[PERMISSION_SET_REPORT_WRITER]),

                Self::GenerationStatusRead => ActionDescription::new(
                    variant,
                    &[PERMISSION_SET_REPORT_VIEWER, PERMISSION_SET_REPORT_WRITER],
                ),
            };
            res.push(action_description);
        }

        res
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
