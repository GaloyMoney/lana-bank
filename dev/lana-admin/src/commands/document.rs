use anyhow::Result;
use serde_json::Value;

use crate::cli::DocumentAction;
use crate::client::{GraphQLClient, MultipartUpload};
use crate::graphql::*;
use crate::output;

pub async fn execute(client: &mut GraphQLClient, action: DocumentAction, json: bool) -> Result<()> {
    match action {
        DocumentAction::Attach { customer_id, file } => {
            let vars = customer_document_create::Variables {
                file: file.clone(),
                customer_id,
            };
            let data = client
                .execute_multipart::<CustomerDocumentCreate>(
                    vars,
                    vec![MultipartUpload::new(file, "file")],
                )
                .await?;
            let doc = data.customer_document_create.document;
            if json {
                let mut value = serde_json::to_value(&doc)?;
                add_document_id_alias(&mut value);
                output::print_json(&value)?;
            } else {
                output::print_kv(&[
                    ("Document ID", &doc.customer_document_id),
                    ("Customer ID", &doc.customer_id),
                    ("Filename", &doc.filename),
                ]);
            }
        }
        DocumentAction::Get { id } => {
            let vars = customer_document_get::Variables { id };
            let data = client.execute::<CustomerDocumentGet>(vars).await?;
            match data.customer_document {
                Some(doc) => {
                    if json {
                        let mut value = serde_json::to_value(&doc)?;
                        add_document_id_alias(&mut value);
                        output::print_json(&value)?;
                    } else {
                        output::print_kv(&[
                            ("Document ID", &doc.customer_document_id),
                            ("Customer ID", &doc.customer_id),
                            ("Filename", &doc.filename),
                            ("Status", &format!("{:?}", doc.status)),
                        ]);
                    }
                }
                None => output::not_found("Document", json),
            }
        }
        DocumentAction::List { customer_id } => {
            let vars = customer_documents_list::Variables { customer_id };
            let data = client.execute::<CustomerDocumentsList>(vars).await?;
            match data.customer {
                Some(c) => {
                    let docs = c.documents;
                    if json {
                        let mut value = serde_json::to_value(&docs)?;
                        add_document_id_alias(&mut value);
                        output::print_json(&value)?;
                    } else {
                        let rows: Vec<Vec<String>> = docs
                            .iter()
                            .map(|d| {
                                vec![
                                    d.customer_document_id.clone(),
                                    d.customer_id.clone(),
                                    d.filename.clone(),
                                    format!("{:?}", d.status),
                                ]
                            })
                            .collect();
                        output::print_table(
                            &["Document ID", "Customer ID", "Filename", "Status"],
                            rows,
                        );
                    }
                }
                None => output::not_found("Customer", json),
            }
        }
        DocumentAction::DownloadLink { document_id } => {
            let vars = customer_document_download_link_generate::Variables {
                input:
                    customer_document_download_link_generate::CustomerDocumentDownloadLinksGenerateInput {
                        document_id,
                    },
            };
            let data = client
                .execute::<CustomerDocumentDownloadLinkGenerate>(vars)
                .await?;
            let result = data.customer_document_download_link_generate;
            if json {
                output::print_json(&result)?;
            } else {
                output::print_kv(&[("Document ID", &result.document_id), ("Link", &result.link)]);
            }
        }
        DocumentAction::Archive { document_id } => {
            let vars = customer_document_archive::Variables {
                input: customer_document_archive::CustomerDocumentArchiveInput { document_id },
            };
            let data = client.execute::<CustomerDocumentArchive>(vars).await?;
            let doc = data.customer_document_archive.document;
            if json {
                let mut value = serde_json::to_value(&doc)?;
                add_document_id_alias(&mut value);
                output::print_json(&value)?;
            } else {
                output::print_kv(&[
                    ("Document ID", &doc.customer_document_id),
                    ("Customer ID", &doc.customer_id),
                    ("Filename", &doc.filename),
                    ("Status", &format!("{:?}", doc.status)),
                ]);
            }
        }
        DocumentAction::Delete { document_id } => {
            let vars = customer_document_delete::Variables {
                input: customer_document_delete::CustomerDocumentDeleteInput { document_id },
            };
            let data = client.execute::<CustomerDocumentDelete>(vars).await?;
            let result = data.customer_document_delete;
            if json {
                output::print_json(&result)?;
            } else {
                output::print_kv(&[("Deleted Document ID", &result.deleted_document_id)]);
            }
        }
    }
    Ok(())
}

fn add_document_id_alias(value: &mut Value) {
    match value {
        Value::Object(obj) => {
            if let Some(id) = obj.get("customerDocumentId").cloned() {
                // Keep backward compatibility for older automation/test scripts.
                obj.insert("documentId".to_string(), id);
            }
            for nested in obj.values_mut() {
                add_document_id_alias(nested);
            }
        }
        Value::Array(items) => {
            for item in items {
                add_document_id_alias(item);
            }
        }
        _ => {}
    }
}
