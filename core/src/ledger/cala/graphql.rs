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
    query_path = "src/ledger/cala/graphql/user.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct UserBalance;

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
    query_path = "src/ledger/cala/graphql/journals.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct CoreJournalCreate;

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
    query_path = "src/ledger/cala/graphql/transactions/pledge-unallocated-collateral.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct PledgeUnallocatedCollateralTemplateCreate;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ledger/cala/graphql/schema.graphql",
    query_path = "src/ledger/cala/graphql/transactions/pledge-unallocated-collateral.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct PostPledgeUnallocatedCollateralTransaction;

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
    query_path = "src/ledger/cala/graphql/transactions/withdrawal.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct InitiateWithdrawalFromCheckingTxTemplateCreate;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ledger/cala/graphql/schema.graphql",
    query_path = "src/ledger/cala/graphql/transactions/withdrawal.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct PostInitiateWithdrawalFromCheckingTransaction;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ledger/cala/graphql/schema.graphql",
    query_path = "src/ledger/cala/graphql/transactions/withdrawal.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct SettleWithdrawalFromCheckingTxTemplateCreate;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ledger/cala/graphql/schema.graphql",
    query_path = "src/ledger/cala/graphql/transactions/withdrawal.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct PostSettleWithdrawalFromCheckingTransaction;

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
    query_path = "src/ledger/cala/graphql/transactions/complete-loan.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct CompleteLoanTemplateCreate;

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
    query_path = "src/ledger/cala/graphql/fixed-term-loan.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct FixedTermLoanBalance;

type UUID = uuid::Uuid;
type JSON = serde_json::Value;
type Decimal = rust_decimal::Decimal;
type CurrencyCode = cala_types::primitives::Currency;
type Expression = String;
