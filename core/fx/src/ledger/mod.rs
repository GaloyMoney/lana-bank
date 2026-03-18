use audit::SystemSubject;
use tracing::instrument;
use tracing_macros::record_error_severity;

use es_entity::clock::ClockHandle;

pub mod error;
mod templates;

use cala_ledger::{CalaLedger, Currency, JournalId, TransactionId};

use crate::primitives::CalaAccountId;
use error::*;

#[derive(Clone)]
pub struct FxLedger {
    cala: CalaLedger,
    clock: ClockHandle,
    journal_id: JournalId,
}

impl FxLedger {
    #[record_error_severity]
    #[instrument(name = "fx_ledger.init", skip_all)]
    pub async fn init(
        cala: &CalaLedger,
        journal_id: JournalId,
        clock: ClockHandle,
    ) -> Result<Self, FxLedgerError> {
        templates::FiatFxConversion::init(cala).await?;

        Ok(Self {
            cala: cala.clone(),
            clock,
            journal_id,
        })
    }

    #[record_error_severity]
    #[instrument(
        name = "fx_ledger.convert_fiat_fx_in_op",
        skip_all,
        fields(
            source_account_id = tracing::field::Empty,
            target_account_id = tracing::field::Empty,
            trading_account_id = tracing::field::Empty,
        )
    )]
    pub async fn convert_fiat_fx_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        source_currency: Currency,
        target_currency: Currency,
        source_amount: rust_decimal::Decimal,
        target_amount: rust_decimal::Decimal,
        source_account_id: CalaAccountId,
        target_account_id: CalaAccountId,
        trading_account_id: CalaAccountId,
        initiated_by: &impl SystemSubject,
    ) -> Result<(), FxLedgerError> {
        if source_currency == Currency::BTC || target_currency == Currency::BTC {
            return Err(FxLedgerError::BtcNotAllowed);
        }

        tracing::Span::current().record(
            "source_account_id",
            tracing::field::debug(&source_account_id),
        );
        tracing::Span::current().record(
            "target_account_id",
            tracing::field::debug(&target_account_id),
        );
        tracing::Span::current().record(
            "trading_account_id",
            tracing::field::debug(&trading_account_id),
        );

        let tx_id = TransactionId::new();
        let params = templates::FiatFxConversionParams {
            entity_id: tx_id.into(),
            journal_id: self.journal_id,
            source_currency,
            target_currency,
            source_amount,
            target_amount,
            source_account_id,
            target_account_id,
            trading_account_id,
            initiated_by,
            effective_date: self.clock.today(),
        };

        self.cala
            .post_transaction_in_op(
                op,
                tx_id,
                templates::FIAT_FX_CONVERSION_VIA_TRADING_CODE,
                params,
            )
            .await?;

        Ok(())
    }
}
