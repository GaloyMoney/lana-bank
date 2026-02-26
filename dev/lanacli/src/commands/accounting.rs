use anyhow::{Result, bail};

use crate::cli::AccountingAction;
use crate::client::{GraphQLClient, MultipartUpload};
use crate::graphql::*;
use crate::output;

pub async fn execute(
    client: &mut GraphQLClient,
    action: AccountingAction,
    json: bool,
) -> Result<()> {
    match action {
        AccountingAction::ChartOfAccounts => {
            let vars = chart_of_accounts_get::Variables {};
            let data = client.execute::<ChartOfAccountsGet>(vars).await?;
            output::print_json(&data.chart_of_accounts)?;
        }
        AccountingAction::AddRootNode {
            code,
            name,
            normal_balance_type,
        } => {
            let parsed_nbt = match normal_balance_type.to_uppercase().as_str() {
                "DEBIT" => chart_of_accounts_add_root_node::DebitOrCredit::DEBIT,
                "CREDIT" => chart_of_accounts_add_root_node::DebitOrCredit::CREDIT,
                other => bail!("Unknown normal_balance_type: {other}. Expected DEBIT or CREDIT"),
            };
            let vars = chart_of_accounts_add_root_node::Variables {
                input: chart_of_accounts_add_root_node::ChartOfAccountsAddRootNodeInput {
                    code,
                    name,
                    normal_balance_type: parsed_nbt,
                },
            };
            let data = client.execute::<ChartOfAccountsAddRootNode>(vars).await?;
            output::print_json(&data.chart_of_accounts_add_root_node.chart_of_accounts)?;
        }
        AccountingAction::AddChildNode { parent, code, name } => {
            let vars = chart_of_accounts_add_child_node::Variables {
                input: chart_of_accounts_add_child_node::ChartOfAccountsAddChildNodeInput {
                    parent,
                    code,
                    name,
                },
            };
            let data = client.execute::<ChartOfAccountsAddChildNode>(vars).await?;
            output::print_json(&data.chart_of_accounts_add_child_node.chart_of_accounts)?;
        }
        AccountingAction::CsvImport { file } => {
            let vars = chart_of_accounts_csv_import::Variables {
                input: chart_of_accounts_csv_import::ChartOfAccountsCsvImportInput {
                    file: file.clone(),
                },
            };
            let data = client
                .execute_multipart::<ChartOfAccountsCsvImport>(
                    vars,
                    vec![MultipartUpload::new(file, "input.file")],
                )
                .await?;
            output::print_json(&data.chart_of_accounts_csv_import.chart_of_accounts)?;
        }
        AccountingAction::BaseConfig => {
            let vars = accounting_base_config::Variables {};
            let data = client.execute::<AccountingBaseConfig>(vars).await?;
            match data.chart_of_accounts.accounting_base_config {
                Some(config) => {
                    output::print_json(&config)?;
                }
                None => {
                    if json {
                        println!("null");
                    } else {
                        println!("Not found");
                    }
                }
            }
        }
        AccountingAction::CreditConfig => {
            let vars = credit_config_get::Variables {};
            let data = client.execute::<CreditConfigGet>(vars).await?;
            match data.credit_config {
                Some(cc) => {
                    output::print_json(&cc)?;
                }
                None => {
                    if json {
                        println!("null");
                    } else {
                        println!("Not found");
                    }
                }
            }
        }
        AccountingAction::DepositConfig => {
            let vars = deposit_config_get::Variables {};
            let data = client.execute::<DepositConfigGet>(vars).await?;
            match data.deposit_config {
                Some(dc) => {
                    output::print_json(&dc)?;
                }
                None => {
                    if json {
                        println!("null");
                    } else {
                        println!("Not found");
                    }
                }
            }
        }
        AccountingAction::AccountSets { category } => {
            let parsed_category = match category.to_uppercase().as_str() {
                "ASSET" => descendant_account_sets_by_category::AccountCategory::ASSET,
                "LIABILITY" => descendant_account_sets_by_category::AccountCategory::LIABILITY,
                "EQUITY" => descendant_account_sets_by_category::AccountCategory::EQUITY,
                "REVENUE" => descendant_account_sets_by_category::AccountCategory::REVENUE,
                "COST_OF_REVENUE" => {
                    descendant_account_sets_by_category::AccountCategory::COST_OF_REVENUE
                }
                "EXPENSES" => descendant_account_sets_by_category::AccountCategory::EXPENSES,
                "OFF_BALANCE_SHEET" => {
                    descendant_account_sets_by_category::AccountCategory::OFF_BALANCE_SHEET
                }
                other => bail!(
                    "Unknown category: {other}. Expected one of: ASSET, LIABILITY, EQUITY, REVENUE, COST_OF_REVENUE, EXPENSES, OFF_BALANCE_SHEET"
                ),
            };
            let vars = descendant_account_sets_by_category::Variables {
                category: parsed_category,
            };
            let data = client
                .execute::<DescendantAccountSetsByCategory>(vars)
                .await?;
            let sets = data.descendant_account_sets_by_category;
            if json {
                output::print_json(&sets)?;
            } else {
                let rows: Vec<Vec<String>> = sets
                    .iter()
                    .map(|s| vec![s.account_set_id.clone(), s.code.clone(), s.name.clone()])
                    .collect();
                output::print_table(&["Account Set ID", "Code", "Name"], rows);
            }
        }
        AccountingAction::ManualTransaction {
            description,
            reference,
            effective,
            entries_json,
        } => {
            let entries_raw: Vec<serde_json::Value> = serde_json::from_str(&entries_json)?;
            let entries: Vec<manual_transaction_execute::ManualTransactionEntryInput> = entries_raw
                .iter()
                .map(|entry| {
                    let direction = match entry["direction"]
                        .as_str()
                        .unwrap_or("")
                        .to_uppercase()
                        .as_str()
                    {
                        "DEBIT" => manual_transaction_execute::DebitOrCredit::DEBIT,
                        "CREDIT" => manual_transaction_execute::DebitOrCredit::CREDIT,
                        other => panic!("Unknown direction: {other}. Expected DEBIT or CREDIT"),
                    };
                    manual_transaction_execute::ManualTransactionEntryInput {
                        account_ref: entry["accountRef"].as_str().unwrap().to_string(),
                        amount: entry["amount"].as_str().unwrap().to_string(),
                        currency: entry["currency"].as_str().unwrap().to_string(),
                        direction,
                        description: entry["description"].as_str().unwrap_or("").to_string(),
                    }
                })
                .collect();
            let vars = manual_transaction_execute::Variables {
                input: manual_transaction_execute::ManualTransactionExecuteInput {
                    description,
                    reference,
                    effective,
                    entries,
                },
            };
            let data = client.execute::<ManualTransactionExecute>(vars).await?;
            output::print_json(&data.manual_transaction_execute.transaction)?;
        }
        AccountingAction::LedgerAccount { code } => {
            let vars = ledger_account_by_code::Variables { code };
            let data = client.execute::<LedgerAccountByCode>(vars).await?;
            match data.ledger_account_by_code {
                Some(la) => {
                    output::print_json(&la)?;
                }
                None => {
                    if json {
                        println!("null");
                    } else {
                        println!("Not found");
                    }
                }
            }
        }
    }
    Ok(())
}
