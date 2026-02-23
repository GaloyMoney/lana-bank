use graphql_client::GraphQLQuery;

// Custom scalar type aliases for CLI presentation layer.
// Using serde_json::Value to handle scalars that serialize as either
// strings or numbers depending on the server implementation.
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
type SignedSatoshis = serde_json::Value;
type SignedUsdCents = serde_json::Value;
type Decimal = serde_json::Value;
type AuditEntryId = String;
type AuditSubjectId = String;
type Json = serde_json::Value;
type Upload = String;

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
