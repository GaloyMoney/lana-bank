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
    let deposits_control_sub_path =
        create_deposits_control_sub_account(chart_of_accounts, chart_id).await?;

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

async fn create_deposits_control_sub_account(
    chart_of_accounts: &ChartOfAccounts,
    chart_id: ChartId,
) -> Result<ChartOfAccountCode, AccountingInitError> {
    let deposits_control_path = match chart_of_accounts
        .find_control_account_by_reference(chart_id, DEPOSITS_CONTROL_ACCOUNT_REF.to_string())
        .await?
    {
        Some(path) => path,
        None => {
            chart_of_accounts
                .create_control_account(
                    chart_id,
                    ChartOfAccountCode::Category(chart_of_accounts::CategoryPath::Liabilities),
                    DEPOSITS_CONTROL_ACCOUNT_NAME.to_string(),
                    DEPOSITS_CONTROL_ACCOUNT_REF.to_string(),
                )
                .await?
        }
    };

    let deposits_control_sub_path = match chart_of_accounts
        .find_control_sub_account_by_reference(
            chart_id,
            DEPOSITS_CONTROL_SUB_ACCOUNT_REF.to_string(),
        )
        .await?
    {
        Some(path) => path,
        None => {
            chart_of_accounts
                .create_control_sub_account(
                    chart_id,
                    deposits_control_path,
                    DEPOSITS_CONTROL_SUB_ACCOUNT_NAME.to_string(),
                    DEPOSITS_CONTROL_SUB_ACCOUNT_REF.to_string(),
                )
                .await?
        }
    };

    Ok(deposits_control_sub_path)
}
