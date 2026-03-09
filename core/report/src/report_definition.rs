use std::{fmt, sync::OnceLock};

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use crate::error::ReportError;

pub const MANUAL_SINGLE_REPORT_TAG_KEY: &str = "lana/manual_single_report";
pub const REPORT_DEFINITION_ID_TAG_KEY: &str = "lana/report_definition_id";
pub const REPORT_NORM_TAG_KEY: &str = "lana/report_norm";
pub const REPORT_NAME_TAG_KEY: &str = "lana/report_name";
pub const AS_OF_DATE_TAG_KEY: &str = "lana/as_of_date";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(transparent)]
pub struct ReportDefinitionId(String);

impl ReportDefinitionId {
    pub fn try_new(value: impl Into<String>) -> Result<Self, ReportError> {
        let value = value.into();
        if value.contains('/') {
            Ok(Self(value))
        } else {
            Err(ReportError::InvalidReportRunRequest(format!(
                "invalid report definition id '{}': expected 'norm/id' format",
                value
            )))
        }
    }

    /// Create from norm and id parts
    pub fn from_parts(norm: &str, id: &str) -> Self {
        Self(format!("{}/{}", norm, id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ReportDefinitionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum ReportOutputFormat {
    Csv,
    Txt,
    Xml,
}

impl ReportOutputFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Csv => "csv",
            Self::Txt => "txt",
            Self::Xml => "xml",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct ReportDefinitionOutput {
    #[serde(rename = "type")]
    pub format: ReportOutputFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct ReportDefinition {
    pub norm: String,
    pub id: String,
    pub friendly_name: String,
    pub source_table: String,
    pub outputs: Vec<ReportDefinitionOutput>,
    pub supports_as_of: bool,
}

impl ReportDefinition {
    pub fn report_definition_id(&self) -> ReportDefinitionId {
        ReportDefinitionId::from_parts(&self.norm, &self.id)
    }

    pub fn validate_as_of_date(
        &self,
        as_of_date: Option<NaiveDate>,
    ) -> Result<(), ReportError> {
        if self.supports_as_of && as_of_date.is_none() {
            return Err(ReportError::InvalidReportRunRequest(format!(
                "report definition '{}' requires an as_of_date",
                self.report_definition_id()
            )));
        }

        if !self.supports_as_of && as_of_date.is_some() {
            return Err(ReportError::InvalidReportRunRequest(format!(
                "report definition '{}' does not support as_of_date",
                self.report_definition_id()
            )));
        }

        Ok(())
    }

    pub fn asset_selection_paths(&self) -> Vec<Vec<String>> {
        self.outputs
            .iter()
            .map(|output| {
                vec![
                    "file_report".to_string(),
                    format!("{}_{}", self.source_table, output.format.as_str()),
                ]
            })
            .collect()
    }
}

#[derive(Debug, Deserialize)]
struct ReportsFile {
    report_jobs: Vec<RawReportDefinition>,
}

#[derive(Debug, Deserialize)]
struct RawReportDefinition {
    norm: String,
    id: String,
    friendly_name: String,
    source_table: String,
    outputs: Vec<ReportDefinitionOutput>,
    #[serde(default)]
    supports_as_of: bool,
}

static REPORT_DEFINITIONS: OnceLock<Vec<ReportDefinition>> = OnceLock::new();

pub fn available_report_definitions() -> &'static [ReportDefinition] {
    REPORT_DEFINITIONS
        .get_or_init(|| {
            let ReportsFile { report_jobs } = serde_yaml::from_str(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../dagster/generate_es_reports/reports.yml"
            )))
            .expect("invalid report definitions YAML");

            report_jobs
                .into_iter()
                .map(|report| ReportDefinition {
                    norm: report.norm,
                    id: report.id,
                    friendly_name: report.friendly_name,
                    source_table: report.source_table,
                    outputs: report.outputs,
                    supports_as_of: report.supports_as_of,
                })
                .collect()
        })
        .as_slice()
}

pub fn find_report_definition(
    report_definition_id: &ReportDefinitionId,
) -> Option<&'static ReportDefinition> {
    available_report_definitions()
        .iter()
        .find(|definition| definition.report_definition_id() == *report_definition_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_report_definitions_from_yaml() {
        let definitions = available_report_definitions();
        assert!(!definitions.is_empty());
    }

    #[test]
    fn finds_as_of_report_and_builds_asset_selection_paths() {
        let id = ReportDefinitionId::try_new("nrp_51/01_saldo_cuenta")
            .expect("valid id format");
        let definition = find_report_definition(&id)
            .expect("nrp_51/01_saldo_cuenta must exist");

        assert!(definition.supports_as_of);
        assert_eq!(
            definition.asset_selection_paths(),
            vec![
                vec![
                    "file_report".to_string(),
                    "report_nrp_51_01_saldo_cuenta_daily_xml".to_string(),
                ],
                vec![
                    "file_report".to_string(),
                    "report_nrp_51_01_saldo_cuenta_daily_csv".to_string(),
                ],
            ]
        );
    }
}
