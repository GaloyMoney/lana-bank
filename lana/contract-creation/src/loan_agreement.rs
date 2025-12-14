use audit::AuditSvc;
use authz::PermissionCheck;
use core_customer::{CoreCustomerAction, CoreCustomerEvent, CustomerId, CustomerObject, Customers};
use outbox::OutboxEventMarker;
use tracing::instrument;

use crate::{
    Applicants, LoanAgreementData, error::ContractCreationError, templates::ContractTemplates,
};

/// Generate a loan agreement PDF for a customer
#[instrument(
    name = "contract_creation.generate_loan_agreement_pdf",
    skip(customers, applicants, contract_templates, renderer),
    fields(customer_id = %customer_id),
    err
)]
pub async fn generate_loan_agreement_pdf<Perms, E>(
    customer_id: CustomerId,
    customers: &Customers<Perms, E>,
    applicants: &Applicants<Perms, E>,
    contract_templates: &ContractTemplates,
    renderer: &rendering::Renderer,
) -> Result<Vec<u8>, ContractCreationError>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCustomerAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CustomerObject>,
{
    // Find the customer for this loan agreement
    let customer = customers.find_by_id_without_audit(customer_id).await?;

    // Get applicant information from Sumsub if available
    let (full_name, address, country) = if customer.applicant_id.is_some() {
        match applicants
            .get_applicant_info_without_audit(customer_id)
            .await
        {
            Ok(applicant_info) => (
                applicant_info
                    .full_name()
                    .unwrap_or_else(|| "N/A".to_string()),
                applicant_info.primary_address().map(|s| s.to_string()),
                applicant_info.nationality().map(|s| s.to_string()),
            ),
            Err(_) => ("N/A (applicant info not available)".to_string(), None, None),
        }
    } else {
        ("N/A (customer has no applicant)".to_string(), None, None)
    };

    let loan_data = LoanAgreementData::new(
        customer.email.clone(),
        customer.telegram_id.clone(),
        customer_id,
        full_name,
        address,
        country,
    );

    let content = contract_templates.render_template("loan_agreement", &loan_data)?;
    let pdf_bytes = renderer.render_template_to_pdf(&content)?;

    Ok(pdf_bytes)
}
