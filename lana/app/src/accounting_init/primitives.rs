pub use crate::primitives::{LedgerAccountId, LedgerJournalId};

use chart_of_accounts::ChartId;

use crate::{
    credit_facility::{CreditFacilityAccountFactories, CreditFacilityOmnibusAccountIds},
    deposit::{DepositAccountFactories, DepositOmnibusAccountIds},
};

#[derive(Clone, Copy)]
pub struct ChartIds {
    pub primary: ChartId,
    pub off_balance_sheet: ChartId,
}

#[derive(Clone)]
pub struct DepositsSeed {
    pub factories: DepositAccountFactories,
    pub omnibus_ids: DepositOmnibusAccountIds,
}

#[derive(Clone)]
pub struct CreditFacilitiesSeed {
    pub factories: CreditFacilityAccountFactories,
    pub omnibus_ids: CreditFacilityOmnibusAccountIds,
}
