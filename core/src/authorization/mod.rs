use std::{str::FromStr, sync::Arc};
use tokio::sync::RwLock;

pub mod error;

use error::AuthorizationError;

use crate::primitives::{Role, Subject};
use sqlx_adapter::{
    casbin::{
        prelude::{DefaultModel, Enforcer},
        CoreApi, MgmtApi,
    },
    SqlxAdapter,
};

use super::audit::Audit;

const MODEL: &str = include_str!("./rbac.conf");

#[derive(Clone)]
pub struct Authorization {
    enforcer: Arc<RwLock<Enforcer>>,
    audit: Audit,
}

impl Authorization {
    pub async fn init(pool: &sqlx::PgPool, audit: Audit) -> Result<Self, AuthorizationError> {
        let model = DefaultModel::from_str(MODEL).await?;
        let adapter = SqlxAdapter::new_with_pool(pool.clone()).await?;

        let enforcer = Enforcer::new(model, adapter).await?;

        let mut auth = Authorization {
            enforcer: Arc::new(RwLock::new(enforcer)),
            audit,
        };

        auth.seed_roles().await?;
        auth.seed_role_hierarchy().await?;

        Ok(auth)
    }

    async fn seed_roles(&mut self) -> Result<(), AuthorizationError> {
        self.add_permissions_for_superuser().await?;
        self.add_permissions_for_bank_manager().await?;
        self.add_permissions_for_admin().await?;

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

        self.add_permission_to_role(&role, Object::User, UserAction::AssignRole(Role::Admin))
            .await?;
        self.add_permission_to_role(&role, Object::User, UserAction::RevokeRole(Role::Admin))
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
        self.add_permission_to_role(
            &role,
            Object::User,
            UserAction::AssignRole(Role::BankManager),
        )
        .await?;
        self.add_permission_to_role(
            &role,
            Object::User,
            UserAction::RevokeRole(Role::BankManager),
        )
        .await?;
        self.add_permission_to_role(&role, Object::Ledger, LedgerAction::Read)
            .await?;
        self.add_permission_to_role(&role, Object::Audit, AuditAction::List)
            .await?;
        Ok(())
    }

    async fn add_permissions_for_bank_manager(&mut self) -> Result<(), AuthorizationError> {
        let role = Role::BankManager;

        self.add_permission_to_role(&role, Object::Loan, LoanAction::Read)
            .await?;
        self.add_permission_to_role(&role, Object::Loan, LoanAction::List)
            .await?;
        self.add_permission_to_role(&role, Object::Loan, LoanAction::Create)
            .await?;
        self.add_permission_to_role(&role, Object::Loan, LoanAction::Approve)
            .await?;
        self.add_permission_to_role(&role, Object::Loan, LoanAction::RecordPayment)
            .await?;
        self.add_permission_to_role(&role, Object::Term, TermAction::Update)
            .await?;
        self.add_permission_to_role(&role, Object::Term, TermAction::Read)
            .await?;
        self.add_permission_to_role(&role, Object::Customer, CustomerAction::Create)
            .await?;
        self.add_permission_to_role(&role, Object::Customer, CustomerAction::List)
            .await?;
        self.add_permission_to_role(&role, Object::Customer, CustomerAction::Read)
            .await?;
        self.add_permission_to_role(&role, Object::Customer, CustomerAction::Update)
            .await?;
        self.add_permission_to_role(&role, Object::Deposit, DepositAction::Record)
            .await?;
        self.add_permission_to_role(&role, Object::Deposit, DepositAction::Read)
            .await?;
        self.add_permission_to_role(&role, Object::Deposit, DepositAction::List)
            .await?;
        self.add_permission_to_role(&role, Object::Audit, AuditAction::List)
            .await?;

        Ok(())
    }

    pub async fn check_permission(
        &self,
        sub: &Subject,
        object: Object,
        action: impl Into<Action>,
    ) -> Result<bool, AuthorizationError> {
        let enforcer = self.enforcer.read().await;

        let action = action.into();
        match enforcer.enforce((sub.as_ref(), object.as_ref(), action.as_ref())) {
            Ok(true) => {
                self.audit.persist(sub, object, action, true).await?;
                Ok(true)
            }
            Ok(false) => {
                self.audit.persist(sub, object, action, false).await?;
                Err(AuthorizationError::NotAuthorized)
            }
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
        let enforcer = self.enforcer.read().await;

        let roles = enforcer
            .get_grouping_policy()
            .into_iter()
            .filter(|r| r[0] == sub.to_string())
            .map(|r| Role::from(r[1].as_str()))
            .collect();

        Ok(roles)
    }
}

pub enum Object {
    Applicant,
    Loan,
    Term,
    User,
    Customer,
    Deposit,
    Audit,
    Ledger,
}

impl AsRef<str> for Object {
    fn as_ref(&self) -> &str {
        match self {
            Object::Applicant => "applicant",
            Object::Loan => "loan",
            Object::Term => "term",
            Object::User => "user",
            Object::Deposit => "deposit",
            Object::Customer => "customer",
            Object::Audit => "audit",
            Object::Ledger => "ledger",
        }
    }
}

impl std::ops::Deref for Object {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl FromStr for Object {
    type Err = crate::authorization::AuthorizationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "applicant" => Ok(Object::Applicant),
            "loan" => Ok(Object::Loan),
            "term" => Ok(Object::Term),
            "user" => Ok(Object::User),
            "audit" => Ok(Object::Audit),
            "customer" => Ok(Object::Customer),
            "ledger" => Ok(Object::Ledger),
            _ => Err(AuthorizationError::ObjectParseError {
                value: s.to_string(),
            }),
        }
    }
}

pub enum Action {
    Loan(LoanAction),
    Term(TermAction),
    User(UserAction),
    Customer(CustomerAction),
    Deposit(DepositAction),
    Audit(AuditAction),
    Ledger(LedgerAction),
}

impl AsRef<str> for Action {
    fn as_ref(&self) -> &str {
        match self {
            Action::Loan(action) => action.as_ref(),
            Action::Term(action) => action.as_ref(),
            Action::User(action) => action.as_ref(),
            Action::Customer(action) => action.as_ref(),
            Action::Deposit(action) => action.as_ref(),
            Action::Audit(action) => action.as_ref(),
            Action::Ledger(action) => action.as_ref(),
        }
    }
}

impl FromStr for Action {
    type Err = crate::authorization::AuthorizationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "loan-read" => Ok(Action::Loan(LoanAction::Read)),
            "loan-create" => Ok(Action::Loan(LoanAction::Create)),
            "loan-list" => Ok(Action::Loan(LoanAction::List)),
            "loan-approve" => Ok(Action::Loan(LoanAction::Approve)),
            "loan-record-payment" => Ok(Action::Loan(LoanAction::RecordPayment)),
            "term-update" => Ok(Action::Term(TermAction::Update)),
            "term-read" => Ok(Action::Term(TermAction::Read)),
            "user-create" => Ok(Action::User(UserAction::Create)),
            "user-read" => Ok(Action::User(UserAction::Read)),
            "user-list" => Ok(Action::User(UserAction::List)),
            "user-update" => Ok(Action::User(UserAction::Update)),
            "user-delete" => Ok(Action::User(UserAction::Delete)),
            "user-assign-role-superuser" => {
                Ok(Action::User(UserAction::AssignRole(Role::Superuser)))
            }
            "user-assign-role-admin" => Ok(Action::User(UserAction::AssignRole(Role::Admin))),
            "user-assign-role-bank-manager" => {
                Ok(Action::User(UserAction::AssignRole(Role::BankManager)))
            }
            "user-revoke-role-superuser" => {
                Ok(Action::User(UserAction::RevokeRole(Role::Superuser)))
            }
            "user-revoke-role-admin" => Ok(Action::User(UserAction::RevokeRole(Role::Admin))),
            "user-revoke-role-bank-manager" => {
                Ok(Action::User(UserAction::RevokeRole(Role::BankManager)))
            }
            "audit-list" => Ok(Action::Audit(AuditAction::List)),
            "customer-create" => Ok(Action::Customer(CustomerAction::Create)),
            "customer-read" => Ok(Action::Customer(CustomerAction::Read)),
            "customer-list" => Ok(Action::Customer(CustomerAction::List)),
            "customer-update" => Ok(Action::Customer(CustomerAction::Update)),
            "ledger-read" => Ok(Action::Ledger(LedgerAction::Read)),
            _ => Err(AuthorizationError::ActionParseError {
                value: s.to_string(),
            }),
        }
    }
}

impl std::ops::Deref for Action {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

pub enum LoanAction {
    List,
    Read,
    Create,
    Approve,
    RecordPayment,
}

impl AsRef<str> for LoanAction {
    fn as_ref(&self) -> &str {
        match self {
            LoanAction::Read => "loan-read",
            LoanAction::Create => "loan-create",
            LoanAction::List => "loan-list",
            LoanAction::Approve => "loan-approve",
            LoanAction::RecordPayment => "loan-record-payment",
        }
    }
}

impl std::ops::Deref for LoanAction {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl From<LoanAction> for Action {
    fn from(action: LoanAction) -> Self {
        Action::Loan(action)
    }
}

pub enum TermAction {
    Update,
    Read,
}

impl AsRef<str> for TermAction {
    fn as_ref(&self) -> &str {
        match self {
            TermAction::Update => "term-update",
            TermAction::Read => "term-read",
        }
    }
}

impl std::ops::Deref for TermAction {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl From<TermAction> for Action {
    fn from(action: TermAction) -> Self {
        Action::Term(action)
    }
}

pub enum AuditAction {
    List,
}

impl AsRef<str> for AuditAction {
    fn as_ref(&self) -> &str {
        match self {
            AuditAction::List => "audit-list",
        }
    }
}

impl std::ops::Deref for AuditAction {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl From<AuditAction> for Action {
    fn from(action: AuditAction) -> Self {
        Action::Audit(action)
    }
}

pub enum UserAction {
    Create,
    Read,
    List,
    Update,
    Delete,
    AssignRole(Role),
    RevokeRole(Role),
}

impl AsRef<str> for UserAction {
    fn as_ref(&self) -> &str {
        match self {
            UserAction::Create => "user-create",
            UserAction::Read => "user-read",
            UserAction::List => "user-list",
            UserAction::Update => "user-update",
            UserAction::Delete => "user-delete",
            UserAction::AssignRole(role) => match role {
                Role::Superuser => "user-assign-role-superuser",
                Role::Admin => "user-assign-role-admin",
                Role::BankManager => "user-assign-role-bank-manager",
            },
            UserAction::RevokeRole(role) => match role {
                Role::Superuser => "user-revoke-role-superuser",
                Role::Admin => "user-revoke-role-admin",
                Role::BankManager => "user-revoke-role-bank-manager",
            },
        }
    }
}

impl std::ops::Deref for UserAction {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl From<UserAction> for Action {
    fn from(action: UserAction) -> Self {
        Action::User(action)
    }
}

pub enum CustomerAction {
    Create,
    Read,
    List,
    Update,
}

impl AsRef<str> for CustomerAction {
    fn as_ref(&self) -> &str {
        match self {
            CustomerAction::Create => "customer-create",
            CustomerAction::Read => "customer-read",
            CustomerAction::List => "customer-list",
            CustomerAction::Update => "customer-update",
        }
    }
}

impl std::ops::Deref for CustomerAction {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl From<CustomerAction> for Action {
    fn from(action: CustomerAction) -> Self {
        Action::Customer(action)
    }
}
pub enum DepositAction {
    Record,
    Read,
    List,
}

impl AsRef<str> for DepositAction {
    fn as_ref(&self) -> &str {
        match self {
            DepositAction::Record => "deposit-record",
            DepositAction::Read => "deposit-read",
            DepositAction::List => "deposit-list",
        }
    }
}
impl std::ops::Deref for DepositAction {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl From<DepositAction> for Action {
    fn from(action: DepositAction) -> Self {
        Action::Deposit(action)
    }
}

impl std::ops::Deref for LedgerAction {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

pub enum LedgerAction {
    Read,
}

impl AsRef<str> for LedgerAction {
    fn as_ref(&self) -> &str {
        match self {
            LedgerAction::Read => "ledger-read",
        }
    }
}

impl From<LedgerAction> for Action {
    fn from(action: LedgerAction) -> Self {
        Action::Ledger(action)
    }
}
