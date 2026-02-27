use anyhow::Result;

use crate::cli::LoanAgreementAction;
use crate::client::GraphQLClient;
use crate::graphql::*;
use crate::output;

pub async fn execute(
    client: &mut GraphQLClient,
    action: LoanAgreementAction,
    json: bool,
) -> Result<()> {
    match action {
        LoanAgreementAction::Find { id } => {
            let vars = find_loan_agreement::Variables { id };
            let data = client.execute::<FindLoanAgreement>(vars).await?;
            match data.loan_agreement {
                Some(la) => {
                    if json {
                        output::print_json(&la)?;
                    } else {
                        output::print_kv(&[
                            ("ID", &la.id),
                            ("Status", &format!("{:?}", la.status)),
                            ("Created At", &la.created_at),
                        ]);
                    }
                }
                None => {
                    if json {
                        println!("null");
                    } else {
                        println!("Loan agreement not found");
                    }
                }
            }
        }
        LoanAgreementAction::Generate { customer_id } => {
            let vars = loan_agreement_generate::Variables {
                input: loan_agreement_generate::LoanAgreementGenerateInput { customer_id },
            };
            let data = client.execute::<LoanAgreementGenerate>(vars).await?;
            let la = data.loan_agreement_generate.loan_agreement;
            if json {
                output::print_json(&la)?;
            } else {
                output::print_kv(&[
                    ("ID", &la.id),
                    ("Status", &format!("{:?}", la.status)),
                    ("Created At", &la.created_at),
                ]);
            }
        }
        LoanAgreementAction::DownloadLink { loan_agreement_id } => {
            let vars = loan_agreement_download_link_generate::Variables {
                input:
                    loan_agreement_download_link_generate::LoanAgreementDownloadLinksGenerateInput {
                        loan_agreement_id,
                    },
            };
            let data = client
                .execute::<LoanAgreementDownloadLinkGenerate>(vars)
                .await?;
            let result = data.loan_agreement_download_link_generate;
            if json {
                output::print_json(&result)?;
            } else {
                output::print_kv(&[
                    ("Loan Agreement ID", &result.loan_agreement_id),
                    ("Link", &result.link),
                ]);
            }
        }
    }
    Ok(())
}
