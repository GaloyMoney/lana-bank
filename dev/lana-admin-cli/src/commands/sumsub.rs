use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::cli::SumsubAction;
use crate::client::GraphQLClient;
use crate::graphql::*;
use crate::output;

const SUMSUB_TEST_APPLICANT_CREATE_MUTATION: &str = r#"
mutation sumsubTestApplicantCreate($input: SumsubTestApplicantCreateInput!) {
  sumsubTestApplicantCreate(input: $input) {
    applicantId
  }
}
"#;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SumsubTestApplicantCreateResponse {
    sumsub_test_applicant_create: SumsubTestApplicantCreatePayload,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct SumsubTestApplicantCreatePayload {
    applicant_id: String,
}

pub async fn execute(client: &mut GraphQLClient, action: SumsubAction, json: bool) -> Result<()> {
    match action {
        SumsubAction::PermalinkCreate { prospect_id } => {
            let vars = sumsub_permalink_create::Variables {
                input: sumsub_permalink_create::SumsubPermalinkCreateInput { prospect_id },
            };
            let data = client.execute::<SumsubPermalinkCreate>(vars).await?;
            let result = data.sumsub_permalink_create;
            if json {
                output::print_json(&result)?;
            } else {
                output::print_kv(&[("URL", &result.url)]);
            }
        }
        SumsubAction::TestApplicantCreate { prospect_id } => {
            let variables = json!({
                "input": {
                    "prospectId": prospect_id,
                }
            });
            let data: SumsubTestApplicantCreateResponse = client
                .execute_raw(
                    SUMSUB_TEST_APPLICANT_CREATE_MUTATION,
                    variables,
                    Some("sumsubTestApplicantCreate"),
                )
                .await?;
            let result = data.sumsub_test_applicant_create;
            if json {
                output::print_json(&result)?;
            } else {
                output::print_kv(&[("Applicant ID", &result.applicant_id)]);
            }
        }
    }
    Ok(())
}
