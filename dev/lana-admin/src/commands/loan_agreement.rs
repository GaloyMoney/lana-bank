use anyhow::Result;
use serde_json::json;

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
                        output::print_json(&json!({
                            "loanAgreementId": la.loan_agreement_id,
                            "status": format!("{:?}", la.status),
                            "createdAt": la.created_at,
                        }))?;
                    } else {
                        output::print_kv(&[
                            ("Loan Agreement ID", &la.loan_agreement_id),
                            ("Status", &format!("{:?}", la.status)),
                            ("Created At", &la.created_at),
                        ]);
                    }
                }
                None => output::not_found("Loan agreement", json),
            }
        }
        LoanAgreementAction::Generate { customer_id } => {
            let requested_customer_id = customer_id.clone();
            let vars = credit_facility_agreement_generate::Variables {
                input: credit_facility_agreement_generate::CreditFacilityAgreementGenerateInput {
                    customer_id,
                },
            };
            let data = client.execute::<CreditFacilityAgreementGenerate>(vars).await?;
            let la = data.credit_facility_agreement_generate.loan_agreement;
            if json {
                output::print_json(&json!({
                    "loanAgreementId": la.loan_agreement_id,
                    "customerId": requested_customer_id,
                    "status": format!("{:?}", la.status),
                    "createdAt": la.created_at,
                }))?;
            } else {
                output::print_kv(&[
                    ("Loan Agreement ID", &la.loan_agreement_id),
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
                output::print_json(&json!({
                    "loanAgreementId": result.loan_agreement_id,
                    "link": result.link,
                }))?;
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
