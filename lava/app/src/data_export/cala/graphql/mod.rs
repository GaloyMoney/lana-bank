#![allow(clippy::enum_variant_names)]
#![allow(clippy::derive_partial_eq_without_eq)]
#![allow(clippy::upper_case_acronyms)]
use graphql_client::GraphQLQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ledger/cala/graphql/schema.graphql",
    query_path = "src/data_export/cala/graphql/row-insert.gql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct RowInsert;

type UUID = uuid::Uuid;
type JSON = serde_json::Value;
