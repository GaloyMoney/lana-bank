use super::*;

const LANA_JOURNAL_CODE: &str = "LANA_BANK_JOURNAL";

const CHART_REF: &str = "primary-chart";

const DEPOSITS_CONTROL_ACCOUNT_REF: &str = "deposits";
const DEPOSITS_CONTROL_ACCOUNT_NAME: &str = "Deposits";

const DEPOSITS_CONTROL_SUB_ACCOUNT_REF: &str = "deposits-user";
const DEPOSITS_CONTROL_SUB_ACCOUNT_NAME: &str = "User Deposits";

pub(super) async fn execute(
    cala: &CalaLedger,
    chart_of_accounts: &ChartOfAccounts,
) -> Result<AccountingInit, AccountingInitError> {
    let journal_id = create_journal(cala).await?;

    let chart_id = create_chart_of_accounts(chart_of_accounts).await?;
    let deposits_control_sub_path = create_control_sub_account(
        chart_of_accounts,
        chart_id,
        ChartOfAccountCode::Category(chart_of_accounts::CategoryPath::Liabilities),
        DEPOSITS_CONTROL_ACCOUNT_NAME.to_string(),
        DEPOSITS_CONTROL_ACCOUNT_REF.to_string(),
        DEPOSITS_CONTROL_SUB_ACCOUNT_NAME.to_string(),
        DEPOSITS_CONTROL_SUB_ACCOUNT_REF.to_string(),
    )
    .await?;

    Ok(AccountingInit {
        journal_id,
        chart_id,
        deposits_control_sub_path,
    })
}

async fn create_journal(cala: &CalaLedger) -> Result<JournalId, AccountingInitError> {
    use cala_ledger::journal::*;

    let new_journal = NewJournal::builder()
        .id(JournalId::new())
        .name("General Ledger")
        .description("General ledger for Lana")
        .code(LANA_JOURNAL_CODE)
        .build()
        .expect("new journal");

    match cala.journals().create(new_journal).await {
        Err(cala_ledger::journal::error::JournalError::CodeAlreadyExists) => {
            let journal = cala
                .journals()
                .find_by_code(LANA_JOURNAL_CODE.to_string())
                .await?;
            Ok(journal.id)
        }
        Err(e) => Err(e.into()),
        Ok(journal) => Ok(journal.id),
    }
}

async fn create_chart_of_accounts(
    chart_of_accounts: &ChartOfAccounts,
) -> Result<ChartId, AccountingInitError> {
    let chart = match chart_of_accounts
        .find_by_reference(CHART_REF.to_string())
        .await?
    {
        Some(chart) => chart,
        None => {
            chart_of_accounts
                .create_chart(ChartId::new(), CHART_REF.to_string())
                .await?
        }
    };

    Ok(chart.id)
}

async fn create_control_sub_account(
    chart_of_accounts: &ChartOfAccounts,
    chart_id: ChartId,
    category: ChartOfAccountCode,
    control_name: String,
    control_reference: String,
    sub_name: String,
    sub_reference: String,
) -> Result<ChartOfAccountCode, AccountingInitError> {
    let control_path = match chart_of_accounts
        .find_control_account_by_reference(chart_id, control_reference.clone())
        .await?
    {
        Some(path) => path,
        None => {
            chart_of_accounts
                .create_control_account(chart_id, category, control_name, control_reference)
                .await?
        }
    };

    let control_sub_path = match chart_of_accounts
        .find_control_sub_account_by_reference(chart_id, sub_reference.clone())
        .await?
    {
        Some(path) => path,
        None => {
            chart_of_accounts
                .create_control_sub_account(chart_id, control_path, sub_name, sub_reference)
                .await?
        }
    };

    Ok(control_sub_path)
}
