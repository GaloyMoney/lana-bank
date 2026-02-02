use core_customer::CustomerId;
use serde::{Deserialize, Serialize};

/// Data structure for loan agreement template
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LoanAgreementData {
    pub email: String,
    pub full_name: String,
    pub address: Option<String>,
    pub country: Option<String>,
    pub customer_id: String,
    pub telegram_id: String,
    pub date: String,
}

impl LoanAgreementData {
    pub fn new(
        email: String,
        telegram_id: String,
        customer_id: CustomerId,
        full_name: String,
        address: Option<String>,
        country: Option<String>,
    ) -> Self {
        let date = chrono::Utc::now().format("%Y-%m-%d").to_string();

        Self {
            email,
            full_name,
            address,
            country,
            customer_id: customer_id.to_string(),
            telegram_id,
            date,
        }
    }
}

/// Data structure for a single credit facility in the export
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreditFacilityExportItem {
    pub customer_email: String,
    pub status: String,
    pub outstanding: String,
    pub disbursed: String,
    pub cvl: String,
}

/// Data structure for credit facility export template
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreditFacilityExportData {
    pub export_date: String,
    pub facilities: Vec<CreditFacilityExportItem>,
    pub total_count: usize,
}

impl CreditFacilityExportData {
    pub fn new(facilities: Vec<CreditFacilityExportItem>) -> Self {
        let total_count = facilities.len();
        let export_date = chrono::Utc::now()
            .format("%Y-%m-%d %H:%M:%S UTC")
            .to_string();

        Self {
            export_date,
            facilities,
            total_count,
        }
    }
}
