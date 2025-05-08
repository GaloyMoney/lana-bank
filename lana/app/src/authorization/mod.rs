mod seed;

use crate::audit::Audit;

pub use authz::error;
use authz::error::AuthorizationError;
use core_accounting::{CoreAccountingAction, CoreAccountingObject};
use core_credit::{CoreCreditAction, CoreCreditObject};
use core_customer::{CoreCustomerAction, CustomerObject};
pub use core_user::{CoreUserAction, UserObject};
use deposit::{CoreDepositAction, CoreDepositObject};
use governance::{GovernanceAction, GovernanceObject};
pub use rbac_types::{AppAction as Action, AppObject as Object, *};

pub type Authorization = authz::Authorization<Audit, Role>;

pub async fn init(pool: &sqlx::PgPool, audit: &Audit) -> Result<Authorization, AuthorizationError> {
    let authz = Authorization::init(pool, audit).await?;

    seed::execute(&authz).await?;

    Ok(authz)
}

pub async fn get_visible_navigation_items(
    authz: &Authorization,
    sub: &Subject,
) -> Result<VisibleNavigationItems, AuthorizationError> {
    Ok(VisibleNavigationItems {
        term: authz
            .check_all_permissions(
                sub,
                Object::TermsTemplate,
                &[
                    Action::TermsTemplate(TermsTemplateAction::Read),
                    Action::TermsTemplate(TermsTemplateAction::List),
                ],
            )
            .await?,
        user: authz
            .check_all_permissions(
                sub,
                UserObject::all_users(),
                &[CoreUserAction::USER_READ, CoreUserAction::USER_LIST],
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
            .check_all_permissions(sub, Object::Audit, &[Action::Audit(AuditAction::List)])
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
