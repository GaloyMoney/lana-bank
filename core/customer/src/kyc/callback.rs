use serde::{Deserialize, Serialize};

use super::error::KycError;
use crate::ProspectId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "UPPERCASE")]
#[strum(serialize_all = "UPPERCASE")]
pub enum ReviewAnswer {
    Green,
    Red,
}

impl std::str::FromStr for ReviewAnswer {
    type Err = KycError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GREEN" => Ok(ReviewAnswer::Green),
            "RED" => Ok(ReviewAnswer::Red),
            _ => Err(KycError::ReviewAnswerParseError(s.to_string())),
        }
    }
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(tag = "type")]
pub enum KycCallbackPayload {
    #[serde(rename = "applicantCreated")]
    #[serde(rename_all = "camelCase")]
    ApplicantCreated {
        applicant_id: String,
        inspection_id: String,
        correlation_id: String,
        level_name: String,
        external_user_id: ProspectId,
        review_status: String,
        created_at_ms: String,
        client_id: Option<String>,
        sandbox_mode: Option<bool>,
    },
    #[serde(rename = "applicantReviewed")]
    #[serde(rename_all = "camelCase")]
    ApplicantReviewed {
        applicant_id: String,
        inspection_id: String,
        correlation_id: String,
        external_user_id: ProspectId,
        level_name: String,
        review_result: ReviewResult,
        review_status: String,
        created_at_ms: String,
        sandbox_mode: Option<bool>,
    },
    #[serde(rename = "applicantPending")]
    #[serde(rename_all = "camelCase")]
    ApplicantPending {
        applicant_id: String,
        sandbox_mode: Option<bool>,
    },
    #[serde(rename = "applicantPersonalInfoChanged")]
    #[serde(rename_all = "camelCase")]
    ApplicantPersonalInfoChanged {
        applicant_id: String,
        sandbox_mode: Option<bool>,
    },
    #[serde(other)]
    Unknown,
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewResult {
    pub review_answer: ReviewAnswer,
    pub moderation_comment: Option<String>,
    pub client_comment: Option<String>,
    pub reject_labels: Option<Vec<String>>,
    pub review_reject_type: Option<String>,
}
