use std::sync::Arc;
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

const MODEL: &str = include_str!("./rbac.conf");

#[derive(Clone)]
pub struct Authorization {
    enforcer: Arc<RwLock<Enforcer>>,
}

impl Authorization {
    pub async fn init(pool: &sqlx::PgPool) -> Result<Self, AuthorizationError> {
        let model = DefaultModel::from_str(MODEL).await?;
        let adapter = SqlxAdapter::new_with_pool(pool.clone()).await?;

        let enforcer = Enforcer::new(model, adapter).await?;

        let mut auth = Authorization {
            enforcer: Arc::new(RwLock::new(enforcer)),
        };

        auth.seed_roles().await?;

        Ok(auth)
    }

    async fn seed_roles(&mut self) -> Result<(), AuthorizationError> {
        let role = Role::SuperUser;

        self.add_permission_to_role(&role, Object::Loan, Action::Loan(LoanAction::Read))
            .await?;

        self.add_permission_to_role(&role, Object::Loan, Action::Loan(LoanAction::List))
            .await?;

        self.add_permission_to_role(&role, Object::Loan, Action::Loan(LoanAction::Create))
            .await?;

        self.add_permission_to_role(&role, Object::Loan, Action::Loan(LoanAction::Approve))
            .await?;

        self.add_permission_to_role(&role, Object::Loan, Action::Loan(LoanAction::RecordPayment))
            .await?;

        self.add_permission_to_role(&role, Object::Term, Action::Term(TermAction::Update))
            .await?;

        self.add_permission_to_role(&role, Object::Term, Action::Term(TermAction::Read))
            .await?;

        Ok(())
    }

    pub async fn check_permission(
        &self,
        sub: &Subject,
        object: Object,
        action: Action,
    ) -> Result<bool, AuthorizationError> {
        let enforcer = self.enforcer.read().await;

        match enforcer.enforce((sub.as_ref(), object.as_ref(), action.as_ref())) {
            Ok(true) => Ok(true),
            Ok(false) => Err(AuthorizationError::NotAuthorized),
            Err(e) => Err(AuthorizationError::Casbin(e)),
        }
    }

    pub async fn add_permission_to_role(
        &mut self,
        role: &Role,
        object: Object,
        action: Action,
    ) -> Result<(), AuthorizationError> {
        let mut enforcer = self.enforcer.write().await;

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
                AuthorizationError::DuplicateRule(_) => Ok(()),
                e => Err(e),
            },
        }
    }

    pub async fn assign_role_to_subject(
        &mut self,
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
                AuthorizationError::DuplicateRule(_) => Ok(()),
                e => Err(e),
            },
        }
    }
}

pub enum Object {
    Applicant,
    Loan,
    Term,
}

impl AsRef<str> for Object {
    fn as_ref(&self) -> &str {
        match self {
            Object::Applicant => "applicant",
            Object::Loan => "loan",
            Object::Term => "term",
        }
    }
}

impl std::ops::Deref for Object {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        match self {
            Object::Applicant => "applicant",
            Object::Loan => "loan",
            Object::Term => "term",
        }
    }
}

pub enum Action {
    Loan(LoanAction),
    Term(TermAction),
}

impl AsRef<str> for Action {
    fn as_ref(&self) -> &str {
        match self {
            Action::Loan(action) => action.as_ref(),
            Action::Term(action) => action.as_ref(),
        }
    }
}

impl std::ops::Deref for Action {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        match self {
            Action::Loan(action) => action.as_ref(),
            Action::Term(action) => action.as_ref(),
        }
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
        match self {
            LoanAction::Read => "loan-read",
            LoanAction::Create => "loan-create",
            LoanAction::List => "loan-list",
            LoanAction::Approve => "loan-approve",
            LoanAction::RecordPayment => "loan-record-payment",
        }
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
        match self {
            TermAction::Update => "term-update",
            TermAction::Read => "term-read",
        }
    }
}
