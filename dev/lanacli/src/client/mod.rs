pub mod auth;

use anyhow::{Context, bail};
use graphql_client::GraphQLQuery;
use serde::Deserialize;

use self::auth::AuthClient;

#[derive(Deserialize)]
struct GraphQLResponse<D> {
    data: Option<D>,
    errors: Option<Vec<GraphQLError>>,
}

#[derive(Deserialize)]
struct GraphQLError {
    message: String,
}

pub struct GraphQLClient {
    connect_url: String,
    host_header: String,
    http: reqwest::Client,
    auth: AuthClient,
}

impl GraphQLClient {
    pub fn new(admin_url: String, auth: AuthClient) -> Self {
        // Parse the admin URL to extract the Host header and rewrite the URL
        // to connect via 127.0.0.1. This handles the common case where
        // admin.localhost doesn't resolve in DNS but Oathkeeper routes by Host header.
        let (connect_url, host_header) = rewrite_url(&admin_url);
        Self {
            connect_url,
            host_header,
            http: reqwest::Client::new(),
            auth,
        }
    }

    pub async fn execute<Q: GraphQLQuery>(
        &mut self,
        variables: Q::Variables,
    ) -> anyhow::Result<Q::ResponseData>
    where
        Q::ResponseData: for<'de> Deserialize<'de>,
    {
        let body = Q::build_query(variables);
        let token = self.auth.get_token().await?;

        let resp = self
            .http
            .post(&self.connect_url)
            .header("Host", &self.host_header)
            .header("Authorization", format!("Bearer {token}"))
            .json(&body)
            .send()
            .await
            .context("Failed to reach admin server")?;

        let status = resp.status();
        let text = resp.text().await.context("Failed to read response body")?;

        if !status.is_success() {
            bail!("HTTP {}: {}", status, text);
        }

        let gql_resp: GraphQLResponse<Q::ResponseData> =
            serde_json::from_str(&text).context("Failed to parse GraphQL response")?;

        if let Some(errors) = gql_resp.errors {
            let messages: Vec<_> = errors.iter().map(|e| e.message.as_str()).collect();
            bail!("GraphQL errors: {}", messages.join("; "));
        }

        gql_resp.data.context("No data in GraphQL response")
    }
}

/// Rewrites a URL like `http://admin.localhost:4455/graphql` to connect via
/// `http://127.0.0.1:4455/graphql` while preserving the original host:port
/// as a Host header value for Oathkeeper virtual host routing.
fn rewrite_url(url: &str) -> (String, String) {
    if let Ok(parsed) = url::Url::parse(url) {
        let host = parsed.host_str().unwrap_or("localhost");
        let port = parsed.port_or_known_default().unwrap_or(80);
        let host_header = format!("{host}:{port}");

        // Only rewrite if host is not already 127.0.0.1 or localhost
        if host != "127.0.0.1" && host != "localhost" {
            let connect_url = format!("{}://127.0.0.1:{}{}", parsed.scheme(), port, parsed.path());
            (connect_url, host_header)
        } else {
            (url.to_string(), host_header)
        }
    } else {
        (url.to_string(), String::new())
    }
}
