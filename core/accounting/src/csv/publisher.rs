use obix::out::{Outbox, OutboxEventMarker};
use tracing::instrument;
use tracing_macros::record_error_severity;

use crate::{
    event::CoreAccountingEvent,
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
            .publish_all_persisted(
                op,
                vec![CoreAccountingEvent::LedgerAccountCsvExportUploaded {
                    id,
                    ledger_account_id,
                }],
            )
            .await?;
        Ok(())
    }
}
