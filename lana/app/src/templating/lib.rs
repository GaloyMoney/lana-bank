use crate::customer::Customers;
use crate::templating::config::TemplatingConfig;
use crate::templating::error::TemplatingError;
use crate::templating::pdf::PdfGenerator;
use crate::templating::template::TemplateEngine;

#[derive(Clone)]
pub struct Templating {
    pub pdf_generator: PdfGenerator,
    pub template_engine: TemplateEngine,
    customers: Customers,
}

impl Templating {
    pub async fn init(
        config: TemplatingConfig,
        customers: &Customers,
    ) -> Result<Self, TemplatingError> {
        let template_engine = TemplateEngine::init(config.template_dir.clone()).await?;
        let pdf_generator = PdfGenerator::init(config.pdf).await?;

        Ok(Self {
            pdf_generator,
            template_engine,
            customers: customers.clone(),
        })
    }

    async fn generate_pdf_from_template<T: serde::Serialize>(
        &self,
        template_name: &str,
        data: &T,
    ) -> Result<Vec<u8>, TemplatingError> {
        let rendered_content = self.template_engine.render(template_name, data).await?;
        self.pdf_generator
            .generate_pdf_from_markdown(&rendered_content)
            .await
    }

    pub async fn generate_loan_agreement_pdf(
        &self,
        customer_id: crate::customer::CustomerId,
    ) -> Result<Vec<u8>, TemplatingError> {
        let customer = self.customers.find_by_id_without_audit(customer_id).await?;

        let loan_data = crate::templating::template::LoanAgreementData::new(customer.email.clone());

        self.generate_pdf_from_template("loan_agreement", &loan_data)
            .await
    }
}
