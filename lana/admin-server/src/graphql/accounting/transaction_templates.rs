use async_graphql::*;

use es_entity::graphql::UUID;

use lana_app::accounting::transaction_templates::TransactionTemplate as DomainTransactionTemplate;

#[derive(Clone, SimpleObject)]
pub struct TransactionTemplate {
    id: UUID,
    code: String,
}

impl From<DomainTransactionTemplate> for TransactionTemplate {
    fn from(value: DomainTransactionTemplate) -> Self {
        todo!()
    }
}
