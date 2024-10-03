mod action;
pub mod error;
mod object;

use sqlx_adapter::{
    casbin::{
        prelude::{DefaultModel, Enforcer},
        CoreApi, MgmtApi,
    },
    SqlxAdapter,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::instrument;

use super::audit::Audit;
use crate::primitives::{AuditInfo, Role, Subject};

pub use action::*;
use error::AuthorizationError;
pub use object::*;

const MODEL: &str = include_str!("./rbac.conf");

#[derive(Clone)]
pub struct Authorization {
    enforcer: Arc<RwLock<Enforcer>>,
    audit: Audit,
}

impl Authorization {
    pub async fn init(pool: &sqlx::PgPool, audit: &Audit) -> Result<Self, AuthorizationError> {
        let model = DefaultModel::from_str(MODEL).await?;
        let adapter = SqlxAdapter::new_with_pool(pool.clone()).await?;

        let enforcer = Enforcer::new(model, adapter).await?;

        let mut auth = Authorization {
            enforcer: Arc::new(RwLock::new(enforcer)),
            audit: audit.clone(),
        };

        auth.seed_roles().await?;
        auth.seed_role_hierarchy().await?;

        Ok(auth)
    }

    async fn seed_roles(&mut self) -> Result<(), AuthorizationError> {
        self.add_permissions_for_superuser().await?;
        self.add_permissions_for_bank_manager().await?;
        self.add_permissions_for_admin().await?;
        self.add_permissions_for_accountant().await?;

        Ok(())
    }

    async fn seed_role_hierarchy(&self) -> Result<(), AuthorizationError> {
        self.add_role_hierarchy(Role::Admin, Role::Superuser)
            .await?;
        self.add_role_hierarchy(Role::BankManager, Role::Admin)
            .await?;

        Ok(())
    }

    async fn add_role_hierarchy(
        &self,
        parent_role: Role,
        child_role: Role,
    ) -> Result<(), AuthorizationError> {
        let mut enforcer = self.enforcer.write().await;

        match enforcer
            .add_grouping_policy(vec![child_role.to_string(), parent_role.to_string()])
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => match AuthorizationError::from(e) {
                AuthorizationError::PermissionAlreadyExistsForRole(_) => Ok(()),
                e => Err(e),
            },
        }
    }

    async fn add_permissions_for_superuser(&mut self) -> Result<(), AuthorizationError> {
        let role = Role::Superuser;

        self.add_permission_to_role(&role, Object::User, UserAction::AssignRole)
            .await?;
        self.add_permission_to_role(&role, Object::User, UserAction::RevokeRole)
            .await?;
        Ok(())
    }

    async fn add_permissions_for_admin(&mut self) -> Result<(), AuthorizationError> {
        let role = Role::Admin;

        self.add_permission_to_role(&role, Object::User, UserAction::Create)
            .await?;
        self.add_permission_to_role(&role, Object::User, UserAction::List)
            .await?;
        self.add_permission_to_role(&role, Object::User, UserAction::Read)
            .await?;
        self.add_permission_to_role(&role, Object::User, UserAction::Update)
            .await?;
        self.add_permission_to_role(&role, Object::User, UserAction::Delete)
            .await?;
        self.add_permission_to_role(&role, Object::User, UserAction::AssignRole)
            .await?;
        self.add_permission_to_role(&role, Object::User, UserAction::RevokeRole)
            .await?;

        self.add_permission_to_role(&role, Object::Ledger, LedgerAction::Read)
            .await?;
        self.add_permission_to_role(&role, Object::Audit, AuditAction::List)
            .await?;
        self.add_permission_to_role(&role, Object::Report, ReportAction::Create)
            .await?;
        self.add_permission_to_role(&role, Object::Report, ReportAction::Read)
            .await?;
        self.add_permission_to_role(&role, Object::Report, ReportAction::List)
            .await?;
        self.add_permission_to_role(&role, Object::Report, ReportAction::GenerateDownloadLink)
            .await?;
        self.add_permission_to_role(&role, Object::CreditFacility, CreditFacilityAction::Create)
            .await?;
        self.add_permission_to_role(&role, Object::CreditFacility, CreditFacilityAction::Approve)
            .await?;
        self.add_permission_to_role(
            &role,
            Object::CreditFacility,
            CreditFacilityAction::InitiateDisbursement,
        )
        .await?;
        self.add_permission_to_role(
            &role,
            Object::CreditFacility,
            CreditFacilityAction::ApproveDisbursement,
        )
        .await?;

        Ok(())
    }

    async fn add_permissions_for_bank_manager(&mut self) -> Result<(), AuthorizationError> {
        let role = Role::BankManager;

        self.add_permission_to_role(
            &role,
            Object::Customer(CustomerAllOrOne::All),
            LoanAction::Create,
        )
        .await?;
        self.add_permission_to_role(&role, Object::Loan(LoanAllOrOne::All), LoanAction::Read)
            .await?;
        self.add_permission_to_role(&role, Object::Loan(LoanAllOrOne::All), LoanAction::List)
            .await?;
        self.add_permission_to_role(&role, Object::Loan(LoanAllOrOne::All), LoanAction::Approve)
            .await?;
        self.add_permission_to_role(
            &role,
            Object::Loan(LoanAllOrOne::All),
            LoanAction::RecordPayment,
        )
        .await?;
        self.add_permission_to_role(
            &role,
            Object::Loan(LoanAllOrOne::All),
            LoanAction::UpdateCollateral,
        )
        .await?;
        self.add_permission_to_role(
            &role,
            Object::Loan(LoanAllOrOne::All),
            LoanAction::UpdateCollateralizationState,
        )
        .await?;
        self.add_permission_to_role(&role, Object::TermsTemplate, TermsTemplateAction::Update)
            .await?;
        self.add_permission_to_role(&role, Object::TermsTemplate, TermsTemplateAction::Read)
            .await?;
        self.add_permission_to_role(&role, Object::TermsTemplate, TermsTemplateAction::Create)
            .await?;
        self.add_permission_to_role(&role, Object::TermsTemplate, TermsTemplateAction::List)
            .await?;
        self.add_permission_to_role(
            &role,
            Object::Customer(CustomerAllOrOne::All),
            CustomerAction::Create,
        )
        .await?;
        self.add_permission_to_role(
            &role,
            Object::Customer(CustomerAllOrOne::All),
            CustomerAction::List,
        )
        .await?;
        self.add_permission_to_role(
            &role,
            Object::Customer(CustomerAllOrOne::All),
            CustomerAction::Read,
        )
        .await?;
        self.add_permission_to_role(
            &role,
            Object::Customer(CustomerAllOrOne::All),
            CustomerAction::Update,
        )
        .await?;
        self.add_permission_to_role(&role, Object::Deposit, DepositAction::Record)
            .await?;
        self.add_permission_to_role(&role, Object::Deposit, DepositAction::Read)
            .await?;
        self.add_permission_to_role(&role, Object::Deposit, DepositAction::List)
            .await?;
        self.add_permission_to_role(&role, Object::Withdraw, WithdrawAction::Initiate)
            .await?;
        self.add_permission_to_role(&role, Object::Withdraw, WithdrawAction::Confirm)
            .await?;
        self.add_permission_to_role(&role, Object::Withdraw, WithdrawAction::Cancel)
            .await?;
        self.add_permission_to_role(&role, Object::Withdraw, WithdrawAction::Read)
            .await?;
        self.add_permission_to_role(&role, Object::Withdraw, WithdrawAction::List)
            .await?;

        Ok(())
    }

    async fn add_permissions_for_accountant(&mut self) -> Result<(), AuthorizationError> {
        let role = Role::Accountant;

        self.add_permission_to_role(&role, Object::Loan(LoanAllOrOne::All), LoanAction::Read)
            .await?;
        self.add_permission_to_role(&role, Object::Loan(LoanAllOrOne::All), LoanAction::List)
            .await?;
        self.add_permission_to_role(&role, Object::TermsTemplate, TermsTemplateAction::Read)
            .await?;
        self.add_permission_to_role(
            &role,
            Object::Customer(CustomerAllOrOne::All),
            CustomerAction::List,
        )
        .await?;
        self.add_permission_to_role(
            &role,
            Object::Customer(CustomerAllOrOne::All),
            CustomerAction::Read,
        )
        .await?;
        self.add_permission_to_role(&role, Object::Deposit, DepositAction::Read)
            .await?;
        self.add_permission_to_role(&role, Object::Deposit, DepositAction::List)
            .await?;
        self.add_permission_to_role(&role, Object::Withdraw, WithdrawAction::Read)
            .await?;
        self.add_permission_to_role(&role, Object::Withdraw, WithdrawAction::List)
            .await?;

        Ok(())
    }

    async fn enforce_permission(
        &self,
        sub: &Subject,
        object: Object,
        action: impl Into<Action> + std::fmt::Debug,
    ) -> Result<bool, sqlx_adapter::casbin::Error> {
        let action = action.into();
        let mut enforcer = self.enforcer.write().await;
        enforcer.load_policy().await?;
        enforcer.enforce((sub.to_string(), object.to_string(), action.to_string()))
    }

    #[instrument(name = "lava.authz.check_permission", skip(self))]
    pub async fn check_permission(
        &self,
        sub: &Subject,
        object: Object,
        action: impl Into<Action> + std::fmt::Debug + std::marker::Copy,
    ) -> Result<AuditInfo, AuthorizationError> {
        let result = self.enforce_permission(sub, object, action).await;
        match result {
            Ok(true) => Ok(self.audit.record_entry(sub, object, action, true).await?),
            Ok(false) => {
                self.audit.record_entry(sub, object, action, false).await?;
                Err(AuthorizationError::NotAuthorized)
            }
            Err(e) => Err(AuthorizationError::Casbin(e)),
        }
    }

    #[instrument(name = "lava.authz.check_permission_without_audit", skip(self))]
    pub async fn check_permission_without_audit_trail(
        &self,
        sub: &Subject,
        object: Object,
        action: impl Into<Action> + std::fmt::Debug,
    ) -> Result<(), AuthorizationError> {
        match self.enforce_permission(sub, object, action).await {
            Ok(true) => Ok(()),
            Ok(false) => Err(AuthorizationError::NotAuthorized),
            Err(e) => Err(AuthorizationError::Casbin(e)),
        }
    }

    pub async fn add_permission_to_role(
        &self,
        role: &Role,
        object: Object,
        action: impl Into<Action>,
    ) -> Result<(), AuthorizationError> {
        let mut enforcer = self.enforcer.write().await;

        let action = action.into();
        match enforcer
            .add_policy(vec![
                role.to_string(),
                object.to_string(),
                action.to_string(),
            ])
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => match AuthorizationError::from(e) {
                AuthorizationError::PermissionAlreadyExistsForRole(_) => Ok(()),
                e => Err(e),
            },
        }
    }

    pub async fn assign_role_to_subject(
        &self,
        sub: impl Into<Subject>,
        role: &Role,
    ) -> Result<(), AuthorizationError> {
        let sub: Subject = sub.into();
        let mut enforcer = self.enforcer.write().await;

        match enforcer
            .add_grouping_policy(vec![sub.to_string(), role.to_string()])
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => match AuthorizationError::from(e) {
                AuthorizationError::PermissionAlreadyExistsForRole(_) => Ok(()),
                e => Err(e),
            },
        }
    }

    pub async fn revoke_role_from_subject(
        &self,
        sub: impl Into<Subject>,
        role: &Role,
    ) -> Result<(), AuthorizationError> {
        let sub: Subject = sub.into();
        let mut enforcer = self.enforcer.write().await;

        match enforcer
            .remove_grouping_policy(vec![sub.to_string(), role.to_string()])
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(AuthorizationError::from(e)),
        }
    }

    pub async fn roles_for_subject(
        &self,
        sub: impl Into<Subject>,
    ) -> Result<Vec<Role>, AuthorizationError> {
        let sub: Subject = sub.into();
        let sub_uuid = sub.to_string();
        let enforcer = self.enforcer.read().await;

        let roles = enforcer
            .get_grouping_policy()
            .into_iter()
            .filter(|r| r[0] == sub_uuid)
            .map(|r| r[1].parse().expect("Could not parse role"))
            .collect();

        Ok(roles)
    }

    async fn check_all_permissions(
        &self,
        sub: &Subject,
        object: Object,
        actions: &[Action],
    ) -> Result<bool, AuthorizationError> {
        for action in actions {
            match self.check_permission(sub, object, *action).await {
                Ok(_) => continue,
                Err(AuthorizationError::NotAuthorized) => return Ok(false),
                Err(e) => return Err(e),
            }
        }
        Ok(true)
    }

    pub async fn get_visible_navigation_items(
        &self,
        sub: &Subject,
    ) -> Result<VisibleNavigationItems, AuthorizationError> {
        Ok(VisibleNavigationItems {
            loan: self
                .check_all_permissions(
                    sub,
                    Object::Loan(LoanAllOrOne::All),
                    &[
                        Action::Loan(LoanAction::Read),
                        Action::Loan(LoanAction::List),
                    ],
                )
                .await?,
            term: self
                .check_all_permissions(
                    sub,
                    Object::TermsTemplate,
                    &[
                        Action::TermsTemplate(TermsTemplateAction::Read),
                        Action::TermsTemplate(TermsTemplateAction::List),
                    ],
                )
                .await?,
            user: self
                .check_all_permissions(
                    sub,
                    Object::User,
                    &[
                        Action::User(UserAction::Read),
                        Action::User(UserAction::List),
                    ],
                )
                .await?,
            customer: self
                .check_all_permissions(
                    sub,
                    Object::Customer(CustomerAllOrOne::All),
                    &[
                        Action::Customer(CustomerAction::Read),
                        Action::Customer(CustomerAction::List),
                    ],
                )
                .await?,
            deposit: self
                .check_all_permissions(
                    sub,
                    Object::Deposit,
                    &[
                        Action::Deposit(DepositAction::Read),
                        Action::Deposit(DepositAction::List),
                    ],
                )
                .await?,
            withdraw: self
                .check_all_permissions(
                    sub,
                    Object::Withdraw,
                    &[
                        Action::Withdraw(WithdrawAction::Read),
                        Action::Withdraw(WithdrawAction::List),
                    ],
                )
                .await?,
            audit: self
                .check_all_permissions(sub, Object::Audit, &[Action::Audit(AuditAction::List)])
                .await?,
            financials: self
                .check_all_permissions(sub, Object::Ledger, &[Action::Ledger(LedgerAction::Read)])
                .await?,
        })
    }
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
}
