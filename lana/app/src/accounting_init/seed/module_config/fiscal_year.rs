use crate::{
    accounting::Chart,
    accounting_init::AccountingInitError,
    fiscal_year::{FiscalYears, error::FiscalYearError},
};

pub(in crate::accounting_init::seed) async fn fiscal_year_module_configure(
    fiscal_year: &FiscalYears,
    chart: &Chart,
) -> Result<(), AccountingInitError> {
    match fiscal_year.configure(chart.id).await {
        Ok(_) => (),
        Err(FiscalYearError::FiscalYearConfigAlreadyExists) => (),
        Err(e) => return Err(e.into()),
    };

    Ok(())
}
