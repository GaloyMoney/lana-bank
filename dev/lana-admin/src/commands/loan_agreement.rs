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
            match data.credit_facility_agreement {
                Some(agreement) => {
                    if json {
                        output::print_json(&json!({
                            "creditFacilityAgreementId": agreement.credit_facility_agreement_id,
                            "status": format!("{:?}", agreement.status),
                            "createdAt": agreement.created_at,
                        }))?;
                    } else {
                        output::print_kv(&[
                            (
                                "Credit Facility Agreement ID",
                                &agreement.credit_facility_agreement_id,
                            ),
                            ("Status", &format!("{:?}", agreement.status)),
                            ("Created At", &agreement.created_at),
                        ]);
                    }
                }
                None => output::not_found("Credit facility agreement", json),
            }
        }
        LoanAgreementAction::Generate { credit_facility_id } => {
            let requested_credit_facility_id = credit_facility_id.clone();
            let vars = credit_facility_agreement_generate::Variables {
                input: credit_facility_agreement_generate::CreditFacilityAgreementGenerateInput {
                    credit_facility_id,
                },
            };
            let data = client
                .execute::<CreditFacilityAgreementGenerate>(vars)
                .await?;
            let agreement = data
                .credit_facility_agreement_generate
                .credit_facility_agreement;
            if json {
                output::print_json(&json!({
                    "creditFacilityAgreementId": agreement.credit_facility_agreement_id,
                    "creditFacilityId": requested_credit_facility_id,
                    "status": format!("{:?}", agreement.status),
                    "createdAt": agreement.created_at,
                }))?;
            } else {
                output::print_kv(&[
                    (
                        "Credit Facility Agreement ID",
                        &agreement.credit_facility_agreement_id,
                    ),
                    ("Status", &format!("{:?}", agreement.status)),
                    ("Created At", &agreement.created_at),
                ]);
            }
        }
        LoanAgreementAction::DownloadLink {
            credit_facility_agreement_id,
        } => {
            let vars = loan_agreement_download_link_generate::Variables {
                input:
                    loan_agreement_download_link_generate::CreditFacilityAgreementDownloadLinksGenerateInput {
                        credit_facility_agreement_id,
                    },
            };
            let data = client
                .execute::<LoanAgreementDownloadLinkGenerate>(vars)
                .await?;
            let result = data.credit_facility_agreement_download_link_generate;
            if json {
                output::print_json(&json!({
                    "creditFacilityAgreementId": result.credit_facility_agreement_id,
                    "link": result.link,
                }))?;
            } else {
                output::print_kv(&[
                    (
                        "Credit Facility Agreement ID",
                        &result.credit_facility_agreement_id,
                    ),
                    ("Link", &result.link),
                ]);
            }
        }
    }
    Ok(())
}
