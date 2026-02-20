use audit::SystemSubject;

use crate::primitives::{CollateralId, CollateralUpdate, LedgerTxId, Satoshis};

use super::{CollateralLedgerAccountIds, CollateralLedgerError};

use crate::collateral::RecordProceedsFromLiquidationData;

pub trait CollateralLedgerOps: Clone + Send + Sync + 'static {
    fn create_collateral_accounts_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        collateral_id: CollateralId,
        account_ids: CollateralLedgerAccountIds,
    ) -> impl std::future::Future<Output = Result<(), CollateralLedgerError>> + Send;

    fn update_collateral_amount_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        update: CollateralUpdate,
        initiated_by: &(impl SystemSubject + Send + Sync),
    ) -> impl std::future::Future<Output = Result<(), CollateralLedgerError>> + Send;

    fn record_collateral_sent_to_liquidation_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        tx_id: LedgerTxId,
        amount: Satoshis,
        account_ids: CollateralLedgerAccountIds,
        initiated_by: &(impl SystemSubject + Send + Sync),
    ) -> impl std::future::Future<Output = Result<(), CollateralLedgerError>> + Send;

    fn record_proceeds_from_liquidation_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        data: RecordProceedsFromLiquidationData,
        initiated_by: &(impl SystemSubject + Send + Sync),
    ) -> impl std::future::Future<Output = Result<(), CollateralLedgerError>> + Send;
}
