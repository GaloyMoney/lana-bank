mod seed;

use crate::audit::Audit;

pub use authz::error;
use authz::error::AuthorizationError;
pub use core_user::{CoreUserAction, UserObject};
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
        loan: authz
            .check_all_permissions(
                sub,
                Object::Loan(LoanAllOrOne::All),
                &[
                    Action::Loan(LoanAction::Read),
                    Action::Loan(LoanAction::List),
                ],
            )
            .await?,
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
                Object::Customer(CustomerAllOrOne::All),
                &[
                    Action::Customer(CustomerAction::Read),
                    Action::Customer(CustomerAction::List),
                ],
            )
            .await?,
        deposit: authz
            .check_all_permissions(
                sub,
                Object::Deposit,
                &[
                    Action::Deposit(DepositAction::Read),
                    Action::Deposit(DepositAction::List),
                ],
            )
            .await?,
        withdraw: authz
            .check_all_permissions(
                sub,
                Object::Withdraw,
                &[
                    Action::Withdraw(WithdrawAction::Read),
                    Action::Withdraw(WithdrawAction::List),
                ],
            )
            .await?,
        audit: authz
            .check_all_permissions(sub, Object::Audit, &[Action::Audit(AuditAction::List)])
            .await?,
        financials: authz
            .check_all_permissions(sub, Object::Ledger, &[Action::Ledger(LedgerAction::Read)])
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
                Object::CreditFacility,
                &[
                    Action::CreditFacility(CreditFacilityAction::Read),
                    Action::CreditFacility(CreditFacilityAction::List),
                ],
            )
            .await?,
    })
}

#[derive(async_graphql::SimpleObject)]
pub struct VisibleNavigationItems {
    pub loan: bool,
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
