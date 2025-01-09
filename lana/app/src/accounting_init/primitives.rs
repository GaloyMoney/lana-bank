use chart_of_accounts::{ChartId, ChartOfAccountCode};

#[derive(Clone, Copy)]
pub struct ChartIds {
    pub primary: ChartId,
    pub off_balance_sheet: ChartId,
}

#[derive(Clone)]
pub struct DepositsAccountPaths {
    pub deposits: ChartOfAccountCode,
}

#[derive(Clone)]
pub struct CreditFacilitiesAccountPaths {
    pub collateral: ChartOfAccountCode,
    pub facility: ChartOfAccountCode,
    pub disbursed_receivable: ChartOfAccountCode,
    pub interest_receivable: ChartOfAccountCode,
    pub interest_income: ChartOfAccountCode,
    pub fee_income: ChartOfAccountCode,
}
