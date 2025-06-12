use std::fs;
use std::path::PathBuf;

use crate::contract_creation::config::ContractCreationConfig;
use crate::contract_creation::error::ContractCreationError;
use crate::customer::Customers;

#[derive(Clone)]
pub struct ContractCreation {
    renderer: rendering::Renderer,
    template_dir: PathBuf,
    customers: Customers,
}

impl ContractCreation {
    pub async fn init(
        config: ContractCreationConfig,
        customers: &Customers,
    ) -> Result<Self, ContractCreationError> {
        let renderer = rendering::Renderer::new(config.pdf_config_file).await?;

        Ok(Self {
            renderer,
            template_dir: config.template_dir,
            customers: customers.clone(),
        })
    }

    async fn load_template(&self, template_name: &str) -> Result<String, ContractCreationError> {
        let template_path = self.template_dir.join(format!("{}.md.hbs", template_name));

        if !template_path.exists() {
            return Err(ContractCreationError::TemplateNotFound(
                template_name.to_string(),
            ));
        }

        let template_content = fs::read_to_string(&template_path)?;
        Ok(template_content)
    }

    async fn generate_contract_pdf_from_template<T: serde::Serialize>(
        &self,
        template_name: &str,
        data: &T,
    ) -> Result<Vec<u8>, ContractCreationError> {
        let template_content = self.load_template(template_name).await?;
        let pdf_bytes = self
            .renderer
            .render_template_to_pdf(&template_content, data)
            .await?;
        Ok(pdf_bytes)
    }

    pub async fn generate_loan_agreement_pdf(
        &self,
        customer_id: crate::customer::CustomerId,
    ) -> Result<Vec<u8>, ContractCreationError> {
        let customer = self.customers.find_by_id_without_audit(customer_id).await?;

        let loan_data = LoanAgreementData::new(customer.email.clone());

        self.generate_contract_pdf_from_template("loan_agreement", &loan_data)
            .await
    }
}

/// Data structure for loan agreement template
#[derive(serde::Serialize)]
pub struct LoanAgreementData {
    pub email: String,
}

impl LoanAgreementData {
    pub fn new(email: String) -> Self {
        Self { email }
    }
}
