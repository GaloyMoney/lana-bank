use std::{str::FromStr, sync::Arc};
use tokio::sync::RwLock;

pub mod error;

use error::AuthorizationError;

use crate::primitives::{AuditInfo, Role, Subject};
use sqlx_adapter::{
    casbin::{
        prelude::{DefaultModel, Enforcer},
        CoreApi, MgmtApi,
    },
    SqlxAdapter,
};

use super::audit::Audit;

macro_rules! impl_from_for_action {
    ($from_type:ty, $variant:ident) => {
        impl From<$from_type> for Action {
            fn from(action: $from_type) -> Self {
                Action::$variant(action)
            }
        }
    };
}

macro_rules! impl_deref_to_str {
    ($type:ty) => {
        impl std::ops::Deref for $type {
            type Target = str;
            fn deref(&self) -> &Self::Target {
                self.as_ref()
            }
        }
    };
}

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
        self.add_permission_to_role(&role, Object::Loan, LoanAction::UpdateCollateral)
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
        self.add_permission_to_role(&role, Object::Audit, AuditAction::List)
            .await?;

        Ok(())
    }

    pub async fn check_permission(
        &self,
        sub: &Subject,
        object: Object,
        action: impl Into<Action>,
    ) -> Result<AuditInfo, AuthorizationError> {
        let mut enforcer = self.enforcer.write().await;
        enforcer.load_policy().await?;

        let action = action.into();
        match enforcer.enforce((sub.to_string(), object.as_ref(), action.as_ref())) {
            Ok(true) => Ok(self.audit.record_entry(sub, object, action, true).await?),
            Ok(false) => {
                self.audit.record_entry(sub, object, action, false).await?;
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
}

pub enum Object {
    Applicant,
    Loan,
    Term,
    User,
    Customer,
    Deposit,
    Withdraw,
    Audit,
    Ledger,
}

impl Object {
    const APPLICANT_STR: &'static str = "applicant";
    const LOAN_STR: &'static str = "loan";
    const TERM_STR: &'static str = "term";
    const USER_STR: &'static str = "user";
    const DEPOSIT_STR: &'static str = "deposit";
    const WITHDRAW_STR: &'static str = "withdraw";
    const CUSTOMER_STR: &'static str = "customer";
    const AUDIT_STR: &'static str = "audit";
    const LEDGER_STR: &'static str = "ledger";
}

impl AsRef<str> for Object {
    fn as_ref(&self) -> &str {
        match self {
            Self::Applicant => Self::APPLICANT_STR,
            Self::Loan => Self::LOAN_STR,
            Self::Term => Self::TERM_STR,
            Self::User => Self::USER_STR,
            Self::Deposit => Self::DEPOSIT_STR,
            Self::Withdraw => Self::WITHDRAW_STR,
            Self::Customer => Self::CUSTOMER_STR,
            Self::Audit => Self::AUDIT_STR,
            Self::Ledger => Self::LEDGER_STR,
        }
    }
}

impl_deref_to_str!(Object);

impl FromStr for Object {
    type Err = crate::authorization::AuthorizationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            Self::APPLICANT_STR => Ok(Self::Applicant),
            Self::LOAN_STR => Ok(Self::Loan),
            Self::TERM_STR => Ok(Self::Term),
            Self::USER_STR => Ok(Self::User),
            Self::AUDIT_STR => Ok(Self::Audit),
            Self::CUSTOMER_STR => Ok(Self::Customer),
            Self::DEPOSIT_STR => Ok(Self::Deposit),
            Self::WITHDRAW_STR => Ok(Self::Withdraw),
            Self::LEDGER_STR => Ok(Self::Ledger),
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
    Withdraw(WithdrawAction),
    Audit(AuditAction),
    Ledger(LedgerAction),
}

impl AsRef<str> for Action {
    fn as_ref(&self) -> &str {
        match self {
            Self::Loan(action) => action.as_ref(),
            Self::Term(action) => action.as_ref(),
            Self::User(action) => action.as_ref(),
            Self::Customer(action) => action.as_ref(),
            Self::Deposit(action) => action.as_ref(),
            Self::Withdraw(action) => action.as_ref(),
            Self::Audit(action) => action.as_ref(),
            Self::Ledger(action) => action.as_ref(),
        }
    }
}

impl FromStr for Action {
    type Err = crate::authorization::AuthorizationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            LoanAction::READ_STR => Ok(Self::Loan(LoanAction::Read)),
            LoanAction::CREATE_STR => Ok(Self::Loan(LoanAction::Create)),
            LoanAction::LIST_STR => Ok(Self::Loan(LoanAction::List)),
            LoanAction::APPROVE_STR => Ok(Self::Loan(LoanAction::Approve)),
            LoanAction::RECORD_PAYMENT_STR => Ok(Self::Loan(LoanAction::RecordPayment)),
            LoanAction::UPDATE_COLLATERAL_STR => Ok(Self::Loan(LoanAction::UpdateCollateral)),

            TermAction::UPDATE_STR => Ok(Self::Term(TermAction::Update)),
            TermAction::READ_STR => Ok(Self::Term(TermAction::Read)),

            UserAction::CREATE_STR => Ok(Self::User(UserAction::Create)),
            UserAction::READ_STR => Ok(Self::User(UserAction::Read)),
            UserAction::LIST_STR => Ok(Self::User(UserAction::List)),
            UserAction::UPDATE_STR => Ok(Self::User(UserAction::Update)),
            UserAction::DELETE_STR => Ok(Self::User(UserAction::Delete)),
            UserAction::ASSIGN_ROLE_SUPERUSER_STR => {
                Ok(Self::User(UserAction::AssignRole(Role::Superuser)))
            }
            UserAction::ASSIGN_ROLE_ADMIN_STR => {
                Ok(Self::User(UserAction::AssignRole(Role::Admin)))
            }
            UserAction::ASSIGN_ROLE_BANK_MANAGER_STR => {
                Ok(Self::User(UserAction::AssignRole(Role::BankManager)))
            }
            UserAction::REVOKE_ROLE_SUPERUSER_STR => {
                Ok(Self::User(UserAction::RevokeRole(Role::Superuser)))
            }
            UserAction::REVOKE_ROLE_ADMIN_STR => {
                Ok(Self::User(UserAction::RevokeRole(Role::Admin)))
            }
            UserAction::REVOKE_ROLE_BANK_MANAGER_STR => {
                Ok(Self::User(UserAction::RevokeRole(Role::BankManager)))
            }

            AuditAction::LIST_STR => Ok(Self::Audit(AuditAction::List)),

            CustomerAction::CREATE_STR => Ok(Self::Customer(CustomerAction::Create)),
            CustomerAction::READ_STR => Ok(Self::Customer(CustomerAction::Read)),
            CustomerAction::LIST_STR => Ok(Self::Customer(CustomerAction::List)),
            CustomerAction::UPDATE_STR => Ok(Self::Customer(CustomerAction::Update)),

            DepositAction::RECORD_STR => Ok(Self::Deposit(DepositAction::Record)),
            DepositAction::READ_STR => Ok(Self::Deposit(DepositAction::Read)),
            DepositAction::LIST_STR => Ok(Self::Deposit(DepositAction::List)),

            WithdrawAction::INITIATE_STR => Ok(Self::Withdraw(WithdrawAction::Initiate)),
            WithdrawAction::CONFIRM_STR => Ok(Self::Withdraw(WithdrawAction::Confirm)),
            WithdrawAction::READ_STR => Ok(Self::Withdraw(WithdrawAction::Read)),
            WithdrawAction::LIST_STR => Ok(Self::Withdraw(WithdrawAction::List)),

            LedgerAction::READ_STR => Ok(Self::Ledger(LedgerAction::Read)),

            _ => Err(AuthorizationError::ActionParseError {
                value: s.to_string(),
            }),
        }
    }
}

impl_deref_to_str!(Action);

pub enum LoanAction {
    List,
    Read,
    Create,
    Approve,
    RecordPayment,
    UpdateCollateral,
    RecordInterest,
}

impl LoanAction {
    const READ_STR: &'static str = "loan-read";
    const CREATE_STR: &'static str = "loan-create";
    const LIST_STR: &'static str = "loan-list";
    const APPROVE_STR: &'static str = "loan-approve";
    const RECORD_PAYMENT_STR: &'static str = "loan-record-payment";
    const UPDATE_COLLATERAL_STR: &'static str = "loan-update-collateral";
    const RECORD_INTEREST_STR: &'static str = "loan-record-interest";
}

impl AsRef<str> for LoanAction {
    fn as_ref(&self) -> &str {
        match self {
            Self::Read => Self::READ_STR,
            Self::Create => Self::CREATE_STR,
            Self::List => Self::LIST_STR,
            Self::Approve => Self::APPROVE_STR,
            Self::RecordPayment => Self::RECORD_PAYMENT_STR,
            Self::UpdateCollateral => Self::UPDATE_COLLATERAL_STR,
            Self::RecordInterest => Self::RECORD_INTEREST_STR,
        }
    }
}

impl_deref_to_str!(LoanAction);
impl_from_for_action!(LoanAction, Loan);

pub enum TermAction {
    Update,
    Read,
}

impl TermAction {
    const UPDATE_STR: &'static str = "term-update";
    const READ_STR: &'static str = "term-read";
}

impl AsRef<str> for TermAction {
    fn as_ref(&self) -> &str {
        match self {
            Self::Update => Self::UPDATE_STR,
            Self::Read => Self::READ_STR,
        }
    }
}

impl_deref_to_str!(TermAction);
impl_from_for_action!(TermAction, Term);

pub enum AuditAction {
    List,
}

impl AuditAction {
    const LIST_STR: &'static str = "audit-list";
}

impl AsRef<str> for AuditAction {
    fn as_ref(&self) -> &str {
        match self {
            Self::List => Self::LIST_STR,
        }
    }
}

impl_deref_to_str!(AuditAction);
impl_from_for_action!(AuditAction, Audit);

pub enum UserAction {
    Create,
    Read,
    List,
    Update,
    Delete,
    AssignRole(Role),
    RevokeRole(Role),
}

impl UserAction {
    const CREATE_STR: &'static str = "user-create";
    const READ_STR: &'static str = "user-read";
    const LIST_STR: &'static str = "user-list";
    const UPDATE_STR: &'static str = "user-update";
    const DELETE_STR: &'static str = "user-delete";
    const ASSIGN_ROLE_SUPERUSER_STR: &'static str = "user-assign-role-superuser";
    const ASSIGN_ROLE_ADMIN_STR: &'static str = "user-assign-role-admin";
    const ASSIGN_ROLE_BANK_MANAGER_STR: &'static str = "user-assign-role-bank-manager";
    const REVOKE_ROLE_SUPERUSER_STR: &'static str = "user-revoke-role-superuser";
    const REVOKE_ROLE_ADMIN_STR: &'static str = "user-revoke-role-admin";
    const REVOKE_ROLE_BANK_MANAGER_STR: &'static str = "user-revoke-role-bank-manager";
}

impl AsRef<str> for UserAction {
    fn as_ref(&self) -> &str {
        match self {
            Self::Create => Self::CREATE_STR,
            Self::Read => Self::READ_STR,
            Self::List => Self::LIST_STR,
            Self::Update => Self::UPDATE_STR,
            Self::Delete => Self::DELETE_STR,
            Self::AssignRole(role) => match role {
                Role::Superuser => Self::ASSIGN_ROLE_SUPERUSER_STR,
                Role::Admin => Self::ASSIGN_ROLE_ADMIN_STR,
                Role::BankManager => Self::ASSIGN_ROLE_BANK_MANAGER_STR,
            },
            Self::RevokeRole(role) => match role {
                Role::Superuser => Self::REVOKE_ROLE_SUPERUSER_STR,
                Role::Admin => Self::REVOKE_ROLE_ADMIN_STR,
                Role::BankManager => Self::REVOKE_ROLE_BANK_MANAGER_STR,
            },
        }
    }
}

impl_deref_to_str!(UserAction);
impl_from_for_action!(UserAction, User);

pub enum CustomerAction {
    Create,
    StartKyc,
    ApproveKyc,
    DeclineKyc,
    Read,
    List,
    Update,
}

impl CustomerAction {
    const CREATE_STR: &'static str = "customer-create";
    const START_KYC_STR: &'static str = "customer-start-kyc";
    const APPROVE_KYC_STR: &'static str = "customer-approve-kyc";
    const DECLINE_KYC_STR: &'static str = "customer-decline-kyc";
    const READ_STR: &'static str = "customer-read";
    const LIST_STR: &'static str = "customer-list";
    const UPDATE_STR: &'static str = "customer-update";
}

impl AsRef<str> for CustomerAction {
    fn as_ref(&self) -> &str {
        match self {
            Self::Create => Self::CREATE_STR,
            Self::StartKyc => Self::START_KYC_STR,
            Self::ApproveKyc => Self::APPROVE_KYC_STR,
            Self::DeclineKyc => Self::DECLINE_KYC_STR,
            Self::Read => Self::READ_STR,
            Self::List => Self::LIST_STR,
            Self::Update => Self::UPDATE_STR,
        }
    }
}

impl_deref_to_str!(CustomerAction);
impl_from_for_action!(CustomerAction, Customer);

pub enum DepositAction {
    Record,
    Read,
    List,
}

impl DepositAction {
    const RECORD_STR: &'static str = "deposit-record";
    const READ_STR: &'static str = "deposit-read";
    const LIST_STR: &'static str = "deposit-list";
}

impl AsRef<str> for DepositAction {
    fn as_ref(&self) -> &str {
        match self {
            Self::Record => Self::RECORD_STR,
            Self::Read => Self::READ_STR,
            Self::List => Self::LIST_STR,
        }
    }
}

impl_deref_to_str!(DepositAction);
impl_from_for_action!(DepositAction, Deposit);

pub enum WithdrawAction {
    Initiate,
    Confirm,
    Read,
    List,
    Cancel,
}

impl WithdrawAction {
    const INITIATE_STR: &'static str = "withdraw-initiate";
    const CONFIRM_STR: &'static str = "withdraw-confirm";
    const READ_STR: &'static str = "withdraw-read";
    const LIST_STR: &'static str = "withdraw-list";
    const CANCEL_STR: &'static str = "withdraw-cancel";
}

impl AsRef<str> for WithdrawAction {
    fn as_ref(&self) -> &str {
        match self {
            Self::Initiate => Self::INITIATE_STR,
            Self::Confirm => Self::CONFIRM_STR,
            Self::Read => Self::READ_STR,
            Self::List => Self::LIST_STR,
            Self::Cancel => Self::CANCEL_STR,
        }
    }
}

impl_deref_to_str!(WithdrawAction);
impl_from_for_action!(WithdrawAction, Withdraw);

pub enum LedgerAction {
    Read,
}

impl LedgerAction {
    const READ_STR: &'static str = "ledger-read";
}

impl AsRef<str> for LedgerAction {
    fn as_ref(&self) -> &str {
        match self {
            Self::Read => Self::READ_STR,
        }
    }
}

impl_deref_to_str!(LedgerAction);
impl_from_for_action!(LedgerAction, Ledger);
