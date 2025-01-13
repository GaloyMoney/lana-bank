use chart_of_accounts::{ChartId, ControlSubAccountPath};

#[derive(Clone, Copy)]
pub struct ChartIds {
    pub primary: ChartId,
    pub off_balance_sheet: ChartId,
}

#[derive(Clone)]
pub struct DepositsAccountPaths {
    pub deposits: ControlSubAccountPath,
}

#[derive(Clone)]
pub struct CreditFacilitiesAccountPaths {
    pub collateral: ControlSubAccountPath,
    pub facility: ControlSubAccountPath,
    pub disbursed_receivable: ControlSubAccountPath,
    pub interest_receivable: ControlSubAccountPath,
    pub interest_income: ControlSubAccountPath,
    pub fee_income: ControlSubAccountPath,
}
