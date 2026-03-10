use anyhow::Result;
use serde_json::Value;

use crate::cli::CsvExportAction;
use crate::client::GraphQLClient;
use crate::graphql::*;
use crate::output;

pub async fn execute(
    client: &mut GraphQLClient,
    action: CsvExportAction,
    json: bool,
) -> Result<()> {
    match action {
        CsvExportAction::AccountEntry { ledger_account_id } => {
            let vars = account_entry_csv::Variables { ledger_account_id };
            let data = client.execute::<AccountEntryCsv>(vars).await?;
            match data.account_entry_csv {
                Some(doc) => {
                    if json {
                        let mut value = serde_json::to_value(&doc)?;
                        if let Value::Object(ref mut obj) = value
                            && let Some(id) = obj.get("accountingCsvDocumentId").cloned()
                        {
                            // Keep backward compatibility for older automation/test scripts.
                            obj.insert("documentId".to_string(), id);
                        }
                        output::print_json(&value)?;
                    } else {
                        output::print_kv(&[
                            ("Document ID", &doc.accounting_csv_document_id),
                            ("Ledger Account ID", &doc.ledger_account_id),
                            ("Status", &format!("{:?}", doc.status)),
                            ("Created At", &doc.created_at),
                            ("Filename", &doc.filename),
                        ]);
                    }
                }
                None => output::not_found("Account entry CSV", json),
            }
        }
        CsvExportAction::CreateLedgerCsv { ledger_account_id } => {
            let vars = ledger_account_csv_create::Variables {
                input: ledger_account_csv_create::LedgerAccountCsvCreateInput { ledger_account_id },
            };
            let data = client.execute::<LedgerAccountCsvCreate>(vars).await?;
            let doc = data.ledger_account_csv_create.accounting_csv_document;
            if json {
                let mut value = serde_json::to_value(&doc)?;
                if let Value::Object(ref mut obj) = value
                    && let Some(id) = obj.get("accountingCsvDocumentId").cloned()
                {
                    // Keep backward compatibility for older automation/test scripts.
                    obj.insert("documentId".to_string(), id);
                }
                output::print_json(&value)?;
            } else {
                output::print_kv(&[
                    ("Document ID", &doc.accounting_csv_document_id),
                    ("Ledger Account ID", &doc.ledger_account_id),
                    ("Status", &format!("{:?}", doc.status)),
                    ("Created At", &doc.created_at),
                    ("Filename", &doc.filename),
                ]);
            }
        }
        CsvExportAction::DownloadLink { document_id } => {
            let vars = accounting_csv_download_link_generate::Variables {
                input:
                    accounting_csv_download_link_generate::AccountingCsvDownloadLinkGenerateInput {
                        document_id,
                    },
            };
            let data = client
                .execute::<AccountingCsvDownloadLinkGenerate>(vars)
                .await?;
            let link = data.accounting_csv_download_link_generate.link;
            if json {
                output::print_json(&link)?;
            } else {
                output::print_kv(&[("URL", &link.url), ("CSV ID", &link.csv_id)]);
            }
        }
    }
    Ok(())
}
