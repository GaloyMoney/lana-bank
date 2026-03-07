pub mod auth;

use anyhow::{Context, bail};
use graphql_client::GraphQLQuery;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use std::path::Path;
use url::Url;

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

pub const CLI_BUILD_VERSION: &str = env!("BUILD_VERSION");

#[derive(Debug)]
pub struct PreviewComplete;

impl std::fmt::Display for PreviewComplete {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GraphQL preview complete")
    }
}

impl std::error::Error for PreviewComplete {}

pub fn is_preview_complete(err: &anyhow::Error) -> bool {
    err.downcast_ref::<PreviewComplete>().is_some()
}

pub struct GraphQLClient {
    connect_url: String,
    host_header: Option<String>,
    http: reqwest::Client,
    auth: AuthClient,
    graphql_debug_level: u8,
    preview_graphql: bool,
}

impl GraphQLClient {
    pub fn new(
        admin_url: String,
        auth: AuthClient,
        graphql_debug_level: u8,
        preview_graphql: bool,
    ) -> Self {
        // Parse the admin URL to extract the Host header and rewrite the URL
        // to connect via 127.0.0.1. This handles the common case where
        // admin.localhost doesn't resolve in DNS but Oathkeeper routes by Host header.
        let (connect_url, host_header) = rewrite_url(&admin_url);
        Self {
            connect_url,
            host_header,
            http: reqwest::Client::new(),
            auth,
            graphql_debug_level,
            preview_graphql,
        }
    }

    pub async fn build_info(
        &mut self,
    ) -> anyhow::Result<crate::graphql::build_info_get::BuildInfoGetBuildInfo> {
        use crate::graphql::{BuildInfoGet, build_info_get};

        let data = self
            .execute::<BuildInfoGet>(build_info_get::Variables {})
            .await?;
        Ok(data.build_info)
    }

    pub async fn execute<Q: GraphQLQuery>(
        &mut self,
        variables: Q::Variables,
    ) -> anyhow::Result<Q::ResponseData>
    where
        Q::ResponseData: DeserializeOwned,
    {
        let body = Q::build_query(variables);

        if self.preview_graphql {
            self.print_operation(
                body.operation_name,
                body.query,
                &body.variables,
                None,
                true,
                true,
            );
            return Err(PreviewComplete.into());
        }

        self.print_operation(
            body.operation_name,
            body.query,
            &body.variables,
            None,
            true,
            false,
        );
        for attempt in 0..=1 {
            let token = self.auth.get_token().await?;
            let request_url = self.validated_request_url()?;

            let mut req = self
                .http
                .post(request_url)
                .header("Authorization", format!("Bearer {token}"))
                .json(&body);
            if let Some(ref host) = self.host_header {
                req = req.header("Host", host);
            }

            let resp = req.send().await.context("Failed to reach admin server")?;

            match self.parse_response(resp).await {
                Err(err) if attempt == 0 && is_retryable_auth_error(&err) => {
                    self.auth.invalidate();
                    continue;
                }
                other => return other,
            }
        }

        unreachable!("request retry loop should always return")
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

        if self.preview_graphql {
            let operation_name = body
                .get("operation_name")
                .or_else(|| body.get("operationName"))
                .and_then(Value::as_str)
                .unwrap_or("<unknown>");
            let query = body
                .get("query")
                .and_then(Value::as_str)
                .unwrap_or("<query unavailable>");
            let variables = body.get("variables").cloned().unwrap_or(Value::Null);
            self.print_operation(
                operation_name,
                query,
                &variables,
                Some(&uploads),
                true,
                true,
            );
            return Err(PreviewComplete.into());
        }

        self.debug_print_multipart_operation(&body, &uploads);

        let operations_json = body.to_string();
        let map_json = serde_json::Value::Object(map).to_string();

        for attempt in 0..=1 {
            let mut form = reqwest::multipart::Form::new()
                .text("operations", operations_json.clone())
                .text("map", map_json.clone());

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
            let request_url = self.validated_request_url()?;

            let mut req = self
                .http
                .post(request_url)
                .header("Authorization", format!("Bearer {token}"))
                .multipart(form);
            if let Some(ref host) = self.host_header {
                req = req.header("Host", host);
            }

            let resp = req.send().await.context("Failed to reach admin server")?;

            match self.parse_response(resp).await {
                Err(err) if attempt == 0 && is_retryable_auth_error(&err) => {
                    self.auth.invalidate();
                    continue;
                }
                other => return other,
            }
        }

        unreachable!("multipart request retry loop should always return")
    }

    async fn parse_response<R: DeserializeOwned>(
        &self,
        resp: reqwest::Response,
    ) -> anyhow::Result<R> {
        let status = resp.status();
        let text = resp.text().await.context("Failed to read response body")?;

        if self.graphql_debug_level >= 2 {
            eprintln!("\n=== GraphQL Response (raw payload) ===");
            if let Ok(value) = serde_json::from_str::<Value>(&text) {
                match serde_json::to_string_pretty(&value) {
                    Ok(pretty) => eprintln!("{pretty}"),
                    Err(_) => eprintln!("{text}"),
                }
            } else {
                eprintln!("{text}");
            }
        }

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

    fn print_operation<V: serde::Serialize>(
        &self,
        operation_name: &str,
        query: &str,
        variables: &V,
        multipart_uploads: Option<&[MultipartUpload]>,
        include_variables: bool,
        force: bool,
    ) {
        if !force && self.graphql_debug_level == 0 {
            return;
        }

        let variables_json = serde_json::to_value(variables).unwrap_or_else(
            |_| json!({"_debugError":"Failed to serialize GraphQL variables for debug output"}),
        );
        let redacted_variables = redact_sensitive_json(variables_json);
        let displayed_query = extract_operation_block(query, operation_name);

        eprintln!("\n=== GraphQL Request ===");
        eprintln!("operation: {operation_name}");
        eprintln!("query:\n{displayed_query}");
        if include_variables {
            eprintln!(
                "variables:\n{}",
                serde_json::to_string_pretty(&redacted_variables)
                    .unwrap_or_else(|_| "<failed to format variables>".to_string())
            );
        }

        if let Some(uploads) = multipart_uploads
            && !uploads.is_empty()
        {
            eprintln!("multipartUploads:");
            for upload in uploads {
                eprintln!(
                    "  - variable: {} | file: {}",
                    upload.variable_path, upload.file_path
                );
            }
        }

        if !force && self.graphql_debug_level >= 2 {
            eprintln!("responseHint: run uses raw GraphQL payload output at this verbosity level");
        }
    }

    fn debug_print_multipart_operation(&self, operations: &Value, uploads: &[MultipartUpload]) {
        if self.graphql_debug_level == 0 {
            return;
        }

        let operation_name = operations
            .get("operation_name")
            .or_else(|| operations.get("operationName"))
            .and_then(Value::as_str)
            .unwrap_or("<unknown>");
        let query = operations
            .get("query")
            .and_then(Value::as_str)
            .unwrap_or("<query unavailable>");
        let variables = operations.get("variables").cloned().unwrap_or(Value::Null);

        self.print_operation(
            operation_name,
            query,
            &variables,
            Some(uploads),
            true,
            false,
        );
    }

    fn validated_request_url(&self) -> anyhow::Result<Url> {
        let parsed = Url::parse(&self.connect_url)
            .with_context(|| format!("Invalid admin URL '{}'", self.connect_url))?;

        match parsed.scheme() {
            "https" => Ok(parsed),
            // Keep local dev ergonomics while rejecting cleartext remote admin endpoints.
            "http" if is_local_http_host(parsed.host_str()) => Ok(parsed),
            "http" => bail!(
                "Refusing insecure admin URL '{}'. Use HTTPS (or localhost/127.0.0.1 for local dev).",
                self.connect_url
            ),
            scheme => bail!(
                "Unsupported admin URL scheme '{}' in '{}'",
                scheme,
                self.connect_url
            ),
        }
    }
}

fn is_retryable_auth_error(err: &anyhow::Error) -> bool {
    let message = err.to_string().to_ascii_lowercase();
    message.contains("http 401")
        || message.contains("http 403")
        || message.contains("unauthorized")
        || message.contains("unauthenticated")
        || message.contains("invalid token")
        || message.contains("token expired")
        || message.contains("expired token")
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

fn is_local_http_host(host: Option<&str>) -> bool {
    matches!(
        host,
        Some("localhost") | Some("127.0.0.1") | Some("::1") | Some("admin.localhost")
    )
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

fn redact_sensitive_json(value: Value) -> Value {
    fn is_sensitive_key(key: &str) -> bool {
        let key = key.to_ascii_lowercase();
        key.contains("password")
            || key.contains("secret")
            || key.contains("token")
            || key == "authorization"
            || key == "apikey"
            || key == "api_key"
            || key == "apisecret"
            || key == "api_secret"
            || key == "longlivedtoken"
            || key == "long_lived_token"
            || key == "webhooksecret"
            || key == "webhook_secret"
    }

    match value {
        Value::Object(map) => {
            let mut out = serde_json::Map::new();
            for (k, v) in map {
                if is_sensitive_key(&k) {
                    out.insert(k, Value::String("***REDACTED***".to_string()));
                } else {
                    out.insert(k, redact_sensitive_json(v));
                }
            }
            Value::Object(out)
        }
        Value::Array(items) => Value::Array(items.into_iter().map(redact_sensitive_json).collect()),
        other => other,
    }
}

pub(crate) fn extract_operation_block(document: &str, operation_name: &str) -> String {
    if operation_name.trim().is_empty() {
        return document.trim().to_string();
    }

    for op_kind in ["query", "mutation", "subscription"] {
        let needle = format!("{op_kind} {operation_name}");
        if let Some(start) = document.find(&needle) {
            return extract_balanced_block(document, start);
        }
    }

    document.trim().to_string()
}

fn extract_balanced_block(document: &str, start: usize) -> String {
    let bytes = document.as_bytes();
    let Some(brace_start) = bytes[start..]
        .iter()
        .position(|b| *b == b'{')
        .map(|idx| start + idx)
    else {
        return document[start..].trim().to_string();
    };

    let mut depth = 0_i32;
    let mut end = bytes.len();
    for (idx, byte) in bytes.iter().enumerate().skip(brace_start) {
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    end = idx + 1;
                    break;
                }
            }
            _ => {}
        }
    }

    document[start..end].trim().to_string()
}
