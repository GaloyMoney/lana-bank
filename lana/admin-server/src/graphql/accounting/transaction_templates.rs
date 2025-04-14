use async_graphql::*;

use crate::primitives::*;

use es_entity::graphql::UUID;

use lana_app::accounting::transaction_templates::TransactionTemplate as DomainTransactionTemplate;
pub use lana_app::accounting::transaction_templates::TransactionTemplateCursor;

#[derive(Clone, SimpleObject)]
pub struct TransactionTemplate {
    id: UUID,
    code: String,

    #[graphql(skip)]
    pub entity: Arc<DomainTransactionTemplate>,
}

impl From<DomainTransactionTemplate> for TransactionTemplate {
    fn from(template: DomainTransactionTemplate) -> Self {
        Self {
            id: template.id.into(),
            code: template.code.clone(),
            entity: Arc::new(template),
        }
    }
}
