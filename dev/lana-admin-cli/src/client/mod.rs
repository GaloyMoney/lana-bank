pub mod auth;

use anyhow::{Context, bail};
use graphql_client::GraphQLQuery;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::json;
use std::path::Path;

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

#[derive(Debug, Clone)]
pub struct MultipartUpload {
    pub file_path: String,
    pub variable_path: String,
}

impl MultipartUpload {
    pub fn new(file_path: impl Into<String>, variable_path: impl Into<String>) -> Self {
        Self {
            file_path: file_path.into(),
            variable_path: variable_path.into(),
        }
    }
}

pub struct GraphQLClient {
    connect_url: String,
    host_header: Option<String>,
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
        Q::ResponseData: DeserializeOwned,
    {
        let body = Q::build_query(variables);
        let token = self.auth.get_token().await?;

        let mut req = self
            .http
            .post(&self.connect_url)
            .header("Authorization", format!("Bearer {token}"))
            .json(&body);
        if let Some(ref host) = self.host_header {
            req = req.header("Host", host);
        }

        let resp = req.send().await.context("Failed to reach admin server")?;

        Self::parse_response(resp).await
    }

    pub async fn execute_multipart<Q: GraphQLQuery>(
        &mut self,
        variables: Q::Variables,
        uploads: Vec<MultipartUpload>,
    ) -> anyhow::Result<Q::ResponseData>
    where
        Q::ResponseData: DeserializeOwned,
    {
        let mut body = serde_json::to_value(Q::build_query(variables))
            .context("Failed to serialize GraphQL request body")?;

        let mut map = serde_json::Map::new();
        for (idx, upload) in uploads.iter().enumerate() {
            let variable_path = format!("variables.{}", upload.variable_path);
            set_path_to_null(&mut body, &variable_path)?;
            map.insert(idx.to_string(), json!([variable_path]));
        }

        let mut form = reqwest::multipart::Form::new()
            .text("operations", body.to_string())
            .text("map", serde_json::Value::Object(map).to_string());

        for (idx, upload) in uploads.iter().enumerate() {
            let file_bytes = std::fs::read(&upload.file_path)
                .with_context(|| format!("Failed to read file '{}'", upload.file_path))?;
            let filename = Path::new(&upload.file_path)
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or("upload.bin")
                .to_string();
            let part = reqwest::multipart::Part::bytes(file_bytes).file_name(filename);
            form = form.part(idx.to_string(), part);
        }

        let token = self.auth.get_token().await?;

        let mut req = self
            .http
            .post(&self.connect_url)
            .header("Authorization", format!("Bearer {token}"))
            .multipart(form);
        if let Some(ref host) = self.host_header {
            req = req.header("Host", host);
        }

        let resp = req.send().await.context("Failed to reach admin server")?;

        Self::parse_response(resp).await
    }

    async fn parse_response<R: DeserializeOwned>(resp: reqwest::Response) -> anyhow::Result<R> {
        let status = resp.status();
        let text = resp.text().await.context("Failed to read response body")?;

        if !status.is_success() {
            bail!("HTTP {}: {}", status, text);
        }

        let gql_resp: GraphQLResponse<R> =
            serde_json::from_str(&text).context("Failed to parse GraphQL response")?;

        if let Some(errors) = gql_resp.errors {
            let messages: Vec<_> = errors.iter().map(|e| e.message.as_str()).collect();
            bail!("GraphQL errors: {}", messages.join("; "));
        }

        gql_resp.data.context("No data in GraphQL response")
    }
}

fn set_path_to_null(value: &mut serde_json::Value, path: &str) -> anyhow::Result<()> {
    if path.is_empty() {
        bail!("Upload variable path cannot be empty");
    }

    let mut current = value;
    let mut segments = path.split('.').peekable();

    while let Some(segment) = segments.next() {
        if segments.peek().is_none() {
            let object = current
                .as_object_mut()
                .with_context(|| format!("Path segment '{segment}' is not an object"))?;
            object.insert(segment.to_string(), serde_json::Value::Null);
            return Ok(());
        }

        current = current
            .get_mut(segment)
            .with_context(|| format!("Missing path segment '{segment}' in '{path}'"))?;
    }

    bail!("Upload variable path cannot be empty")
}

/// Rewrites a URL like `http://admin.localhost:4455/graphql` to connect via
/// `http://127.0.0.1:4455/graphql` while preserving the original host:port
/// as a Host header value for Oathkeeper virtual host routing.
fn rewrite_url(url: &str) -> (String, Option<String>) {
    if let Ok(parsed) = url::Url::parse(url) {
        let host = parsed.host_str().unwrap_or("localhost");
        let port = parsed.port_or_known_default().unwrap_or(80);
        let host_header = format!("{host}:{port}");

        // Rewrite only local dev virtual-host targets.
        if host.ends_with(".localhost") {
            let connect_url = format!("{}://127.0.0.1:{}{}", parsed.scheme(), port, parsed.path());
            (connect_url, Some(host_header))
        } else {
            (url.to_string(), None)
        }
    } else {
        (url.to_string(), None)
    }
}
