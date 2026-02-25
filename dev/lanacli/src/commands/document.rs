use anyhow::Result;

use crate::cli::DocumentAction;
use crate::client::GraphQLClient;
use crate::graphql::*;
use crate::output;

pub async fn execute(client: &mut GraphQLClient, action: DocumentAction, json: bool) -> Result<()> {
    match action {
        DocumentAction::Get { id } => {
            let vars = customer_document_get::Variables { id };
            let data = client.execute::<CustomerDocumentGet>(vars).await?;
            match data.customer_document {
                Some(doc) => {
                    if json {
                        output::print_json(&doc)?;
                    } else {
                        output::print_kv(&[
                            ("Document ID", &doc.document_id),
                            ("Customer ID", &doc.customer_id),
                            ("Filename", &doc.filename),
                            ("Status", &format!("{:?}", doc.status)),
                        ]);
                    }
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
        DocumentAction::List { customer_id } => {
            let vars = customer_documents_list::Variables { customer_id };
            let data = client.execute::<CustomerDocumentsList>(vars).await?;
            match data.customer {
                Some(c) => {
                    let docs = c.documents;
                    if json {
                        output::print_json(&docs)?;
                    } else {
                        let rows: Vec<Vec<String>> = docs
                            .iter()
                            .map(|d| {
                                vec![
                                    d.document_id.clone(),
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
                None => {
                    if json {
                        println!("null");
                    } else {
                        println!("Customer not found");
                    }
                }
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
                output::print_json(&doc)?;
            } else {
                output::print_kv(&[
                    ("ID", &doc.id),
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
