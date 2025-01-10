use chart_of_accounts::{ChartId, ChartPath};

#[derive(Clone, Copy)]
pub struct ChartIds {
    pub primary: ChartId,
    pub off_balance_sheet: ChartId,
}

#[derive(Clone)]
pub struct DepositsAccountPaths {
    pub deposits: ChartPath,
}

#[derive(Clone)]
pub struct CreditFacilitiesAccountPaths {
    pub collateral: ChartPath,
    pub facility: ChartPath,
    pub disbursed_receivable: ChartPath,
    pub interest_receivable: ChartPath,
    pub interest_income: ChartPath,
    pub fee_income: ChartPath,
}
