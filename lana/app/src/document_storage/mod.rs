use sqlx::PgPool;

use authz::PermissionCheck;
use crate::authorization::Authorization;
use crate::job::Jobs;
use crate::storage::Storage;

use core_document_storage::{LoanAgreements, DocumentStorage};

#[derive(Clone)]
pub struct DocumentStorageApp {
    document_storage: DocumentStorage,
    loan_agreements: LoanAgreements<Authorization>,
}

impl DocumentStorageApp {
    pub fn new(
        pool: &PgPool,
        authz: &Authorization,
        jobs: &Jobs,
        storage: &Storage,
    ) -> Self {
        let document_storage = DocumentStorage::new(pool, storage);
        let loan_agreements = LoanAgreements::new(pool, authz, jobs, storage);

        Self {
            document_storage,
            loan_agreements,
        }
    }

    pub fn documents(&self) -> &DocumentStorage {
        &self.document_storage
    }

    pub fn loan_agreements(&self) -> &LoanAgreements<Authorization> {
        &self.loan_agreements
    }
}