use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, bail};
use graphql_client_codegen::{
    CodegenMode, GraphQLClientCodegenOptions, generate_module_token_stream_from_string,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tempfile::{Builder, NamedTempFile};
use url::Url;

use crate::{
    cli::SchemaAction,
    client::auth::{AuthClient, load_saved_login_profile},
    output,
};

const DEFAULT_SCHEMA_RELATIVE_PATH: &str = "lana/admin-server/src/graphql/schema.graphql";
const INTROSPECTION_QUERY: &str = r#"query IntrospectionQuery {
  __schema {
    queryType { name }
    mutationType { name }
    subscriptionType { name }
    types {
      ...FullType
    }
    directives {
      name
      description
      locations
      args {
        ...InputValue
      }
    }
  }
}

fragment FullType on __Type {
  kind
  name
  description
  fields(includeDeprecated: true) {
    name
    description
    args {
      ...InputValue
    }
    type {
      ...TypeRef
    }
    isDeprecated
    deprecationReason
  }
  inputFields {
    ...InputValue
  }
  interfaces {
    ...TypeRef
  }
  enumValues(includeDeprecated: true) {
    name
    description
    isDeprecated
    deprecationReason
  }
  possibleTypes {
    ...TypeRef
  }
}

fragment InputValue on __InputValue {
  name
  description
  type {
    ...TypeRef
  }
  defaultValue
}

fragment TypeRef on __Type {
  kind
  name
  ofType {
    kind
    name
    ofType {
      kind
      name
      ofType {
        kind
        name
        ofType {
          kind
          name
          ofType {
            kind
            name
            ofType {
              kind
              name
              ofType {
                kind
                name
              }
            }
          }
        }
      }
    }
  }
}"#;

struct EmbeddedDocument {
    path: &'static str,
    content: &'static str,
}

const GRAPHQL_DOCUMENTS: &[EmbeddedDocument] = &[
    EmbeddedDocument {
        path: "src/graphql/accounting.graphql",
        content: include_str!("graphql/accounting.graphql"),
    },
    EmbeddedDocument {
        path: "src/graphql/approval_process.graphql",
        content: include_str!("graphql/approval_process.graphql"),
    },
    EmbeddedDocument {
        path: "src/graphql/audit.graphql",
        content: include_str!("graphql/audit.graphql"),
    },
    EmbeddedDocument {
        path: "src/graphql/build_info.graphql",
        content: include_str!("graphql/build_info.graphql"),
    },
    EmbeddedDocument {
        path: "src/graphql/collateral.graphql",
        content: include_str!("graphql/collateral.graphql"),
    },
    EmbeddedDocument {
        path: "src/graphql/credit_facility.graphql",
        content: include_str!("graphql/credit_facility.graphql"),
    },
    EmbeddedDocument {
        path: "src/graphql/csv_export.graphql",
        content: include_str!("graphql/csv_export.graphql"),
    },
    EmbeddedDocument {
        path: "src/graphql/custodian.graphql",
        content: include_str!("graphql/custodian.graphql"),
    },
    EmbeddedDocument {
        path: "src/graphql/customer.graphql",
        content: include_str!("graphql/customer.graphql"),
    },
    EmbeddedDocument {
        path: "src/graphql/dashboard.graphql",
        content: include_str!("graphql/dashboard.graphql"),
    },
    EmbeddedDocument {
        path: "src/graphql/deposit_account.graphql",
        content: include_str!("graphql/deposit_account.graphql"),
    },
    EmbeddedDocument {
        path: "src/graphql/deposit_mutations.graphql",
        content: include_str!("graphql/deposit_mutations.graphql"),
    },
    EmbeddedDocument {
        path: "src/graphql/document.graphql",
        content: include_str!("graphql/document.graphql"),
    },
    EmbeddedDocument {
        path: "src/graphql/domain_config.graphql",
        content: include_str!("graphql/domain_config.graphql"),
    },
    EmbeddedDocument {
        path: "src/graphql/financial_statement.graphql",
        content: include_str!("graphql/financial_statement.graphql"),
    },
    EmbeddedDocument {
        path: "src/graphql/fiscal_year.graphql",
        content: include_str!("graphql/fiscal_year.graphql"),
    },
    EmbeddedDocument {
        path: "src/graphql/liquidation.graphql",
        content: include_str!("graphql/liquidation.graphql"),
    },
    EmbeddedDocument {
        path: "src/graphql/loan_agreement.graphql",
        content: include_str!("graphql/loan_agreement.graphql"),
    },
    EmbeddedDocument {
        path: "src/graphql/prospect.graphql",
        content: include_str!("graphql/prospect.graphql"),
    },
    EmbeddedDocument {
        path: "src/graphql/prospect_mutations.graphql",
        content: include_str!("graphql/prospect_mutations.graphql"),
    },
    EmbeddedDocument {
        path: "src/graphql/report.graphql",
        content: include_str!("graphql/report.graphql"),
    },
    EmbeddedDocument {
        path: "src/graphql/role.graphql",
        content: include_str!("graphql/role.graphql"),
    },
    EmbeddedDocument {
        path: "src/graphql/terms_template.graphql",
        content: include_str!("graphql/terms_template.graphql"),
    },
    EmbeddedDocument {
        path: "src/graphql/user.graphql",
        content: include_str!("graphql/user.graphql"),
    },
    EmbeddedDocument {
        path: "src/graphql/withdrawal.graphql",
        content: include_str!("graphql/withdrawal.graphql"),
    },
];

enum SchemaSource {
    File {
        path: PathBuf,
    },
    Remote {
        admin_url: String,
        schema_file: NamedTempFile,
        authenticated: bool,
    },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SchemaSourceSummary {
    kind: &'static str,
    value: String,
    authenticated: Option<bool>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DocumentCheckResult {
    path: &'static str,
    valid: bool,
    error: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SchemaCheckReport {
    ok: bool,
    source: SchemaSourceSummary,
    checked_document_count: usize,
    failure_count: usize,
    documents: Vec<DocumentCheckResult>,
}

#[derive(Deserialize)]
struct RemoteGraphqlResponse {
    data: Option<serde_json::Value>,
    errors: Option<Vec<RemoteGraphqlError>>,
}

#[derive(Deserialize)]
struct RemoteGraphqlError {
    message: String,
}

pub async fn execute(action: SchemaAction, json_output: bool) -> anyhow::Result<()> {
    let result = run_check(action).await;
    match result {
        Ok(report) => {
            if json_output {
                output::print_json(&report)?;
            } else {
                print_human_report(&report);
            }

            if report.ok {
                Ok(())
            } else {
                std::process::exit(1);
            }
        }
        Err(err) => {
            if json_output {
                output::print_json(&json!({
                    "ok": false,
                    "error": err.to_string(),
                }))?;
                std::process::exit(1);
            }
            Err(err)
        }
    }
}

async fn run_check(action: SchemaAction) -> anyhow::Result<SchemaCheckReport> {
    let SchemaAction::Check {
        schema_file,
        admin_url,
    } = action;

    let source = match (schema_file, admin_url) {
        (Some(path), None) => SchemaSource::File {
            path: canonicalize_or_keep(path),
        },
        (None, Some(url)) => fetch_remote_schema(&url).await?,
        (None, None) => SchemaSource::File {
            path: resolve_default_schema_file()?,
        },
        (Some(_), Some(_)) => {
            bail!("Use either `--schema-file` or `--admin-url`, not both");
        }
    };

    let schema_path = source.path();
    if !schema_path.exists() {
        bail!("Schema file not found at {}", schema_path.display());
    }

    let mut documents = Vec::with_capacity(GRAPHQL_DOCUMENTS.len());
    let mut failure_count = 0_usize;

    for document in GRAPHQL_DOCUMENTS {
        let result = validate_document(document, schema_path);
        if result.error.is_some() {
            failure_count += 1;
        }
        documents.push(result);
    }

    Ok(SchemaCheckReport {
        ok: failure_count == 0,
        source: source.summary(),
        checked_document_count: documents.len(),
        failure_count,
        documents,
    })
}

fn validate_document(document: &EmbeddedDocument, schema_path: &Path) -> DocumentCheckResult {
    let options = GraphQLClientCodegenOptions::new(CodegenMode::Cli);
    let error = generate_module_token_stream_from_string(document.content, schema_path, options)
        .err()
        .map(|err| err.to_string());

    DocumentCheckResult {
        path: document.path,
        valid: error.is_none(),
        error,
    }
}

fn print_human_report(report: &SchemaCheckReport) {
    output::print_kv(&[
        ("Schema Source", report.source.kind),
        ("Schema Value", &report.source.value),
        (
            "Documents Checked",
            &report.checked_document_count.to_string(),
        ),
        ("Failures", &report.failure_count.to_string()),
    ]);
    if let Some(authenticated) = report.source.authenticated {
        output::print_kv(&[(
            "Authenticated",
            if authenticated { "true" } else { "false" },
        )]);
    }

    if report.ok {
        println!("All embedded GraphQL documents are compatible.");
        return;
    }

    println!("Incompatible GraphQL documents:");
    for document in report.documents.iter().filter(|doc| !doc.valid) {
        println!();
        println!("- {}", document.path);
        if let Some(error) = &document.error {
            for line in error.lines() {
                println!("  {line}");
            }
        }
    }
}

fn resolve_default_schema_file() -> anyhow::Result<PathBuf> {
    let cwd = std::env::current_dir().context("Failed to resolve current directory")?;
    for base in cwd.ancestors() {
        let candidate = base.join(DEFAULT_SCHEMA_RELATIVE_PATH);
        if candidate.is_file() {
            return Ok(canonicalize_or_keep(candidate));
        }
    }

    let manifest_fallback = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../lana/admin-server/src/graphql/schema.graphql");
    if manifest_fallback.is_file() {
        return Ok(canonicalize_or_keep(manifest_fallback));
    }

    bail!(
        "Could not find `{DEFAULT_SCHEMA_RELATIVE_PATH}` from the current directory. Pass `--schema-file` explicitly if running outside the repository."
    );
}

async fn fetch_remote_schema(admin_url: &str) -> anyhow::Result<SchemaSource> {
    let request_url = validated_request_url(admin_url)?;
    let (connect_url, host_header) = rewrite_url(admin_url);

    let mut request = reqwest::Client::new().post(request_url).json(&json!({
        "operationName": "IntrospectionQuery",
        "query": INTROSPECTION_QUERY,
        "variables": {},
    }));

    if let Some(host) = host_header {
        request = request.header("Host", host);
    }

    let mut authenticated = false;
    if let Some(token) = maybe_saved_bearer_token(admin_url).await? {
        request = request.header("Authorization", format!("Bearer {token}"));
        authenticated = true;
    }

    let response = request
        .send()
        .await
        .with_context(|| format!("Failed to fetch remote schema from {connect_url}"))?;
    let status = response.status();
    let body = response
        .text()
        .await
        .context("Failed to read remote schema response body")?;

    if !status.is_success() {
        let auth_hint = if status == reqwest::StatusCode::UNAUTHORIZED
            || status == reqwest::StatusCode::FORBIDDEN
        {
            " Run `lana-admin auth login --admin-url ...` first if the endpoint requires authentication."
        } else {
            ""
        };
        bail!("HTTP {status}: {body}{auth_hint}");
    }

    let parsed: RemoteGraphqlResponse =
        serde_json::from_str(&body).context("Failed to parse remote introspection response")?;
    if let Some(errors) = parsed.errors {
        let messages = errors
            .into_iter()
            .map(|err| err.message)
            .collect::<Vec<_>>()
            .join("; ");
        bail!("Remote introspection returned GraphQL errors: {messages}");
    }
    if parsed.data.is_none() {
        bail!("Remote introspection returned no `data` payload");
    }

    let schema_file = Builder::new()
        .prefix("lana-admin-schema-")
        .suffix(".json")
        .tempfile()
        .context("Failed to create temporary schema file")?;
    fs::write(schema_file.path(), body).context("Failed to write temporary schema file")?;

    Ok(SchemaSource::Remote {
        admin_url: admin_url.to_string(),
        schema_file,
        authenticated,
    })
}

async fn maybe_saved_bearer_token(admin_url: &str) -> anyhow::Result<Option<String>> {
    let Ok(saved) = load_saved_login_profile() else {
        return Ok(None);
    };
    if saved.admin_url != admin_url {
        return Ok(None);
    }

    let mut auth = AuthClient::new(
        saved.keycloak_url,
        saved.keycloak_client_id,
        saved.admin_url,
        saved.username,
        saved.password,
    );
    Ok(Some(auth.get_token().await?))
}

fn canonicalize_or_keep(path: PathBuf) -> PathBuf {
    path.canonicalize().unwrap_or(path)
}

fn validated_request_url(url: &str) -> anyhow::Result<Url> {
    let (connect_url, _) = rewrite_url(url);
    let parsed =
        Url::parse(&connect_url).with_context(|| format!("Invalid admin URL '{connect_url}'"))?;

    match parsed.scheme() {
        "https" => Ok(parsed),
        "http" if is_local_http_host(parsed.host_str()) => Ok(parsed),
        "http" => bail!(
            "Refusing insecure admin URL '{}'. Use HTTPS (or localhost/127.0.0.1 for local dev).",
            connect_url
        ),
        scheme => bail!(
            "Unsupported admin URL scheme '{}' in '{}'",
            scheme,
            connect_url
        ),
    }
}

fn is_local_http_host(host: Option<&str>) -> bool {
    matches!(
        host,
        Some("localhost") | Some("127.0.0.1") | Some("::1") | Some("admin.localhost")
    )
}

fn rewrite_url(url: &str) -> (String, Option<String>) {
    if let Ok(parsed) = Url::parse(url) {
        let host = parsed.host_str().unwrap_or("localhost");
        let port = parsed.port_or_known_default().unwrap_or(80);
        let host_header = format!("{host}:{port}");

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

impl SchemaSource {
    fn path(&self) -> &Path {
        match self {
            SchemaSource::File { path } => path.as_path(),
            SchemaSource::Remote { schema_file, .. } => schema_file.path(),
        }
    }

    fn summary(&self) -> SchemaSourceSummary {
        match self {
            SchemaSource::File { path } => SchemaSourceSummary {
                kind: "schemaFile",
                value: path.display().to_string(),
                authenticated: None,
            },
            SchemaSource::Remote {
                admin_url,
                authenticated,
                ..
            } => SchemaSourceSummary {
                kind: "adminUrl",
                value: admin_url.clone(),
                authenticated: Some(*authenticated),
            },
        }
    }
}
