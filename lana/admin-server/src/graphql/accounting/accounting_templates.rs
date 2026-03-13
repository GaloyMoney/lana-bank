use async_graphql::*;

use crate::primitives::*;

use lana_app::accounting::accounting_templates::AccountingTemplate as DomainAccountingTemplate;
use lana_app::primitives::DebitOrCredit;

#[derive(Clone, SimpleObject)]
pub struct AccountingTemplateEntry {
    pub account_id_or_code: String,
    pub direction: DebitOrCredit,
    pub description_template: Option<String>,
}

#[derive(Clone, SimpleObject)]
pub struct AccountingTemplate {
    pub id: UUID,
    pub code: String,
    pub name: String,
    pub chart_ref: Option<String>,
    pub description_template: String,
    pub entries: Vec<AccountingTemplateEntry>,
}

impl From<DomainAccountingTemplate> for AccountingTemplate {
    fn from(template: DomainAccountingTemplate) -> Self {
        let DomainAccountingTemplate {
            id,
            code,
            name,
            values,
            ..
        } = template;
        Self {
            id: id.into(),
            code,
            name,
            chart_ref: values.chart_ref,
            description_template: values.description_template,
            entries: values
                .entries
                .into_iter()
                .map(|e| AccountingTemplateEntry {
                    account_id_or_code: e.account_id_or_code,
                    direction: e.direction,
                    description_template: e.description_template,
                })
                .collect(),
        }
    }
}
