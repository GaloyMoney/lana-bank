use async_graphql::*;

use crate::primitives::*;

use lana_app::accounting::transaction_templates::TransactionTemplate as DomainTransactionTemplate;
pub use lana_app::accounting::transaction_templates::TransactionTemplateCursor;

#[derive(Clone, SimpleObject)]
pub struct TransactionTemplate {
    id: ID,
    transaction_template_id: UUID,
    code: String,

    #[graphql(skip)]
    pub entity: Arc<DomainTransactionTemplate>,
}

impl From<DomainTransactionTemplate> for TransactionTemplate {
    fn from(template: DomainTransactionTemplate) -> Self {
        Self {
            id: template.id.to_global_id(),
            transaction_template_id: template.id.into(),
            code: template.values().code.clone(),
            entity: Arc::new(template),
        }
    }
}
