use async_graphql::*;

pub use lana_app::report::{
    ReportDefinition as DomainReportDefinition,
    ReportDefinitionOutput as DomainReportDefinitionOutput,
    ReportOutputFormat as DomainReportOutputFormat,
};

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum ReportOutputFormat {
    Csv,
    Txt,
    Xml,
}

impl From<DomainReportOutputFormat> for ReportOutputFormat {
    fn from(format: DomainReportOutputFormat) -> Self {
        match format {
            DomainReportOutputFormat::Csv => ReportOutputFormat::Csv,
            DomainReportOutputFormat::Txt => ReportOutputFormat::Txt,
            DomainReportOutputFormat::Xml => ReportOutputFormat::Xml,
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct ReportDefinitionOutput {
    format: ReportOutputFormat,
}

impl From<DomainReportDefinitionOutput> for ReportDefinitionOutput {
    fn from(output: DomainReportDefinitionOutput) -> Self {
        Self {
            format: ReportOutputFormat::from(output.format),
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct ReportDefinition {
    report_definition_id: String,
    norm: String,
    id: String,
    friendly_name: String,
    source_table: String,
    outputs: Vec<ReportDefinitionOutput>,
    supports_as_of: bool,
}

impl From<DomainReportDefinition> for ReportDefinition {
    fn from(report_definition: DomainReportDefinition) -> Self {
        Self {
            report_definition_id: report_definition.report_definition_id(),
            norm: report_definition.norm,
            id: report_definition.id,
            friendly_name: report_definition.friendly_name,
            source_table: report_definition.source_table,
            outputs: report_definition
                .outputs
                .into_iter()
                .map(ReportDefinitionOutput::from)
                .collect(),
            supports_as_of: report_definition.supports_as_of,
        }
    }
}
