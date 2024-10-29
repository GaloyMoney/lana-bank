use authz::error::AuthorizationError;
use core_user::{CoreUserAction, UserEntityAction, UserObject};
use governance::{GovernanceAction, GovernanceObject};

use super::*;
use rbac_types::LavaRole;

pub(super) async fn execute(authz: &Authorization) -> Result<(), AuthorizationError> {
    seed_roles(authz).await?;
    seed_role_hierarchy(authz).await?;
    Ok(())
}

async fn seed_role_hierarchy(authz: &Authorization) -> Result<(), AuthorizationError> {
    authz
        .add_role_hierarchy(LavaRole::ADMIN, LavaRole::SUPERUSER)
        .await?;
    authz
        .add_role_hierarchy(LavaRole::BANK_MANAGER, LavaRole::ADMIN)
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
    let role = LavaRole::SUPERUSER;

    authz
        .add_permission_to_role(
            &role,
            UserObject::all_users(),
            CoreUserAction::USER_ASSIGN_ROLE,
        )
        .await?;
    authz
        .add_permission_to_role(
            &role,
            UserObject::all_users(),
            CoreUserAction::USER_REVOKE_ROLE,
        )
        .await?;
    Ok(())
}

async fn add_permissions_for_admin(authz: &Authorization) -> Result<(), AuthorizationError> {
    let role = LavaRole::ADMIN;

    authz
        .add_permission_to_role(
            &role,
            UserObject::all_users(),
            CoreUserAction::User(UserEntityAction::Create),
        )
        .await?;
    authz
        .add_permission_to_role(
            &role,
            UserObject::all_users(),
            CoreUserAction::User(UserEntityAction::List),
        )
        .await?;
    authz
        .add_permission_to_role(
            &role,
            UserObject::all_users(),
            CoreUserAction::User(UserEntityAction::Read),
        )
        .await?;
    authz
        .add_permission_to_role(
            &role,
            UserObject::all_users(),
            CoreUserAction::User(UserEntityAction::Update),
        )
        .await?;
    authz
        .add_permission_to_role(
            &role,
            UserObject::all_users(),
            CoreUserAction::User(UserEntityAction::AssignRole),
        )
        .await?;
    authz
        .add_permission_to_role(
            &role,
            UserObject::all_users(),
            CoreUserAction::User(UserEntityAction::RevokeRole),
        )
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
    authz
        .add_permission_to_role(
            &role,
            GovernanceObject::all_committees(),
            GovernanceAction::COMMITTEE_CREATE,
        )
        .await?;
    authz
        .add_permission_to_role(
            &role,
            GovernanceObject::all_committees(),
            GovernanceAction::COMMITTEE_LIST,
        )
        .await?;
    authz
        .add_permission_to_role(
            &role,
            GovernanceObject::all_committees(),
            GovernanceAction::COMMITTEE_READ,
        )
        .await?;
    authz
        .add_permission_to_role(
            &role,
            GovernanceObject::all_committees(),
            GovernanceAction::COMMITTEE_ADD_USER,
        )
        .await?;
    authz
        .add_permission_to_role(
            &role,
            GovernanceObject::all_committees(),
            GovernanceAction::COMMITTEE_REMOVE_USER,
        )
        .await?;

    authz
        .add_permission_to_role(
            &role,
            GovernanceObject::all_policies(),
            GovernanceAction::POLICY_CREATE,
        )
        .await?;
    authz
        .add_permission_to_role(
            &role,
            GovernanceObject::all_policies(),
            GovernanceAction::POLICY_UPDATE_RULES,
        )
        .await?;
    authz
        .add_permission_to_role(
            &role,
            GovernanceObject::all_policies(),
            GovernanceAction::POLICY_READ,
        )
        .await?;
    authz
        .add_permission_to_role(
            &role,
            GovernanceObject::all_policies(),
            GovernanceAction::POLICY_LIST,
        )
        .await?;
    authz
        .add_permission_to_role(
            &role,
            GovernanceObject::all_approval_processes(),
            GovernanceAction::APPROVAL_PROCESS_CREATE,
        )
        .await?;
    authz
        .add_permission_to_role(
            &role,
            GovernanceObject::all_approval_processes(),
            GovernanceAction::APPROVAL_PROCESS_LIST,
        )
        .await?;
    authz
        .add_permission_to_role(
            &role,
            GovernanceObject::all_approval_processes(),
            GovernanceAction::APPROVAL_PROCESS_READ,
        )
        .await?;
    authz
        .add_permission_to_role(
            &role,
            GovernanceObject::all_approval_processes(),
            GovernanceAction::APPROVAL_PROCESS_APPROVE,
        )
        .await?;
    authz
        .add_permission_to_role(
            &role,
            GovernanceObject::all_approval_processes(),
            GovernanceAction::APPROVAL_PROCESS_DENY,
        )
        .await?;
    Ok(())
}

async fn add_permissions_for_bank_manager(authz: &Authorization) -> Result<(), AuthorizationError> {
    let role = LavaRole::BANK_MANAGER;

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
        .add_permission_to_role(
            &role,
            Object::CreditFacility,
            CreditFacilityAction::Activate,
        )
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
    let role = LavaRole::ACCOUNTANT;

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
