pub mod error;
mod graphql;

use graphql_client::{GraphQLQuery, Response};
use reqwest::Client as ReqwestClient;

use super::{ExportEntityEventData, ExportPriceData, ExportSumsubApplicantData};

use error::*;
use graphql::*;

#[derive(Clone)]
pub struct CalaClient {
    url: String,
    client: ReqwestClient,
}

impl CalaClient {
    pub fn new(url: String) -> Self {
        let client = ReqwestClient::new();
        CalaClient { client, url }
    }

    pub async fn export_price_data(
        &self,
        table_name: &str,
        data: ExportPriceData,
    ) -> Result<(), CalaError> {
        let insert_id = uuid::Uuid::new_v4().to_string();
        tracing::Span::current().record("insert_id", &insert_id);
        let variables = row_insert::Variables {
            insert_id,
            table_name: table_name.to_string(),
            row_data: serde_json::to_value(data).expect("Could not serialize price data"),
        };
        Self::traced_gql_request::<RowInsert, _>(&self.client, &self.url, variables).await?;
        Ok(())
    }

    pub async fn export_applicant_data(
        &self,
        table_name: &str,
        data: ExportSumsubApplicantData,
    ) -> Result<(), CalaError> {
        let insert_id = uuid::Uuid::new_v4().to_string();
        tracing::Span::current().record("insert_id", &insert_id);
        let variables = row_insert::Variables {
            insert_id,
            table_name: table_name.to_string(),
            row_data: serde_json::to_value(data).expect("Could not serialize event"),
        };
        Self::traced_gql_request::<RowInsert, _>(&self.client, &self.url, variables).await?;
        Ok(())
    }

    pub async fn export_entity_event_to_bq(
        &self,
        table_name: &str,
        data: &ExportEntityEventData,
    ) -> Result<(), CalaError> {
        let insert_id = format!("{}:{}", data.id, data.sequence);
        tracing::Span::current().record("insert_id", &insert_id);
        let variables = row_insert::Variables {
            insert_id: format!("{}:{}", data.id, data.sequence),
            table_name: table_name.to_string(),
            row_data: serde_json::to_value(data).expect("Could not serialize event"),
        };
        Self::traced_gql_request::<RowInsert, _>(&self.client, &self.url, variables).await?;
        Ok(())
    }

    async fn traced_gql_request<Q: GraphQLQuery, U: reqwest::IntoUrl>(
        client: &ReqwestClient,
        url: U,
        variables: Q::Variables,
    ) -> Result<Response<Q::ResponseData>, CalaError>
    where
        <Q as GraphQLQuery>::ResponseData: std::fmt::Debug,
    {
        let trace_headers = tracing_utils::http::inject_trace();
        let body = Q::build_query(variables);
        let response = client
            .post(url)
            .headers(trace_headers)
            .json(&body)
            .send()
            .await?;
        let response = response.json::<Response<Q::ResponseData>>().await?;

        if let Some(errors) = response.errors {
            return Err(CalaError::from(errors));
        }

        Ok(response)
    }
}
