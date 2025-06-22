pub mod seed;

use crate::audit::Audit;

pub use authz::error;
use authz::error::AuthorizationError;
pub use core_access::{CoreAccessAction, CoreAccessObject};
use core_accounting::{CoreAccountingAction, CoreAccountingObject};
use core_credit::{CoreCreditAction, CoreCreditObject};
use core_customer::{CoreCustomerAction, CustomerObject};
use core_deposit::{CoreDepositAction, CoreDepositObject};
use governance::{GovernanceAction, GovernanceObject};
pub use rbac_types::{AppAction as Action, AppObject as Object, *};
use core_document_storage::{CoreDocumentStorageAction, CoreDocumentStorageObject};

pub type Authorization = authz::Authorization<Audit, core_access::AuthRoleToken>;

pub async fn get_visible_navigation_items(
    authz: &Authorization,
    sub: &Subject,
) -> Result<VisibleNavigationItems, AuthorizationError> {
    Ok(VisibleNavigationItems {
        term: authz
            .check_all_permissions(
                sub,
                CoreCreditObject::all_terms_templates(),
                &[
                    CoreCreditAction::TERMS_TEMPLATE_READ,
                    CoreCreditAction::TERMS_TEMPLATE_LIST,
                ],
            )
            .await?,
        user: authz
            .check_all_permissions(
                sub,
                CoreAccessObject::all_users(),
                &[CoreAccessAction::USER_READ, CoreAccessAction::USER_LIST],
            )
            .await?,
        customer: authz
            .check_all_permissions(
                sub,
                CustomerObject::all_customers(),
                &[
                    CoreCustomerAction::CUSTOMER_READ,
                    CoreCustomerAction::CUSTOMER_LIST,
                ],
            )
            .await?,
        deposit: authz
            .check_all_permissions(
                sub,
                CoreDepositObject::all_deposits(),
                &[
                    CoreDepositAction::DEPOSIT_READ,
                    CoreDepositAction::DEPOSIT_LIST,
                    CoreDepositAction::DEPOSIT_CREATE,
                ],
            )
            .await?,
        withdraw: authz
            .check_all_permissions(
                sub,
                CoreDepositObject::all_withdrawals(),
                &[
                    CoreDepositAction::WITHDRAWAL_READ,
                    CoreDepositAction::WITHDRAWAL_LIST,
                    CoreDepositAction::WITHDRAWAL_INITIATE,
                    CoreDepositAction::WITHDRAWAL_CONFIRM,
                    CoreDepositAction::WITHDRAWAL_CANCEL,
                    CoreDepositAction::WITHDRAWAL_CONCLUDE_APPROVAL_PROCESS,
                ],
            )
            .await?,
        audit: authz
            .check_all_permissions(
                sub,
                Object::all_audits(),
                &[Action::Audit(AuditAction::List)],
            )
            .await?,
        financials: authz
            .check_all_permissions(
                sub,
                CoreAccountingObject::all_journals(),
                &[CoreAccountingAction::JOURNAL_READ_ENTRIES],
            )
            .await?,
        governance: GovernanceNavigationItems {
            committee: authz
                .check_all_permissions(
                    sub,
                    GovernanceObject::all_committees(),
                    &[
                        GovernanceAction::COMMITTEE_READ,
                        GovernanceAction::COMMITTEE_LIST,
                    ],
                )
                .await?,
            policy: authz
                .check_all_permissions(
                    sub,
                    GovernanceObject::all_policies(),
                    &[GovernanceAction::POLICY_READ, GovernanceAction::POLICY_LIST],
                )
                .await?,
            approval_process: authz
                .check_all_permissions(
                    sub,
                    GovernanceObject::all_approval_processes(),
                    &[
                        GovernanceAction::APPROVAL_PROCESS_READ,
                        GovernanceAction::APPROVAL_PROCESS_LIST,
                    ],
                )
                .await?,
        },
        credit_facilities: authz
            .check_all_permissions(
                sub,
                CoreCreditObject::all_credit_facilities(),
                &[
                    CoreCreditAction::CREDIT_FACILITY_READ,
                    CoreCreditAction::CREDIT_FACILITY_LIST,
                ],
            )
            .await?,
    })
}

#[derive(async_graphql::SimpleObject)]
pub struct VisibleNavigationItems {
    pub term: bool,
    pub user: bool,
    pub customer: bool,
    pub deposit: bool,
    pub withdraw: bool,
    pub audit: bool,
    pub financials: bool,
    pub governance: GovernanceNavigationItems,
    pub credit_facilities: bool,
}

#[derive(async_graphql::SimpleObject)]
pub struct GovernanceNavigationItems {
    pub committee: bool,
    pub policy: bool,
    pub approval_process: bool,
}

impl From<CoreDocumentStorageAction> for AuditAction {
    fn from(action: CoreDocumentStorageAction) -> Self {
        match action {
            CoreDocumentStorageAction::DOCUMENT_CREATE => AuditAction::DocumentCreate,
            CoreDocumentStorageAction::DOCUMENT_GENERATE_DOWNLOAD_LINK => AuditAction::DocumentGenerateDownloadLink,
            CoreDocumentStorageAction::DOCUMENT_DELETE => AuditAction::DocumentDelete,
            CoreDocumentStorageAction::DOCUMENT_ARCHIVE => AuditAction::DocumentArchive,
            CoreDocumentStorageAction::LOAN_AGREEMENT_CREATE => AuditAction::LoanAgreementCreate,
            CoreDocumentStorageAction::LOAN_AGREEMENT_GENERATE => AuditAction::LoanAgreementGenerate,
            CoreDocumentStorageAction::LOAN_AGREEMENT_GENERATE_DOWNLOAD_LINK => AuditAction::LoanAgreementGenerateDownloadLink,
        }
    }
}

impl From<CoreDocumentStorageObject> for AppObject {
    fn from(object: CoreDocumentStorageObject) -> Self {
        match object {
            CoreDocumentStorageObject::Document(id) => AppObject::Document(id.into()),
            CoreDocumentStorageObject::LoanAgreement(id) => AppObject::LoanAgreement(id.into()),
            CoreDocumentStorageObject::AllDocuments => AppObject::AllDocuments,
            CoreDocumentStorageObject::AllLoanAgreements => AppObject::AllLoanAgreements,
        }
    }
}
