use authz::error::AuthorizationError;
use chart_of_accounts::{CoreChartOfAccountsAction, CoreChartOfAccountsObject};
use core_user::{CoreUserAction, UserEntityAction, UserObject};
use dashboard::{DashboardModuleAction, DashboardModuleObject};
use deposit::{CoreDepositAction, CoreDepositObject};
use governance::{GovernanceAction, GovernanceObject};

use super::*;
use rbac_types::LanaRole;

pub(super) async fn execute(authz: &Authorization) -> Result<(), AuthorizationError> {
    seed_roles(authz).await?;
    seed_role_hierarchy(authz).await?;
    Ok(())
}

async fn seed_role_hierarchy(authz: &Authorization) -> Result<(), AuthorizationError> {
    authz
        .add_role_hierarchy(LanaRole::ADMIN, LanaRole::SUPERUSER)
        .await?;
    authz
        .add_role_hierarchy(LanaRole::BANK_MANAGER, LanaRole::ADMIN)
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
    let role = LanaRole::SUPERUSER;

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
    let role = LanaRole::ADMIN;

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
            GovernanceAction::COMMITTEE_ADD_MEMBER,
        )
        .await?;
    authz
        .add_permission_to_role(
            &role,
            GovernanceObject::all_committees(),
            GovernanceAction::COMMITTEE_REMOVE_MEMBER,
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
    let role = LanaRole::BANK_MANAGER;

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
            CreditFacilityAction::InitiateDisbursal,
        )
        .await?;
    authz
        .add_permission_to_role(
            &role,
            Object::CreditFacility,
            CreditFacilityAction::SettleDisbursal,
        )
        .await?;
    authz
        .add_permission_to_role(
            &role,
            Object::CreditFacility,
            CreditFacilityAction::ListDisbursals,
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
    authz
        .add_permission_to_role(&role, Object::TrialBalance, TrialBalanceAction::Read)
        .await?;
    authz
        .add_permission_to_role(&role, Object::TrialBalance, TrialBalanceAction::Create)
        .await?;
    authz
        .add_permission_to_role(&role, Object::TrialBalance, TrialBalanceAction::Update)
        .await?;
    authz
        .add_permission_to_role(
            &role,
            DashboardModuleObject::Dashboard,
            DashboardModuleAction::DASHBOARD_READ,
        )
        .await?;

    authz
        .add_permission_to_role(
            &role,
            CoreChartOfAccountsObject::all_charts(),
            CoreChartOfAccountsAction::CHART_LIST,
        )
        .await?;

    authz
        .add_permission_to_role(
            &role,
            CoreDepositObject::all_deposits(),
            CoreDepositAction::DEPOSIT_CREATE,
        )
        .await?;
    authz
        .add_permission_to_role(
            &role,
            CoreDepositObject::all_deposits(),
            CoreDepositAction::DEPOSIT_READ,
        )
        .await?;
    authz
        .add_permission_to_role(
            &role,
            CoreDepositObject::all_deposits(),
            CoreDepositAction::DEPOSIT_LIST,
        )
        .await?;
    authz
        .add_permission_to_role(
            &role,
            CoreDepositObject::all_deposit_accounts(),
            CoreDepositAction::DEPOSIT_ACCOUNT_CREATE,
        )
        .await?;
    authz
        .add_permission_to_role(
            &role,
            CoreDepositObject::all_deposit_accounts(),
            CoreDepositAction::DEPOSIT_ACCOUNT_READ,
        )
        .await?;
    authz
        .add_permission_to_role(
            &role,
            CoreDepositObject::all_deposit_accounts(),
            CoreDepositAction::DEPOSIT_ACCOUNT_LIST,
        )
        .await?;
    authz
        .add_permission_to_role(
            &role,
            CoreDepositObject::all_deposit_accounts(),
            CoreDepositAction::DEPOSIT_ACCOUNT_READ_BALANCE,
        )
        .await?;
    authz
        .add_permission_to_role(
            &role,
            CoreDepositObject::all_withdrawals(),
            CoreDepositAction::WITHDRAWAL_INITIATE,
        )
        .await?;
    authz
        .add_permission_to_role(
            &role,
            CoreDepositObject::all_withdrawals(),
            CoreDepositAction::WITHDRAWAL_CANCEL,
        )
        .await?;
    authz
        .add_permission_to_role(
            &role,
            CoreDepositObject::all_withdrawals(),
            CoreDepositAction::WITHDRAWAL_CONFIRM,
        )
        .await?;
    authz
        .add_permission_to_role(
            &role,
            CoreDepositObject::all_withdrawals(),
            CoreDepositAction::WITHDRAWAL_READ,
        )
        .await?;
    authz
        .add_permission_to_role(
            &role,
            CoreDepositObject::all_withdrawals(),
            CoreDepositAction::WITHDRAWAL_LIST,
        )
        .await?;

    Ok(())
}

async fn add_permissions_for_accountant(authz: &Authorization) -> Result<(), AuthorizationError> {
    let role = LanaRole::ACCOUNTANT;

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
        .add_permission_to_role(
            &role,
            CoreDepositObject::all_deposits(),
            CoreDepositAction::DEPOSIT_READ,
        )
        .await?;
    authz
        .add_permission_to_role(
            &role,
            CoreDepositObject::all_deposits(),
            CoreDepositAction::DEPOSIT_LIST,
        )
        .await?;
    authz
        .add_permission_to_role(
            &role,
            CoreDepositObject::all_withdrawals(),
            CoreDepositAction::WITHDRAWAL_READ,
        )
        .await?;
    authz
        .add_permission_to_role(
            &role,
            CoreDepositObject::all_withdrawals(),
            CoreDepositAction::WITHDRAWAL_LIST,
        )
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
