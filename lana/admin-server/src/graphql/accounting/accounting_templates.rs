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

    #[graphql(skip)]
    pub entity: Arc<DomainAccountingTemplate>,
}

#[derive(InputObject)]
pub struct AccountingTemplateEntryInput {
    pub account_id_or_code: String,
    pub direction: DebitOrCredit,
    pub description_template: Option<String>,
}

#[derive(InputObject)]
pub struct AccountingTemplateCreateInput {
    pub code: String,
    pub name: String,
    pub chart_ref: Option<String>,
    pub description_template: String,
    pub entries: Vec<AccountingTemplateEntryInput>,
}

#[derive(InputObject)]
pub struct AccountingTemplateUpdateInput {
    pub id: UUID,
    pub name: Option<String>,
    pub chart_ref: Option<String>,
    pub description_template: Option<String>,
    pub entries: Option<Vec<AccountingTemplateEntryInput>>,
}

crate::mutation_payload! { AccountingTemplateCreatePayload, accounting_template: AccountingTemplate }
crate::mutation_payload! { AccountingTemplateUpdatePayload, accounting_template: AccountingTemplate }

impl From<DomainAccountingTemplate> for AccountingTemplate {
    fn from(domain: DomainAccountingTemplate) -> Self {
        let values = domain.values.clone();
        let code = domain.code.clone();
        let name = domain.name.clone();

        Self {
            id: domain.id.into(),
            code,
            name,
            chart_ref: values.chart_ref.clone(),
            description_template: values.description_template.clone(),
            entries: values
                .entries
                .into_iter()
                .map(|e| AccountingTemplateEntry {
                    account_id_or_code: e.account_id_or_code,
                    direction: e.direction,
                    description_template: e.description_template,
                })
                .collect(),
            entity: Arc::new(domain),
        }
    }
}

impl From<AccountingTemplateEntryInput>
    for lana_app::accounting::accounting_templates::AccountingTemplateEntry
{
    fn from(input: AccountingTemplateEntryInput) -> Self {
        Self {
            account_id_or_code: input.account_id_or_code,
            direction: input.direction,
            description_template: input.description_template,
        }
    }
}

impl From<AccountingTemplateCreateInput>
    for lana_app::accounting::accounting_templates::AccountingTemplateValues
{
    fn from(input: AccountingTemplateCreateInput) -> Self {
        Self {
            chart_ref: input.chart_ref,
            description_template: input.description_template,
            entries: input.entries.into_iter().map(Into::into).collect(),
        }
    }
}
