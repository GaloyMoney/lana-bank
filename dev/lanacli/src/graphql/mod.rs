use graphql_client::GraphQLQuery;

// Custom scalar type aliases for CLI presentation layer.
// Using serde_json::Value to handle scalars that serialize as either
// strings or numbers depending on the server implementation.
#[allow(clippy::upper_case_acronyms)]
type UUID = String;
type UsdCents = serde_json::Value;
type Satoshis = serde_json::Value;
type AnnualRatePct = serde_json::Value;
type CVLPctValue = serde_json::Value;
type OneTimeFeeRatePct = serde_json::Value;
type Timestamp = String;
type Date = String;
type PublicId = String;
type AccountCode = String;
type Json = serde_json::Value;
type Upload = String;
type SignedUsdCents = serde_json::Value;
type SignedSatoshis = serde_json::Value;
type Decimal = String;

// -- Prospect operations --

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/prospect.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct ProspectCreate;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/prospect.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct ProspectsList;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/prospect.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct ProspectGet;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/prospect.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct ProspectConvert;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/prospect.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct ProspectClose;

// -- Customer operations --

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/customer.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct CustomersList;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/customer.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct CustomerGet;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/customer.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct CustomerGetByEmail;

// -- Deposit Account operations --

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/deposit_account.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct DepositAccountCreate;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/deposit_account.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct DepositAccountsList;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/deposit_account.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct DepositAccountGet;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/deposit_account.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct DepositRecord;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/deposit_account.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct WithdrawalInitiate;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/deposit_account.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct WithdrawalConfirm;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/deposit_account.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct WithdrawalCancel;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/deposit_account.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct WithdrawalRevert;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/deposit_account.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct DepositAccountFreeze;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/deposit_account.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct DepositAccountUnfreeze;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/deposit_account.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct DepositAccountClose;

// -- Terms Template operations --

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/terms_template.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct TermsTemplateCreate;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/terms_template.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct TermsTemplatesList;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/terms_template.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct TermsTemplateGet;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/terms_template.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct TermsTemplateUpdate;

// -- Credit Facility operations --

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/credit_facility.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct CreditFacilityProposalCreate;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/credit_facility.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct CreditFacilityProposalsList;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/credit_facility.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct CreditFacilitiesList;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/credit_facility.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct CreditFacilityGet;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/credit_facility.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct CreditFacilityDisbursalInitiate;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/credit_facility.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct CreditFacilityProposalGet;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/credit_facility.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct CreditFacilityProposalCustomerApprovalConclude;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/credit_facility.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct PendingCreditFacilityGet;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/credit_facility.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct CreditFacilityFind;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/credit_facility.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct CreditFacilityPartialPaymentRecord;

// -- Collateral operations --

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/collateral.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct CollateralUpdate;

// -- Approval Process operations --

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/approval_process.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct ApprovalProcessApprove;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/approval_process.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct ApprovalProcessDeny;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/approval_process.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct ApprovalProcessesList;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/approval_process.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct ApprovalProcessGet;

// -- Liquidation operations --

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/liquidation.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct FindLiquidation;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/liquidation.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct LiquidationRecordCollateralSent;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/liquidation.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct LiquidationRecordPaymentReceived;

// -- Dashboard operations --

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/dashboard.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct DashboardGet;

// -- Accounting operations --

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/accounting.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct ChartOfAccountsGet;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/accounting.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct ChartOfAccountsAddRootNode;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/accounting.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct ChartOfAccountsAddChildNode;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/accounting.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct ChartOfAccountsCsvImport;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/accounting.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct AccountingBaseConfig;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/accounting.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct CreditConfigGet;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/accounting.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct DepositConfigGet;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/accounting.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct DescendantAccountSetsByCategory;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/accounting.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct ManualTransactionExecute;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/accounting.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct LedgerAccountByCode;

// -- Fiscal Year operations --

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/fiscal_year.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct FiscalYearsList;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/fiscal_year.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct FiscalYearCloseMonth;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/fiscal_year.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct FiscalYearClose;

// -- CSV Export operations --

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/csv_export.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct AccountEntryCsv;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/csv_export.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct LedgerAccountCsvCreate;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/csv_export.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct AccountingCsvDownloadLinkGenerate;

// -- Custodian operations --

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/custodian.graphql",
    response_derives = "Debug, Clone, Serialize",
    variables_derives = "Deserialize"
)]
pub struct CustodianCreate;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/custodian.graphql",
    response_derives = "Debug, Clone, Serialize",
    variables_derives = "Deserialize"
)]
pub struct CustodianConfigUpdate;

// -- Document operations --

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/document.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct CustomerDocumentGet;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/document.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct CustomerDocumentsList;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/document.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct CustomerDocumentDownloadLinkGenerate;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/document.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct CustomerDocumentArchive;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/document.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct CustomerDocumentDelete;

// -- Domain Config operations --

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/domain_config.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct DomainConfigsList;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/domain_config.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct DomainConfigUpdate;

// -- Audit operations --

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/audit.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct AuditLogsList;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/audit.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct CustomerAuditLog;

// -- Financial Statement operations --

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/financial_statement.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct BalanceSheetGet;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/financial_statement.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct TrialBalanceGet;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/financial_statement.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct ProfitAndLossGet;

// -- Sumsub operations --

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/sumsub.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct SumsubPermalinkCreate;

// -- Loan Agreement operations --

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/loan_agreement.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct FindLoanAgreement;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/loan_agreement.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct LoanAgreementGenerate;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/loan_agreement.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct LoanAgreementDownloadLinkGenerate;

// -- User operations --

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/user.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct RolesList;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/user.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct UserCreate;

// -- Report operations --

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/report.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct FindReportRun;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/report.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct ReportRunsList;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/report.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct ReportFileDownloadLinkGenerate;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/report.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct TriggerReportRun;

// -- Withdrawal operations --

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../lana/admin-server/src/graphql/schema.graphql",
    query_path = "src/graphql/withdrawal.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct WithdrawalFind;
