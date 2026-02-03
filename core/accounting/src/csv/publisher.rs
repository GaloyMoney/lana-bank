use obix::out::{Outbox, OutboxEventMarker};

use crate::{
    event::{CSV_EXPORT_EVENT_TYPE, CoreAccountingEvent},
    primitives::{AccountingCsvId, LedgerAccountId},
};

use super::error::AccountingCsvExportError;

pub struct AccountingCsvPublisher<E>
where
    E: OutboxEventMarker<CoreAccountingEvent>,
{
    outbox: Outbox<E>,
}

impl<E> Clone for AccountingCsvPublisher<E>
where
    E: OutboxEventMarker<CoreAccountingEvent>,
{
    fn clone(&self) -> Self {
        Self {
            outbox: self.outbox.clone(),
        }
    }
}

impl<E> AccountingCsvPublisher<E>
where
    E: OutboxEventMarker<CoreAccountingEvent>,
{
    pub fn new(outbox: &Outbox<E>) -> Self {
        Self {
            outbox: outbox.clone(),
        }
    }

    pub async fn publish_csv_export_uploaded(
        &self,
        op: &mut es_entity::DbOp<'_>,
        id: AccountingCsvId,
        ledger_account_id: LedgerAccountId,
    ) -> Result<(), AccountingCsvExportError> {
        self.outbox
            .publish_ephemeral_in_op(
                op,
                CSV_EXPORT_EVENT_TYPE,
                CoreAccountingEvent::LedgerAccountCsvExportUploaded {
                    id,
                    ledger_account_id,
                },
            )
            .await?;
        Ok(())
    }
}
