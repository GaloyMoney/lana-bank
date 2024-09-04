#![allow(clippy::enum_variant_names)]
#![allow(clippy::derive_partial_eq_without_eq)]
#![allow(clippy::upper_case_acronyms)]
use graphql_client::GraphQLQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ledger/cala/graphql/schema.graphql",
    query_path = "src/ledger/cala/graphql/accounts.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct AccountByCode;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ledger/cala/graphql/schema.graphql",
    query_path = "src/ledger/cala/graphql/accounts.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct AccountByExternalId;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ledger/cala/graphql/schema.graphql",
    query_path = "src/ledger/cala/graphql/accounts.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct AccountById;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ledger/cala/graphql/schema.graphql",
    query_path = "src/ledger/cala/graphql/accounts.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct AccountCreate;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ledger/cala/graphql/schema.graphql",
    query_path = "src/ledger/cala/graphql/account-set.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct AccountSetCreate;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ledger/cala/graphql/schema.graphql",
    query_path = "src/ledger/cala/graphql/account-set-with-balance.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct AccountSetAndSubAccountsWithBalance;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ledger/cala/graphql/schema.graphql",
    query_path = "src/ledger/cala/graphql/account-set.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct AddToAccountSet;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ledger/cala/graphql/schema.graphql",
    query_path = "src/ledger/cala/graphql/customer.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct CustomerBalance;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ledger/cala/graphql/schema.graphql",
    query_path = "src/ledger/cala/graphql/customer_accounts.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct CreateCustomerAccounts;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ledger/cala/graphql/schema.graphql",
    query_path = "src/ledger/cala/graphql/loan_accounts.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct CreateLoanAccounts;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ledger/cala/graphql/schema.graphql",
    query_path = "src/ledger/cala/graphql/journals.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct JournalById;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ledger/cala/graphql/schema.graphql",
    query_path = "src/ledger/cala/graphql/transactions/find-tx-template.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct TxTemplateByCode;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ledger/cala/graphql/schema.graphql",
    query_path = "src/ledger/cala/graphql/transactions/add-equity.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct AddEquityTemplateCreate;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ledger/cala/graphql/schema.graphql",
    query_path = "src/ledger/cala/graphql/transactions/add-equity.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct PostAddEquityTransaction;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ledger/cala/graphql/schema.graphql",
    query_path = "src/ledger/cala/graphql/transactions/deposit-checking.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct DepositCheckingTemplateCreate;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ledger/cala/graphql/schema.graphql",
    query_path = "src/ledger/cala/graphql/transactions/deposit-checking.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct PostDepositCheckingTransaction;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ledger/cala/graphql/schema.graphql",
    query_path = "src/ledger/cala/graphql/transactions/initiate-withdraw.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct InitiateWithdrawTemplateCreate;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ledger/cala/graphql/schema.graphql",
    query_path = "src/ledger/cala/graphql/transactions/initiate-withdraw.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct PostInitiateWithdrawTransaction;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ledger/cala/graphql/schema.graphql",
    query_path = "src/ledger/cala/graphql/transactions/confirm-withdraw.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct ConfirmWithdrawTemplateCreate;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ledger/cala/graphql/schema.graphql",
    query_path = "src/ledger/cala/graphql/transactions/confirm-withdraw.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct PostConfirmWithdrawTransaction;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ledger/cala/graphql/schema.graphql",
    query_path = "src/ledger/cala/graphql/transactions/cancel-withdraw.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct CancelWithdrawTemplateCreate;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ledger/cala/graphql/schema.graphql",
    query_path = "src/ledger/cala/graphql/transactions/cancel-withdraw.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct PostCancelWithdrawTransaction;
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ledger/cala/graphql/schema.graphql",
    query_path = "src/ledger/cala/graphql/transactions/approve-loan.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct ApproveLoanTemplateCreate;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ledger/cala/graphql/schema.graphql",
    query_path = "src/ledger/cala/graphql/transactions/approve-loan.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct PostApproveLoanTransaction;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ledger/cala/graphql/schema.graphql",
    query_path = "src/ledger/cala/graphql/transactions/adjust-collateral.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct AddCollateralTemplateCreate;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ledger/cala/graphql/schema.graphql",
    query_path = "src/ledger/cala/graphql/transactions/adjust-collateral.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct PostAddCollateralTransaction;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ledger/cala/graphql/schema.graphql",
    query_path = "src/ledger/cala/graphql/transactions/adjust-collateral.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct RemoveCollateralTemplateCreate;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ledger/cala/graphql/schema.graphql",
    query_path = "src/ledger/cala/graphql/transactions/adjust-collateral.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct PostRemoveCollateralTransaction;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ledger/cala/graphql/schema.graphql",
    query_path = "src/ledger/cala/graphql/transactions/complete-loan.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct PostCompleteLoanTransaction;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ledger/cala/graphql/schema.graphql",
    query_path = "src/ledger/cala/graphql/transactions/incur-interest.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct IncurInterestTemplateCreate;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ledger/cala/graphql/schema.graphql",
    query_path = "src/ledger/cala/graphql/transactions/incur-interest.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct PostIncurInterestTransaction;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ledger/cala/graphql/schema.graphql",
    query_path = "src/ledger/cala/graphql/transactions/record-payment.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct RecordPaymentTemplateCreate;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ledger/cala/graphql/schema.graphql",
    query_path = "src/ledger/cala/graphql/transactions/record-payment.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct PostRecordPaymentTransaction;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ledger/cala/graphql/schema.graphql",
    query_path = "src/ledger/cala/graphql/loan.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct LoanBalance;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ledger/cala/graphql/schema.graphql",
    query_path = "src/ledger/cala/graphql/trial-balance.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct TrialBalance;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ledger/cala/graphql/schema.graphql",
    query_path = "src/ledger/cala/graphql/chart-of-accounts.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct ChartOfAccounts;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ledger/cala/graphql/schema.graphql",
    query_path = "src/ledger/cala/graphql/balance-sheet.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct BalanceSheet;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ledger/cala/graphql/schema.graphql",
    query_path = "src/ledger/cala/graphql/profit-and-loss.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct ProfitAndLossStatement;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ledger/cala/graphql/schema.graphql",
    query_path = "src/ledger/cala/graphql/cash-flow.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct CashFlowStatement;

type UUID = uuid::Uuid;
type JSON = serde_json::Value;
type Decimal = rust_decimal::Decimal;
type CurrencyCode = cala_types::primitives::Currency;
type Expression = String;
type Timestamp = chrono::DateTime<chrono::Utc>;
