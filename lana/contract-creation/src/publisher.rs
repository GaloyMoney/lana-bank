use document_storage::DocumentId;
use obix::out::{Outbox, OutboxEventMarker};

use crate::{ContractCreationError, ContractCreationEvent};

pub struct ContractCreationPublisher<E>
where
    E: OutboxEventMarker<ContractCreationEvent>,
{
    outbox: Outbox<E>,
}

impl<E> Clone for ContractCreationPublisher<E>
where
    E: OutboxEventMarker<ContractCreationEvent>,
{
    fn clone(&self) -> Self {
        Self {
            outbox: self.outbox.clone(),
        }
    }
}

impl<E> ContractCreationPublisher<E>
where
    E: OutboxEventMarker<ContractCreationEvent>,
{
    pub fn new(outbox: &Outbox<E>) -> Self {
        Self {
            outbox: outbox.clone(),
        }
    }

    pub async fn publish_loan_agreement_generated(
        &self,
        op: &mut es_entity::DbOp<'_>,
        loan_agreement_id: DocumentId,
    ) -> Result<(), ContractCreationError> {
        self.outbox
            .publish_all_persisted(
                op,
                vec![ContractCreationEvent::LoanAgreementGenerated { loan_agreement_id }],
            )
            .await?;
        Ok(())
    }
}
