use crate::{accounting::Chart, accounting_init::AccountingInitError, fiscal_year::FiscalYears};

pub(in crate::accounting_init::seed) async fn fiscal_year_module_configure(
    fiscal_year: &FiscalYears,
    chart: &Chart,
) -> Result<(), AccountingInitError> {
    fiscal_year.configure(chart.id).await?;
    Ok(())
}
