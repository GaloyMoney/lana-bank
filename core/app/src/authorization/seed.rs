use authz::error::AuthorizationError;

use super::{action::*, object::*, Authorization};
use crate::primitives::Role;

pub(super) async fn execute(authz: &Authorization) -> Result<(), AuthorizationError> {
    seed_roles(authz).await?;
    seed_role_hierarchy(authz).await?;
    Ok(())
}

async fn seed_role_hierarchy(authz: &Authorization) -> Result<(), AuthorizationError> {
    authz
        .add_role_hierarchy(Role::Admin, Role::Superuser)
        .await?;
    authz
        .add_role_hierarchy(Role::BankManager, Role::Admin)
        .await?;

    Ok(())
}

async fn seed_roles(authz: &Authorization) -> Result<(), AuthorizationError> {
    add_permissions_for_superuser(authz).await?;
    add_permissions_for_bank_manager(authz).await?;
    add_permissions_for_admin(authz).await?;
    add_permissions_for_accountant(authz).await?;

    Ok(())
}

async fn add_permissions_for_superuser(authz: &Authorization) -> Result<(), AuthorizationError> {
    let role = Role::Superuser;

    authz
        .add_permission_to_role(&role, Object::User, UserAction::AssignRole)
        .await?;
    authz
        .add_permission_to_role(&role, Object::User, UserAction::RevokeRole)
        .await?;
    Ok(())
}

async fn add_permissions_for_admin(authz: &Authorization) -> Result<(), AuthorizationError> {
    let role = Role::Admin;

    authz
        .add_permission_to_role(&role, Object::User, UserAction::Create)
        .await?;
    authz
        .add_permission_to_role(&role, Object::User, UserAction::List)
        .await?;
    authz
        .add_permission_to_role(&role, Object::User, UserAction::Read)
        .await?;
    authz
        .add_permission_to_role(&role, Object::User, UserAction::Update)
        .await?;
    authz
        .add_permission_to_role(&role, Object::User, UserAction::AssignRole)
        .await?;
    authz
        .add_permission_to_role(&role, Object::User, UserAction::RevokeRole)
        .await?;

    authz
        .add_permission_to_role(&role, Object::Ledger, LedgerAction::Read)
        .await?;
    authz
        .add_permission_to_role(&role, Object::Audit, AuditAction::List)
        .await?;
    authz
        .add_permission_to_role(&role, Object::Report, ReportAction::Create)
        .await?;
    authz
        .add_permission_to_role(&role, Object::Report, ReportAction::Read)
        .await?;
    authz
        .add_permission_to_role(&role, Object::Report, ReportAction::List)
        .await?;
    authz
        .add_permission_to_role(&role, Object::Report, ReportAction::GenerateDownloadLink)
        .await?;

    Ok(())
}

async fn add_permissions_for_bank_manager(authz: &Authorization) -> Result<(), AuthorizationError> {
    let role = Role::BankManager;

    authz
        .add_permission_to_role(
            &role,
            Object::Customer(CustomerAllOrOne::All),
            LoanAction::Create,
        )
        .await?;
    authz
        .add_permission_to_role(&role, Object::Loan(LoanAllOrOne::All), LoanAction::Read)
        .await?;
    authz
        .add_permission_to_role(&role, Object::Loan(LoanAllOrOne::All), LoanAction::List)
        .await?;
    authz
        .add_permission_to_role(&role, Object::Loan(LoanAllOrOne::All), LoanAction::Approve)
        .await?;
    authz
        .add_permission_to_role(
            &role,
            Object::Loan(LoanAllOrOne::All),
            LoanAction::RecordPayment,
        )
        .await?;
    authz
        .add_permission_to_role(
            &role,
            Object::Loan(LoanAllOrOne::All),
            LoanAction::UpdateCollateral,
        )
        .await?;
    authz
        .add_permission_to_role(
            &role,
            Object::Loan(LoanAllOrOne::All),
            LoanAction::UpdateCollateralizationState,
        )
        .await?;
    authz
        .add_permission_to_role(&role, Object::TermsTemplate, TermsTemplateAction::Update)
        .await?;
    authz
        .add_permission_to_role(&role, Object::TermsTemplate, TermsTemplateAction::Read)
        .await?;
    authz
        .add_permission_to_role(&role, Object::TermsTemplate, TermsTemplateAction::Create)
        .await?;
    authz
        .add_permission_to_role(&role, Object::TermsTemplate, TermsTemplateAction::List)
        .await?;
    authz
        .add_permission_to_role(
            &role,
            Object::Customer(CustomerAllOrOne::All),
            CustomerAction::Create,
        )
        .await?;
    authz
        .add_permission_to_role(
            &role,
            Object::Customer(CustomerAllOrOne::All),
            CustomerAction::List,
        )
        .await?;
    authz
        .add_permission_to_role(
            &role,
            Object::Customer(CustomerAllOrOne::All),
            CustomerAction::Read,
        )
        .await?;
    authz
        .add_permission_to_role(
            &role,
            Object::Customer(CustomerAllOrOne::All),
            CustomerAction::Update,
        )
        .await?;
    authz
        .add_permission_to_role(&role, Object::Deposit, DepositAction::Record)
        .await?;
    authz
        .add_permission_to_role(&role, Object::Deposit, DepositAction::Read)
        .await?;
    authz
        .add_permission_to_role(&role, Object::Deposit, DepositAction::List)
        .await?;
    authz
        .add_permission_to_role(&role, Object::Withdraw, WithdrawAction::Initiate)
        .await?;
    authz
        .add_permission_to_role(&role, Object::Withdraw, WithdrawAction::Confirm)
        .await?;
    authz
        .add_permission_to_role(&role, Object::Withdraw, WithdrawAction::Cancel)
        .await?;
    authz
        .add_permission_to_role(&role, Object::Withdraw, WithdrawAction::Read)
        .await?;
    authz
        .add_permission_to_role(&role, Object::Withdraw, WithdrawAction::List)
        .await?;
    authz
        .add_permission_to_role(&role, Object::Document, DocumentAction::Create)
        .await?;
    authz
        .add_permission_to_role(&role, Object::Document, DocumentAction::Read)
        .await?;
    authz
        .add_permission_to_role(&role, Object::Document, DocumentAction::List)
        .await?;
    authz
        .add_permission_to_role(
            &role,
            Object::Document,
            DocumentAction::GenerateDownloadLink,
        )
        .await?;
    authz
        .add_permission_to_role(&role, Object::Document, DocumentAction::Delete)
        .await?;
    authz
        .add_permission_to_role(&role, Object::Document, DocumentAction::Archive)
        .await?;
    authz
        .add_permission_to_role(&role, Object::CreditFacility, CreditFacilityAction::Create)
        .await?;
    authz
        .add_permission_to_role(&role, Object::CreditFacility, CreditFacilityAction::Read)
        .await?;
    authz
        .add_permission_to_role(&role, Object::CreditFacility, CreditFacilityAction::List)
        .await?;
    authz
        .add_permission_to_role(&role, Object::CreditFacility, CreditFacilityAction::Approve)
        .await?;
    authz
        .add_permission_to_role(
            &role,
            Object::CreditFacility,
            CreditFacilityAction::InitiateDisbursement,
        )
        .await?;
    authz
        .add_permission_to_role(
            &role,
            Object::CreditFacility,
            CreditFacilityAction::ApproveDisbursement,
        )
        .await?;
    authz
        .add_permission_to_role(
            &role,
            Object::CreditFacility,
            CreditFacilityAction::ListDisbursement,
        )
        .await?;
    authz
        .add_permission_to_role(
            &role,
            Object::CreditFacility,
            CreditFacilityAction::UpdateCollateral,
        )
        .await?;
    authz
        .add_permission_to_role(
            &role,
            Object::CreditFacility,
            CreditFacilityAction::RecordPayment,
        )
        .await?;
    authz
        .add_permission_to_role(
            &role,
            Object::CreditFacility,
            CreditFacilityAction::Complete,
        )
        .await?;

    Ok(())
}

async fn add_permissions_for_accountant(authz: &Authorization) -> Result<(), AuthorizationError> {
    let role = Role::Accountant;

    authz
        .add_permission_to_role(&role, Object::Loan(LoanAllOrOne::All), LoanAction::Read)
        .await?;
    authz
        .add_permission_to_role(&role, Object::Loan(LoanAllOrOne::All), LoanAction::List)
        .await?;
    authz
        .add_permission_to_role(&role, Object::TermsTemplate, TermsTemplateAction::Read)
        .await?;
    authz
        .add_permission_to_role(
            &role,
            Object::Customer(CustomerAllOrOne::All),
            CustomerAction::List,
        )
        .await?;
    authz
        .add_permission_to_role(
            &role,
            Object::Customer(CustomerAllOrOne::All),
            CustomerAction::Read,
        )
        .await?;
    authz
        .add_permission_to_role(&role, Object::Deposit, DepositAction::Read)
        .await?;
    authz
        .add_permission_to_role(&role, Object::Deposit, DepositAction::List)
        .await?;
    authz
        .add_permission_to_role(&role, Object::Withdraw, WithdrawAction::Read)
        .await?;
    authz
        .add_permission_to_role(&role, Object::Withdraw, WithdrawAction::List)
        .await?;
    authz
        .add_permission_to_role(&role, Object::Document, DocumentAction::Read)
        .await?;
    authz
        .add_permission_to_role(&role, Object::Document, DocumentAction::List)
        .await?;
    authz
        .add_permission_to_role(
            &role,
            Object::Document,
            DocumentAction::GenerateDownloadLink,
        )
        .await?;

    Ok(())
}
