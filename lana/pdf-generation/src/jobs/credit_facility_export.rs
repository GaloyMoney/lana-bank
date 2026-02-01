use std::marker::PhantomData;

use async_trait::async_trait;
use authz::PermissionCheck;
use serde::{Deserialize, Serialize};

use audit::AuditSvc;
use core_credit::{CoreCreditAction, CoreCreditEvent, CoreCreditObject};
use core_custody::{CoreCustodyAction, CoreCustodyObject};
use core_customer::{CoreCustomerAction, CoreCustomerEvent, CustomerObject};
use document_storage::{DocumentId, DocumentStorage};
use governance::{GovernanceAction, GovernanceObject};
use job::*;
use obix::out::OutboxEventMarker;
use tracing_macros::record_error_severity;

use crate::{
    CreditFacilities, Customers,
    document_types::{CreditFacilityExportData, CreditFacilityExportItem},
    templates::PdfTemplates,
};

#[derive(Serialize, Deserialize)]
pub struct GenerateCreditFacilityExportConfig<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<governance::GovernanceEvent>
        + OutboxEventMarker<core_custody::CoreCustodyEvent>
        + OutboxEventMarker<core_price::CorePriceEvent>,
{
    #[serde(skip)]
    pub phantom: PhantomData<(Perms, E)>,
}

impl<Perms, E> Clone for GenerateCreditFacilityExportConfig<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<governance::GovernanceEvent>
        + OutboxEventMarker<core_custody::CoreCustodyEvent>
        + OutboxEventMarker<core_price::CorePriceEvent>,
{
    fn clone(&self) -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

pub struct GenerateCreditFacilityExportJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>
        + From<CoreCustomerAction>
        + From<governance::GovernanceAction>
        + From<core_custody::CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>
        + From<CustomerObject>
        + From<governance::GovernanceObject>
        + From<core_custody::CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<governance::GovernanceEvent>
        + OutboxEventMarker<core_custody::CoreCustodyEvent>
        + OutboxEventMarker<core_price::CorePriceEvent>,
{
    credit_facilities: CreditFacilities<Perms, E>,
    customers: Customers<Perms, E>,
    document_storage: DocumentStorage,
    pdf_templates: PdfTemplates,
    renderer: rendering::Renderer,
    authz: Perms,
}

impl<Perms, E> GenerateCreditFacilityExportJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>
        + From<CoreCustomerAction>
        + From<governance::GovernanceAction>
        + From<core_custody::CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>
        + From<CustomerObject>
        + From<governance::GovernanceObject>
        + From<core_custody::CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<governance::GovernanceEvent>
        + OutboxEventMarker<core_custody::CoreCustodyEvent>
        + OutboxEventMarker<core_price::CorePriceEvent>,
{
    pub fn new(
        credit_facilities: &CreditFacilities<Perms, E>,
        customers: &Customers<Perms, E>,
        document_storage: &DocumentStorage,
        pdf_templates: PdfTemplates,
        renderer: rendering::Renderer,
        authz: &Perms,
    ) -> Self
    where
        <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
            From<GovernanceObject> + From<CoreCustodyObject>,
        <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
            From<GovernanceAction> + From<CoreCustodyAction>,
    {
        Self {
            credit_facilities: credit_facilities.clone(),
            customers: customers.clone(),
            document_storage: document_storage.clone(),
            pdf_templates,
            renderer,
            authz: authz.clone(),
        }
    }
}

pub const GENERATE_CREDIT_FACILITY_EXPORT_JOB: JobType =
    JobType::new("task.generate-credit-facility-export");

impl<Perms, E> JobInitializer for GenerateCreditFacilityExportJobInitializer<Perms, E>
where
    Perms: PermissionCheck + Send + Sync,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>
        + From<CoreCustomerAction>
        + From<governance::GovernanceAction>
        + From<core_custody::CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>
        + From<CustomerObject>
        + From<governance::GovernanceObject>
        + From<core_custody::CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<governance::GovernanceEvent>
        + OutboxEventMarker<core_custody::CoreCustodyEvent>
        + OutboxEventMarker<core_price::CorePriceEvent>
        + Send
        + Sync,
{
    type Config = GenerateCreditFacilityExportConfig<Perms, E>;
    fn job_type(&self) -> JobType {
        GENERATE_CREDIT_FACILITY_EXPORT_JOB
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(GenerateCreditFacilityExportJobRunner {
            _config: job.config()?,
            credit_facilities: self.credit_facilities.clone(),
            customers: self.customers.clone(),
            document_storage: self.document_storage.clone(),
            pdf_templates: self.pdf_templates.clone(),
            renderer: self.renderer.clone(),
            _authz: self.authz.clone(),
        }))
    }
}

pub struct GenerateCreditFacilityExportJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>
        + From<CoreCustomerAction>
        + From<governance::GovernanceAction>
        + From<core_custody::CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>
        + From<CustomerObject>
        + From<governance::GovernanceObject>
        + From<core_custody::CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<governance::GovernanceEvent>
        + OutboxEventMarker<core_custody::CoreCustodyEvent>
        + OutboxEventMarker<core_price::CorePriceEvent>,
{
    _config: GenerateCreditFacilityExportConfig<Perms, E>,
    credit_facilities: CreditFacilities<Perms, E>,
    customers: Customers<Perms, E>,
    document_storage: DocumentStorage,
    pdf_templates: PdfTemplates,
    renderer: rendering::Renderer,
    _authz: Perms,
}

#[async_trait]
impl<Perms, E> JobRunner for GenerateCreditFacilityExportJobRunner<Perms, E>
where
    Perms: PermissionCheck + Send + Sync,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>
        + From<CoreCustomerAction>
        + From<governance::GovernanceAction>
        + From<core_custody::CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>
        + From<CustomerObject>
        + From<governance::GovernanceObject>
        + From<core_custody::CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<governance::GovernanceEvent>
        + OutboxEventMarker<core_custody::CoreCustodyEvent>
        + OutboxEventMarker<core_price::CorePriceEvent>
        + Send
        + Sync,
{
    #[record_error_severity]
    #[tracing::instrument(
        name = "pdf_generation.generate_credit_facility_export_job.run",
        skip_all,
        fields(
            job_id = %current_job.id(),
            job_attempt = current_job.attempt(),
        ),
    )]
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        use audit::SystemSubject;
        use core_credit::{CreditFacilitiesFilter, CreditFacilitiesSortBy, ListDirection, Sort};

        // Use system subject for this background job
        let system_subject = <<Perms as PermissionCheck>::Audit as AuditSvc>::Subject::system();

        // List all credit facilities (no pagination for now - gets all facilities)
        // Note: We use WithStatus filter with Active status, but in the future we might want all statuses
        let facilities_result = self
            .credit_facilities
            .list(
                &system_subject,
                Default::default(),
                CreditFacilitiesFilter::WithStatus(core_credit::CreditFacilityStatus::Active),
                Sort {
                    by: CreditFacilitiesSortBy::CreatedAt,
                    direction: ListDirection::Descending,
                },
            )
            .await?;

        // Build export items by fetching customer data for each facility
        let mut export_items = Vec::new();
        for facility in facilities_result.entities {
            // Get customer information
            let customer = self
                .customers
                .find_by_id_without_audit(facility.customer_id)
                .await?;

            let cvl = facility.last_collateralization_ratio();

            let export_item = CreditFacilityExportItem {
                customer_email: customer.email.clone(),
                status: format!("{:?}", facility.status()),
                outstanding: "0".to_string(), // Placeholder - would need ledger access
                disbursed: format!("{}", facility.amount),
                cvl: format!("{:?}", cvl),
            };

            export_items.push(export_item);
        }

        let export_data = CreditFacilityExportData::new(export_items);

        let content = self
            .pdf_templates
            .render_template("credit_facility_export", &export_data)?;
        let pdf_bytes = self.renderer.render_template_to_pdf(&content).await?;

        // Convert job ID to document ID
        let document_id = DocumentId::from(uuid::Uuid::from(*current_job.id()));

        // Find the document that was created for this job
        let mut document = self.document_storage.find_by_id(document_id).await?;

        // Upload the PDF content to the document
        self.document_storage
            .upload(pdf_bytes, &mut document)
            .await?;

        Ok(JobCompletion::Complete)
    }
}

pub type GenerateCreditFacilityExportJobSpawner<Perms, E> =
    JobSpawner<GenerateCreditFacilityExportConfig<Perms, E>>;
