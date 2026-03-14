use std::collections::HashSet;
use std::ffi::OsString;

use anyhow::{Result, bail};

use crate::client::extract_operation_block;

const ACCOUNTING_DOC: &str = include_str!("graphql/accounting.graphql");
const APPROVAL_PROCESS_DOC: &str = include_str!("graphql/approval_process.graphql");
const AUDIT_DOC: &str = include_str!("graphql/audit.graphql");
const BUILD_INFO_DOC: &str = include_str!("graphql/build_info.graphql");
const COLLATERAL_DOC: &str = include_str!("graphql/collateral.graphql");
const CREDIT_FACILITY_DOC: &str = include_str!("graphql/credit_facility.graphql");
const CSV_EXPORT_DOC: &str = include_str!("graphql/csv_export.graphql");
const CUSTODIAN_DOC: &str = include_str!("graphql/custodian.graphql");
const CUSTOMER_DOC: &str = include_str!("graphql/customer.graphql");
const DASHBOARD_DOC: &str = include_str!("graphql/dashboard.graphql");
const DEPOSIT_ACCOUNT_DOC: &str = include_str!("graphql/deposit_account.graphql");
const DOCUMENT_DOC: &str = include_str!("graphql/document.graphql");
const DOMAIN_CONFIG_DOC: &str = include_str!("graphql/domain_config.graphql");
const FINANCIAL_STATEMENT_DOC: &str = include_str!("graphql/financial_statement.graphql");
const FISCAL_YEAR_DOC: &str = include_str!("graphql/fiscal_year.graphql");
const LIQUIDATION_DOC: &str = include_str!("graphql/liquidation.graphql");
const LOAN_AGREEMENT_DOC: &str = include_str!("graphql/loan_agreement.graphql");
const PROSPECT_DOC: &str = include_str!("graphql/prospect.graphql");
const REPORT_DOC: &str = include_str!("graphql/report.graphql");
const ROLE_DOC: &str = include_str!("graphql/role.graphql");
const TERMS_TEMPLATE_DOC: &str = include_str!("graphql/terms_template.graphql");
const USER_DOC: &str = include_str!("graphql/user.graphql");
const WITHDRAWAL_DOC: &str = include_str!("graphql/withdrawal.graphql");
const ADMIN_SCHEMA_DOC: &str =
    include_str!("../../../lana/admin-server/src/graphql/schema.graphql");

#[derive(Clone, Copy)]
struct QuerySpec {
    operation: &'static str,
    document: &'static str,
}

pub fn has_show_query_flag(args: &[OsString]) -> bool {
    args.iter().any(|arg| arg == "--show-query")
}

pub fn run_show_query(args: &[OsString]) -> Result<()> {
    if args.iter().any(|arg| arg == "--preview-graphql") {
        bail!("`--show-query` cannot be combined with `--preview-graphql`");
    }

    let path = extract_command_path(args);
    if path.is_empty() {
        bail!("No command specified. Example: `lana-admin --show-query credit facility get`");
    }

    let path_refs: Vec<&str> = path.iter().map(String::as_str).collect();

    if matches!(
        path_refs.as_slice(),
        ["auth", ..] | ["workflow", ..] | ["schema", ..]
    ) {
        println!("No GraphQL query/mutation: this command is a local CLI action.");
        return Ok(());
    }

    let Some(spec) = lookup_query(&path_refs) else {
        bail!(
            "No GraphQL operation mapping for command path: `{}`",
            path.join(" ")
        );
    };

    let query = extract_operation_block(spec.document, spec.operation);
    println!("command: {}", path.join(" "));
    println!("operation: {}", spec.operation);
    println!("query:\n{}", query);
    if let Some(input_type) = extract_input_type_name(&query) {
        println!("input_type: {}", input_type);
        if let Some(shape) = render_input_shape(ADMIN_SCHEMA_DOC, &input_type) {
            println!("input_shape:\n{}", shape);
        } else {
            println!("input_shape: <definition not found in local schema>");
        }
    }
    Ok(())
}

fn extract_command_path(args: &[OsString]) -> Vec<String> {
    let mut path = Vec::new();
    let mut started = false;

    for arg in args.iter().skip(1) {
        let s = arg.to_string_lossy();

        if s == "--show-query"
            || s == "--preview-graphql"
            || s == "--json"
            || s == "-v"
            || s == "-vv"
            || s == "-vvv"
        {
            continue;
        }

        if s.starts_with('-') {
            if started {
                break;
            }
            continue;
        }

        started = true;
        path.push(s.to_string());
    }

    path
}

fn lookup_query(path: &[&str]) -> Option<QuerySpec> {
    match path {
        ["prospect", "create"] => q("ProspectCreate", PROSPECT_DOC),
        ["prospect", "list"] => q("ProspectsList", PROSPECT_DOC),
        ["prospect", "get"] => q("ProspectGet", PROSPECT_DOC),
        ["prospect", "convert"] => q("ProspectConvert", PROSPECT_DOC),
        ["prospect", "close"] => q("ProspectClose", PROSPECT_DOC),
        ["prospect", "sumsub-link"] => q("ProspectKycLinkCreate", PROSPECT_DOC),

        ["customer", "list"] => q("CustomersList", CUSTOMER_DOC),
        ["customer", "get"] => q("CustomerGet", CUSTOMER_DOC),
        ["customer", "get-by-email"] => q("CustomerGetByEmail", CUSTOMER_DOC),
        ["customer", "close"] => q("CustomerClose", CUSTOMER_DOC),
        ["customer", "freeze"] => q("CustomerFreeze", CUSTOMER_DOC),
        ["customer", "unfreeze"] => q("CustomerUnfreeze", CUSTOMER_DOC),

        ["deposit", "account", "create"] => q("DepositAccountCreate", DEPOSIT_ACCOUNT_DOC),
        ["deposit", "account", "list"] => q("DepositAccountsList", DEPOSIT_ACCOUNT_DOC),
        ["deposit", "account", "get"] => q("DepositAccountGet", DEPOSIT_ACCOUNT_DOC),
        ["deposit", "account", "freeze"] => q("DepositAccountFreeze", DEPOSIT_ACCOUNT_DOC),
        ["deposit", "account", "unfreeze"] => q("DepositAccountUnfreeze", DEPOSIT_ACCOUNT_DOC),
        ["deposit", "account", "close"] => q("DepositAccountClose", DEPOSIT_ACCOUNT_DOC),
        ["deposit", "record"] => q("DepositRecord", DEPOSIT_ACCOUNT_DOC),
        ["deposit", "withdrawal", "initiate"] => q("WithdrawalInitiate", DEPOSIT_ACCOUNT_DOC),
        ["deposit", "withdrawal", "confirm"] => q("WithdrawalConfirm", DEPOSIT_ACCOUNT_DOC),
        ["deposit", "withdrawal", "cancel"] => q("WithdrawalCancel", DEPOSIT_ACCOUNT_DOC),
        ["deposit", "withdrawal", "revert"] => q("WithdrawalRevert", DEPOSIT_ACCOUNT_DOC),
        ["deposit", "withdrawal", "find"] => q("WithdrawalFind", WITHDRAWAL_DOC),

        ["credit", "terms-template", "create"] => q("TermsTemplateCreate", TERMS_TEMPLATE_DOC),
        ["credit", "terms-template", "list"] => q("TermsTemplatesList", TERMS_TEMPLATE_DOC),
        ["credit", "terms-template", "get"] => q("TermsTemplateGet", TERMS_TEMPLATE_DOC),
        ["credit", "terms-template", "update"] => q("TermsTemplateUpdate", TERMS_TEMPLATE_DOC),

        ["credit", "facility", "proposal-create"] => {
            q("CreditFacilityProposalCreate", CREDIT_FACILITY_DOC)
        }
        ["credit", "facility", "proposal-get"] => {
            q("CreditFacilityProposalGet", CREDIT_FACILITY_DOC)
        }
        ["credit", "facility", "proposal-conclude"] => q(
            "CreditFacilityProposalCustomerApprovalConclude",
            CREDIT_FACILITY_DOC,
        ),
        ["credit", "facility", "pending-get"] => q("PendingCreditFacilityGet", CREDIT_FACILITY_DOC),
        ["credit", "facility", "proposal-list"] => {
            q("CreditFacilityProposalsList", CREDIT_FACILITY_DOC)
        }
        ["credit", "facility", "list"] => q("CreditFacilitiesList", CREDIT_FACILITY_DOC),
        ["credit", "facility", "get"] => q("CreditFacilityGet", CREDIT_FACILITY_DOC),
        ["credit", "facility", "find"] => q("CreditFacilityFind", CREDIT_FACILITY_DOC),
        ["credit", "facility", "disbursal-initiate"] => {
            q("CreditFacilityDisbursalInitiate", CREDIT_FACILITY_DOC)
        }
        ["credit", "facility", "partial-payment-record"] => {
            q("CreditFacilityPartialPaymentRecord", CREDIT_FACILITY_DOC)
        }

        ["credit", "approval-process", "approve"] => {
            q("ApprovalProcessApprove", APPROVAL_PROCESS_DOC)
        }
        ["credit", "approval-process", "deny"] => q("ApprovalProcessDeny", APPROVAL_PROCESS_DOC),
        ["credit", "approval-process", "list"] => q("ApprovalProcessesList", APPROVAL_PROCESS_DOC),
        ["credit", "approval-process", "get"] => q("ApprovalProcessGet", APPROVAL_PROCESS_DOC),

        ["credit", "collateral", "update"] => q("CollateralUpdate", COLLATERAL_DOC),

        ["credit", "liquidation", "find"] => q("FindLiquidation", LIQUIDATION_DOC),
        ["credit", "liquidation", "record-collateral-sent"] => {
            q("LiquidationRecordCollateralSent", LIQUIDATION_DOC)
        }
        ["credit", "liquidation", "record-payment-received"] => {
            q("LiquidationRecordPaymentReceived", LIQUIDATION_DOC)
        }

        ["credit", "loan-agreement", "find"] => q("FindLoanAgreement", LOAN_AGREEMENT_DOC),
        ["credit", "loan-agreement", "generate"] => {
            q("CreditFacilityAgreementGenerate", LOAN_AGREEMENT_DOC)
        }
        ["credit", "loan-agreement", "download-link"] => {
            q("LoanAgreementDownloadLinkGenerate", LOAN_AGREEMENT_DOC)
        }

        ["dashboard", "get"] => q("DashboardGet", DASHBOARD_DOC),

        ["accounting", "chart-of-accounts"] => q("ChartOfAccountsGet", ACCOUNTING_DOC),
        ["accounting", "add-child-node"] => q("ChartOfAccountsAddChildNode", ACCOUNTING_DOC),
        ["accounting", "csv-import"] => q("ChartOfAccountsCsvImport", ACCOUNTING_DOC),
        ["accounting", "base-config"] => q("AccountingBaseConfig", ACCOUNTING_DOC),
        ["accounting", "credit-config"] => q("CreditConfigGet", ACCOUNTING_DOC),
        ["accounting", "deposit-config"] => q("DepositConfigGet", ACCOUNTING_DOC),
        ["accounting", "account-sets"] => q("DescendantAccountSetsByCategory", ACCOUNTING_DOC),
        ["accounting", "manual-transaction"] => q("ManualTransactionExecute", ACCOUNTING_DOC),
        ["accounting", "ledger-account"] => q("LedgerAccountByCode", ACCOUNTING_DOC),

        ["accounting", "fiscal-year", "list"] => q("FiscalYearsList", FISCAL_YEAR_DOC),
        ["accounting", "fiscal-year", "close-month"] => q("FiscalYearCloseMonth", FISCAL_YEAR_DOC),
        ["accounting", "fiscal-year", "close"] => q("FiscalYearClose", FISCAL_YEAR_DOC),

        ["accounting", "statement", "balance-sheet"] => {
            q("BalanceSheetGet", FINANCIAL_STATEMENT_DOC)
        }
        ["accounting", "statement", "trial-balance"] => {
            q("TrialBalanceGet", FINANCIAL_STATEMENT_DOC)
        }
        ["accounting", "statement", "profit-and-loss"] => {
            q("ProfitAndLossGet", FINANCIAL_STATEMENT_DOC)
        }

        ["accounting", "export", "account-entry"] => q("AccountEntryCsv", CSV_EXPORT_DOC),
        ["accounting", "export", "create-ledger-csv"] => {
            q("LedgerAccountCsvCreate", CSV_EXPORT_DOC)
        }
        ["accounting", "export", "download-link"] => {
            q("AccountingCsvDownloadLinkGenerate", CSV_EXPORT_DOC)
        }

        ["document", "attach"] => q("CustomerDocumentCreate", DOCUMENT_DOC),
        ["document", "get"] => q("CustomerDocumentGet", DOCUMENT_DOC),
        ["document", "list"] => q("CustomerDocumentsList", DOCUMENT_DOC),
        ["document", "download-link"] => q("CustomerDocumentDownloadLinkGenerate", DOCUMENT_DOC),
        ["document", "archive"] => q("CustomerDocumentArchive", DOCUMENT_DOC),
        ["document", "delete"] => q("CustomerDocumentDelete", DOCUMENT_DOC),

        ["audit", "list"] => q("AuditLogsList", AUDIT_DOC),
        ["audit", "customer"] => q("CustomerAuditLog", AUDIT_DOC),

        ["report", "find"] => q("FindReportRun", REPORT_DOC),
        ["report", "list"] => q("ReportRunsList", REPORT_DOC),
        ["report", "download-link"] => q("ReportFileDownloadLinkGenerate", REPORT_DOC),
        ["report", "trigger"] => q("TriggerReportRun", REPORT_DOC),

        ["iam", "user", "create"] => q("UserCreate", USER_DOC),
        ["iam", "user", "update-role"] => q("UserUpdateRole", USER_DOC),
        ["iam", "role", "list"] => q("RolesList", ROLE_DOC),
        ["iam", "role", "get"] => q("RoleGet", ROLE_DOC),
        ["iam", "role", "add-permission-sets"] => q("RoleAddPermissionSets", ROLE_DOC),
        ["iam", "role", "remove-permission-sets"] => q("RoleRemovePermissionSets", ROLE_DOC),

        ["system", "domain-config", "list"] => q("DomainConfigsList", DOMAIN_CONFIG_DOC),
        ["system", "domain-config", "update"] => q("DomainConfigUpdate", DOMAIN_CONFIG_DOC),
        ["system", "custodian", "create"] => q("CustodianCreate", CUSTODIAN_DOC),
        ["system", "custodian", "config-update"] => q("CustodianConfigUpdate", CUSTODIAN_DOC),

        ["version"] => q("BuildInfoGet", BUILD_INFO_DOC),

        _ => None,
    }
}

fn q(operation: &'static str, document: &'static str) -> Option<QuerySpec> {
    Some(QuerySpec {
        operation,
        document,
    })
}

fn extract_input_type_name(operation_block: &str) -> Option<String> {
    let header = operation_block
        .split_once('{')
        .map(|(head, _)| head)
        .unwrap_or(operation_block);
    let open = header.find('(')?;
    let close = find_matching(header, open, '(', ')')?;
    let vars = &header[open + 1..close];

    for variable in vars.split(',') {
        let variable = variable.trim();
        if variable.is_empty() {
            continue;
        }
        let (name, ty) = variable.split_once(':')?;
        if name.trim() != "$input" {
            continue;
        }
        let type_expr = ty.split_once('=').map(|(lhs, _)| lhs).unwrap_or(ty).trim();
        let base = base_graphql_type_name(type_expr);
        if !base.is_empty() {
            return Some(base.to_string());
        }
    }
    None
}

fn render_input_shape(schema: &str, root_input_type: &str) -> Option<String> {
    let mut out = Vec::new();
    let mut queue = vec![root_input_type.to_string()];
    let mut seen = HashSet::new();

    while let Some(input_type) = queue.pop() {
        if !seen.insert(input_type.clone()) {
            continue;
        }
        let Some(block) = extract_input_block(schema, &input_type) else {
            continue;
        };
        for nested in extract_nested_input_types(schema, &block) {
            if !seen.contains(&nested) {
                queue.push(nested);
            }
        }
        out.push(block);
    }

    if out.is_empty() {
        None
    } else {
        Some(out.join("\n\n"))
    }
}

fn extract_input_block(schema: &str, input_type: &str) -> Option<String> {
    let needle = format!("input {input_type}");
    for (idx, _) in schema.match_indices(&needle) {
        let after_name = idx + needle.len();
        if !schema
            .get(after_name..after_name + 1)
            .is_none_or(|c| matches!(c, " " | "\t" | "\r" | "\n" | "@" | "{"))
        {
            continue;
        }

        let block_start = schema[idx..].find('{').map(|pos| idx + pos)?;
        let block_end = find_matching(schema, block_start, '{', '}')?;
        return Some(schema[idx..=block_end].trim().to_string());
    }
    None
}

fn extract_nested_input_types(schema: &str, input_block: &str) -> Vec<String> {
    let (Some(open), Some(close)) = (input_block.find('{'), input_block.rfind('}')) else {
        return Vec::new();
    };
    let body = &input_block[open + 1..close];

    let mut nested = Vec::new();
    for line in body.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || !line.contains(':') {
            continue;
        }
        let Some((_, type_expr)) = line.split_once(':') else {
            continue;
        };
        let type_expr = type_expr
            .split_once('=')
            .map(|(lhs, _)| lhs)
            .unwrap_or(type_expr)
            .trim();
        let base = base_graphql_type_name(type_expr);
        if base.is_empty() {
            continue;
        }
        if extract_input_block(schema, base).is_some() {
            nested.push(base.to_string());
        }
    }
    nested
}

fn base_graphql_type_name(type_expr: &str) -> &str {
    let mut ty = type_expr.trim();
    while ty.ends_with('!') {
        ty = ty[..ty.len() - 1].trim_end();
    }
    while ty.starts_with('[') && ty.ends_with(']') {
        ty = ty[1..ty.len() - 1].trim();
        while ty.ends_with('!') {
            ty = ty[..ty.len() - 1].trim_end();
        }
    }
    ty
}

fn find_matching(text: &str, start_index: usize, open: char, close: char) -> Option<usize> {
    let mut depth = 0_i32;
    for (idx, ch) in text
        .char_indices()
        .skip_while(|(idx, _)| *idx < start_index)
    {
        if ch == open {
            depth += 1;
        } else if ch == close {
            depth -= 1;
            if depth == 0 {
                return Some(idx);
            }
        }
    }
    None
}
