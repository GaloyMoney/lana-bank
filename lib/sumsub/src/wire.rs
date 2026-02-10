use serde::{Deserialize, Serialize};

/// Response from Sumsub permalink creation
#[derive(Deserialize, Debug)]
pub struct PermalinkResponse {
    pub url: String,
}

/// Sumsub applicant details response
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicantDetails<T = String> {
    pub id: String,
    #[serde(rename = "externalUserId")]
    pub external_user_id: T,
    #[serde(default)]
    pub info: ApplicantInfo,
    #[serde(default)]
    pub fixed_info: ApplicantInfo,
    #[serde(rename = "type")]
    pub applicant_type: String,
}

/// Applicant personal information
#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ApplicantInfo {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub country: Option<String>,
    pub country_of_birth: Option<String>,
    pub dob: Option<String>,
    pub addresses: Option<Vec<Address>>,
    pub id_docs: Option<Vec<IdDocument>>,
}

impl ApplicantInfo {
    /// Get the applicant's first name
    pub fn first_name(&self) -> Option<&str> {
        self.first_name.as_deref()
    }

    /// Get the applicant's last name
    pub fn last_name(&self) -> Option<&str> {
        self.last_name.as_deref()
    }

    /// Get the applicant's full name as "FirstName LastName"
    pub fn full_name(&self) -> Option<String> {
        match (self.first_name(), self.last_name()) {
            (Some(first), Some(last)) => Some(format!("{first} {last}")),
            (Some(first), None) => Some(first.to_string()),
            (None, Some(last)) => Some(last.to_string()),
            (None, None) => None,
        }
    }

    /// Get the primary address (first in the list)
    pub fn primary_address(&self) -> Option<&str> {
        self.addresses
            .as_ref()?
            .first()?
            .formatted_address
            .as_deref()
    }

    /// Get nationality from country field or from identity documents
    pub fn nationality(&self) -> Option<&str> {
        // First try the country field in info
        if let Some(ref country) = self.country {
            return Some(country);
        }

        // Try country_of_birth from fixedInfo
        if let Some(ref country_of_birth) = self.country_of_birth {
            return Some(country_of_birth);
        }

        // If not found, try to get it from passport documents
        if let Some(ref id_docs) = self.id_docs {
            for doc in id_docs {
                if doc.doc_type == "PASSPORT"
                    && let Some(ref country) = doc.country
                {
                    return Some(country);
                }
            }
        }
        None
    }
}

/// Address information
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Address {
    pub formatted_address: Option<String>,
}

/// Identity document information
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IdDocument {
    #[serde(rename = "idDocType")]
    pub doc_type: String,
    pub country: Option<String>,
}

/// Internal API error response
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ApiError {
    pub description: String,
    pub code: u16,
    #[serde(default)]
    pub correlation_id: Option<String>,
    #[serde(default)]
    pub error_code: Option<u32>,
    #[serde(default)]
    pub error_name: Option<String>,
}

/// Internal Sumsub API response wrapper
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub(crate) enum SumsubResponse<T> {
    Success(T),
    Error(ApiError),
}

// Testing constants (only available with sumsub-testing feature)
#[cfg(feature = "sumsub-testing")]
pub mod testing {
    // Document types
    pub const DOC_TYPE_PASSPORT: &str = "PASSPORT";
    pub const DOC_TYPE_SELFIE: &str = "SELFIE";
    pub const DOC_TYPE_UTILITY_BILL: &str = "UTILITY_BILL";

    // Document subtypes
    pub const DOC_SUBTYPE_FRONT_SIDE: &str = "FRONT_SIDE";
    pub const DOC_SUBTYPE_BACK_SIDE: &str = "BACK_SIDE";

    // Review answers
    pub const REVIEW_ANSWER_GREEN: &str = "GREEN";
    pub const REVIEW_ANSWER_RED: &str = "RED";

    // Test data
    pub const TEST_FIRST_NAME: &str = "John";
    pub const TEST_LAST_NAME: &str = "Mock-Doe";
    pub const TEST_DATE_OF_BIRTH: &str = "1990-01-01";
    pub const TEST_COUNTRY_CODE: &str = "DEU";
    pub const TEST_QUESTIONNAIRE_ID: &str = "v1_onboarding";

    // Questionnaire defaults
    pub const DEFAULT_QUESTIONNAIRE_SECTION: &str = "testSumsubQuestionar";
    pub const DEFAULT_QUESTIONNAIRE_ITEM: &str = "test";
    pub const DEFAULT_QUESTIONNAIRE_VALUE: &str = "0";
}
