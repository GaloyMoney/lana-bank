use audit::SystemSubject;
use rust_decimal::Decimal;
use tracing::instrument;
use tracing_macros::record_error_severity;

use es_entity::clock::ClockHandle;

pub mod error;
pub(crate) mod templates;

use cala_ledger::{CalaLedger, Currency, JournalId, TransactionId};

use crate::position::FxPositions;
use crate::primitives::{CalaAccountId, ExchangeRate, FxConversionResult};
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
        templates::RealizedFxGainLoss::init(cala).await?;
        templates::FxRoundingAdjustment::init(cala).await?;

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

    #[record_error_severity]
    #[instrument(name = "fx_ledger.convert_fiat_fx_with_rate_in_op", skip_all)]
    #[allow(clippy::too_many_arguments)]
    pub async fn convert_fiat_fx_with_rate_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        source_currency: Currency,
        target_currency: Currency,
        source_amount: Decimal,
        rate: ExchangeRate,
        source_account_id: CalaAccountId,
        target_account_id: CalaAccountId,
        trading_account_id: CalaAccountId,
        gain_account_id: CalaAccountId,
        loss_account_id: CalaAccountId,
        rounding_account_id: CalaAccountId,
        functional_currency: Currency,
        positions: &FxPositions,
        initiated_by: &impl SystemSubject,
    ) -> Result<FxConversionResult, FxLedgerError> {
        // 1. Compute target amount and rounding difference
        let (target_amount, rounding_difference) = rate.convert(source_amount);

        // 2. Post 4-entry conversion via existing template
        self.convert_fiat_fx_in_op(
            &mut *op,
            source_currency,
            target_currency,
            source_amount,
            target_amount,
            source_account_id,
            target_account_id,
            trading_account_id,
            initiated_by,
        )
        .await?;

        // 3. Update positions and compute realized G/L
        let mut realized_gain_loss = Decimal::ZERO;

        // When source != functional: trading receives source currency, position increases
        // The functional cost is the target amount (what was given in exchange)
        if source_currency != functional_currency {
            let mut position = positions
                .find_or_create_in_op(&mut *op, source_currency.code())
                .await?;
            // Trading account receives source_amount of source_currency
            // Cost in functional currency = target_amount (what we gave away)
            position.increase_position(source_amount, target_amount)?;
            positions.update_in_op(&mut *op, &mut position).await?;
        }

        // When target != functional: trading gives target currency, position decreases
        // The functional proceeds is the source amount (what was received in exchange)
        if target_currency != functional_currency {
            let mut position = positions
                .find_or_create_in_op(&mut *op, target_currency.code())
                .await?;
            // Trading account gives target_amount of target_currency
            // Proceeds in functional currency = source_amount (what we received)
            realized_gain_loss = position.decrease_position(target_amount, source_amount)?;
            positions.update_in_op(&mut *op, &mut position).await?;
        }

        // 4. Post realized G/L if non-zero
        if realized_gain_loss != Decimal::ZERO {
            let abs_amount = realized_gain_loss.abs();
            let (trading_direction, gain_loss_direction, gl_account_id) =
                if realized_gain_loss > Decimal::ZERO {
                    // Gain: Dr Trading / Cr Gain account
                    ("DEBIT", "CREDIT", gain_account_id)
                } else {
                    // Loss: Dr Loss account / Cr Trading
                    ("CREDIT", "DEBIT", loss_account_id)
                };

            let gl_tx_id = TransactionId::new();
            let gl_params = templates::RealizedFxGainLossParams {
                entity_id: gl_tx_id.into(),
                journal_id: self.journal_id,
                currency: functional_currency,
                amount: abs_amount,
                trading_account_id,
                gain_or_loss_account_id: gl_account_id,
                trading_direction,
                gain_loss_direction,
                initiated_by,
                effective_date: self.clock.today(),
            };
            self.cala
                .post_transaction_in_op(
                    &mut *op,
                    gl_tx_id,
                    templates::REALIZED_FX_GAIN_LOSS_CODE,
                    gl_params,
                )
                .await?;
        }

        // 5. Post rounding adjustment if non-zero
        if rounding_difference > Decimal::ZERO {
            let rounding_tx_id = TransactionId::new();
            let rounding_params = templates::FxRoundingAdjustmentParams {
                entity_id: rounding_tx_id.into(),
                journal_id: self.journal_id,
                currency: target_currency,
                amount: rounding_difference,
                trading_account_id,
                rounding_account_id,
                initiated_by,
                effective_date: self.clock.today(),
            };
            self.cala
                .post_transaction_in_op(
                    &mut *op,
                    rounding_tx_id,
                    templates::FX_ROUNDING_ADJUSTMENT_CODE,
                    rounding_params,
                )
                .await?;
        }

        Ok(FxConversionResult {
            target_amount,
            rounding_difference,
            realized_gain_loss,
        })
    }
}
