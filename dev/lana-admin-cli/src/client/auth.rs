use std::{
    fs,
    io::{self, IsTerminal, Write},
    path::PathBuf,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, bail};
use base64::Engine;
use rand::{RngExt, distr::Alphanumeric};
use reqwest::header::{COOKIE, LOCATION, SET_COOKIE};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use url::Url;

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
}

#[derive(Serialize, Deserialize)]
struct SessionFile {
    access_token: String,
    expires_at: u64,
    keycloak_url: String,
    #[serde(default = "default_keycloak_client_id")]
    keycloak_client_id: String,
    username: String,
}

pub struct AuthClient {
    keycloak_url: String,
    keycloak_client_id: String,
    keycloak_redirect_uri: String,
    username: String,
    password: String,
    http: reqwest::Client,
    cached: Option<CachedToken>,
}

struct CachedToken {
    access_token: String,
    expires_at: Instant,
}

fn session_path() -> Option<PathBuf> {
    std::env::var("HOME")
        .ok()
        .map(|home| PathBuf::from(home).join(".config/lana-admin-cli/session.json"))
}

fn now_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn default_keycloak_client_id() -> String {
    "admin-panel".to_string()
}

fn save_session(
    token: &str,
    expires_in: u64,
    keycloak_url: &str,
    keycloak_client_id: &str,
    username: &str,
) {
    let Some(path) = session_path() else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let session = SessionFile {
        access_token: token.to_string(),
        expires_at: now_epoch() + expires_in.saturating_sub(30),
        keycloak_url: keycloak_url.to_string(),
        keycloak_client_id: keycloak_client_id.to_string(),
        username: username.to_string(),
    };
    if let Ok(json) = serde_json::to_string_pretty(&session) {
        let _ = fs::write(&path, &json);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
        }
    }
}

fn load_session(
    keycloak_url: &str,
    keycloak_client_id: &str,
    username: &str,
) -> Option<SessionFile> {
    let path = session_path()?;
    let data = fs::read_to_string(path).ok()?;
    let session: SessionFile = serde_json::from_str(&data).ok()?;
    if session.keycloak_url != keycloak_url
        || session.keycloak_client_id != keycloak_client_id
        || session.username != username
    {
        return None;
    }
    if now_epoch() < session.expires_at {
        Some(session)
    } else {
        None
    }
}

pub fn clear_session() {
    if let Some(path) = session_path() {
        let _ = fs::remove_file(path);
    }
}

impl AuthClient {
    pub fn new(
        keycloak_url: String,
        keycloak_client_id: String,
        admin_url: String,
        username: String,
        password: String,
    ) -> Self {
        Self {
            keycloak_url,
            keycloak_client_id,
            keycloak_redirect_uri: derive_redirect_uri(&admin_url),
            username,
            password,
            http: reqwest::Client::new(),
            cached: None,
        }
    }

    pub async fn get_token(&mut self) -> anyhow::Result<String> {
        if let Some(ref cached) = self.cached
            && Instant::now() < cached.expires_at
        {
            return Ok(cached.access_token.clone());
        }

        if let Some(session) =
            load_session(&self.keycloak_url, &self.keycloak_client_id, &self.username)
        {
            let remaining = session.expires_at.saturating_sub(now_epoch());
            self.cached = Some(CachedToken {
                access_token: session.access_token.clone(),
                expires_at: Instant::now() + Duration::from_secs(remaining),
            });
            return Ok(session.access_token);
        }

        self.fetch_token().await
    }

    async fn fetch_token(&mut self) -> anyhow::Result<String> {
        // Prefer direct grant first for local/dev and automation.
        let direct_grant_result = self.fetch_token_password_grant().await;
        match direct_grant_result {
            Ok(token_resp) => return self.save_and_cache_token(token_resp),
            Err((status, body))
                if (status == reqwest::StatusCode::BAD_REQUEST
                    || status == reqwest::StatusCode::UNAUTHORIZED)
                    && (body.contains("unauthorized_client")
                        || body.contains("invalid_grant")
                        || body.contains("direct access grants")) =>
            {
                eprintln!("Direct grant not available. Falling back to authorization code + PKCE.");
            }
            Err((status, body)) => {
                bail!("Keycloak auth failed ({status}): {body}");
            }
        }

        let token_resp = match self.fetch_token_auth_code_grant_non_interactive().await {
            Ok(token_resp) => token_resp,
            Err(non_interactive_err) => {
                if io::stdin().is_terminal() {
                    eprintln!(
                        "Non-interactive PKCE login failed: {non_interactive_err}. Falling back to interactive browser login."
                    );
                    self.fetch_token_auth_code_grant().await?
                } else {
                    bail!(
                        "Non-interactive PKCE login failed: {non_interactive_err}. No interactive terminal available for browser fallback."
                    );
                }
            }
        };
        self.save_and_cache_token(token_resp)
    }

    async fn fetch_token_password_grant(
        &mut self,
    ) -> Result<TokenResponse, (reqwest::StatusCode, String)> {
        let url = format!(
            "{}/realms/internal/protocol/openid-connect/token",
            self.keycloak_url
        );

        let params = [
            ("client_id", self.keycloak_client_id.as_str()),
            ("username", self.username.as_str()),
            ("password", self.password.as_str()),
            ("grant_type", "password"),
            ("scope", "openid profile email"),
        ];

        let resp = self
            .http
            .post(&url)
            .form(&params)
            .send()
            .await
            .map_err(|e| {
                (
                    reqwest::StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to reach Keycloak: {e}"),
                )
            })?;

        let status = resp.status();
        let body = resp.text().await.map_err(|e| {
            (
                reqwest::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to read Keycloak response: {e}"),
            )
        })?;

        if !status.is_success() {
            return Err((status, body));
        }

        serde_json::from_str(&body).map_err(|e| {
            (
                reqwest::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to parse Keycloak token response: {e}"),
            )
        })
    }

    async fn fetch_token_auth_code_grant(&mut self) -> anyhow::Result<TokenResponse> {
        let code_verifier = random_alphanumeric(96);
        let state = random_alphanumeric(32);
        let nonce = random_alphanumeric(32);
        let code_challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(Sha256::digest(code_verifier.as_bytes()));
        let auth_url = build_authorization_url(
            &self.keycloak_url,
            &self.keycloak_client_id,
            &self.keycloak_redirect_uri,
            &state,
            &nonce,
            &code_challenge,
        )?;

        eprintln!();
        eprintln!("Open this URL in your browser and complete login:");
        eprintln!("{auth_url}");
        eprintln!();
        eprint!("Paste the final redirected URL: ");
        io::stderr().flush().ok();

        let mut redirected_url = String::new();
        io::stdin()
            .read_line(&mut redirected_url)
            .context("Failed to read redirected URL from stdin")?;
        let redirected_url = redirected_url.trim();
        if redirected_url.is_empty() {
            bail!("No redirected URL provided");
        }

        let callback = Url::parse(redirected_url)
            .with_context(|| format!("Invalid redirected URL: {redirected_url}"))?;
        let code = extract_auth_code(&callback, &state)?;
        self.exchange_auth_code(&code, &code_verifier).await
    }

    async fn fetch_token_auth_code_grant_non_interactive(
        &mut self,
    ) -> anyhow::Result<TokenResponse> {
        let code_verifier = random_alphanumeric(96);
        let state = random_alphanumeric(32);
        let nonce = random_alphanumeric(32);
        let code_challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(Sha256::digest(code_verifier.as_bytes()));
        let auth_url = build_authorization_url(
            &self.keycloak_url,
            &self.keycloak_client_id,
            &self.keycloak_redirect_uri,
            &state,
            &nonce,
            &code_challenge,
        )?;

        let browserless_http = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .context("Failed to build HTTP client for PKCE login")?;

        let auth_resp = browserless_http
            .get(&auth_url)
            .send()
            .await
            .context("Failed to reach Keycloak authorization endpoint")?;
        let auth_url = auth_resp.url().clone();
        let auth_headers = auth_resp.headers().clone();
        let auth_status = auth_resp.status();

        if auth_status.is_redirection() {
            let location = header_value(&auth_headers, LOCATION)
                .context("Authorization redirect missing Location header")?;
            let callback =
                resolve_url(&auth_url, &location).context("Invalid authorization redirect URL")?;
            let code = extract_auth_code(&callback, &state)?;
            return self.exchange_auth_code(&code, &code_verifier).await;
        }

        let login_page = auth_resp
            .text()
            .await
            .context("Failed to read Keycloak authorization response")?;
        if !auth_status.is_success() {
            bail!("Authorization endpoint failed ({auth_status}): {login_page}");
        }

        let action = extract_login_form_action(&login_page)
            .context("Could not find Keycloak login form action")?;
        let action_url =
            resolve_url(&auth_url, &action).context("Invalid login form action URL")?;
        let cookie_header = build_cookie_header(&auth_headers);

        let mut post = browserless_http.post(action_url).form(&[
            ("username", self.username.as_str()),
            // Keycloak accepts this as the submit button name for default login theme.
            ("login", "Sign In"),
        ]);
        if let Some(cookie_value) = cookie_header {
            post = post.header(COOKIE, cookie_value);
        }

        let submit_resp = post
            .send()
            .await
            .context("Failed to submit Keycloak login form")?;
        let submit_url = submit_resp.url().clone();
        let submit_headers = submit_resp.headers().clone();
        let submit_status = submit_resp.status();

        if !submit_status.is_redirection() {
            let body = submit_resp
                .text()
                .await
                .context("Failed to read Keycloak login form response")?;
            bail!(
                "Login form submission did not redirect ({submit_status}). Response snippet: {}",
                trim_for_error(&body)
            );
        }

        let location = header_value(&submit_headers, LOCATION)
            .context("Login redirect missing Location header")?;
        let callback = resolve_url(&submit_url, &location).context("Invalid login redirect URL")?;
        let code = extract_auth_code(&callback, &state)?;
        self.exchange_auth_code(&code, &code_verifier).await
    }

    async fn exchange_auth_code(
        &self,
        code: &str,
        code_verifier: &str,
    ) -> anyhow::Result<TokenResponse> {
        let token_url = format!(
            "{}/realms/internal/protocol/openid-connect/token",
            self.keycloak_url
        );
        let token_params = [
            ("client_id", self.keycloak_client_id.as_str()),
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", self.keycloak_redirect_uri.as_str()),
            ("code_verifier", code_verifier),
        ];

        let resp = self
            .http
            .post(&token_url)
            .form(&token_params)
            .send()
            .await
            .context("Failed to reach Keycloak token endpoint")?;

        let status = resp.status();
        let body = resp
            .text()
            .await
            .context("Failed to read Keycloak token response")?;

        if !status.is_success() {
            bail!("Keycloak auth code exchange failed ({status}): {body}");
        }

        serde_json::from_str(&body).context("Failed to parse Keycloak token response")
    }

    fn save_and_cache_token(&mut self, token_resp: TokenResponse) -> anyhow::Result<String> {
        let expires_at =
            Instant::now() + Duration::from_secs(token_resp.expires_in.saturating_sub(30));

        save_session(
            &token_resp.access_token,
            token_resp.expires_in,
            &self.keycloak_url,
            &self.keycloak_client_id,
            &self.username,
        );

        let token = token_resp.access_token.clone();
        self.cached = Some(CachedToken {
            access_token: token_resp.access_token,
            expires_at,
        });

        Ok(token)
    }
}

fn derive_redirect_uri(admin_url: &str) -> String {
    if let Ok(mut parsed) = Url::parse(admin_url) {
        parsed.set_path("/");
        parsed.set_query(None);
        parsed.set_fragment(None);
        return parsed.to_string();
    }
    admin_url.to_string()
}

fn random_alphanumeric(len: usize) -> String {
    rand::rng()
        .sample_iter(Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

fn build_authorization_url(
    keycloak_url: &str,
    keycloak_client_id: &str,
    keycloak_redirect_uri: &str,
    state: &str,
    nonce: &str,
    code_challenge: &str,
) -> anyhow::Result<String> {
    let mut url = Url::parse(&format!(
        "{}/realms/internal/protocol/openid-connect/auth",
        keycloak_url
    ))
    .context("Invalid keycloak-url")?;
    url.query_pairs_mut()
        .append_pair("client_id", keycloak_client_id)
        .append_pair("redirect_uri", keycloak_redirect_uri)
        .append_pair("state", state)
        .append_pair("response_mode", "fragment")
        .append_pair("response_type", "code")
        .append_pair("scope", "openid profile email")
        .append_pair("nonce", nonce)
        .append_pair("code_challenge", code_challenge)
        .append_pair("code_challenge_method", "S256");
    Ok(url.to_string())
}

fn extract_params_from_url(url: &Url) -> std::collections::HashMap<String, String> {
    let mut params = std::collections::HashMap::new();
    if let Some(query) = url.query() {
        for (k, v) in url::form_urlencoded::parse(query.as_bytes()) {
            params.insert(k.into_owned(), v.into_owned());
        }
    }
    if let Some(fragment) = url.fragment() {
        for (k, v) in url::form_urlencoded::parse(fragment.as_bytes()) {
            params.insert(k.into_owned(), v.into_owned());
        }
    }
    params
}

fn extract_auth_code(callback: &Url, expected_state: &str) -> anyhow::Result<String> {
    let params = extract_params_from_url(callback);
    if let Some(error) = params.get("error") {
        let description = params
            .get("error_description")
            .map(String::as_str)
            .unwrap_or("no description");
        bail!("Browser login failed: {error} ({description})");
    }
    let returned_state = params
        .get("state")
        .context("Missing 'state' parameter in redirected URL")?;
    if returned_state != expected_state {
        bail!("State mismatch in redirected URL");
    }
    let code = params
        .get("code")
        .context("Missing 'code' parameter in redirected URL")?;
    Ok(code.to_string())
}

fn extract_login_form_action(html: &str) -> Option<String> {
    let mut cursor = 0usize;
    while let Some(form_start_rel) = html[cursor..].find("<form") {
        let form_start = cursor + form_start_rel;
        let form_end = html[form_start..].find('>')?;
        let tag = &html[form_start..=form_start + form_end];
        if tag.contains("kc-form-login")
            && let Some(action) = extract_html_attr(tag, "action")
        {
            return Some(action.replace("&amp;", "&"));
        }
        cursor = form_start + form_end + 1;
    }
    None
}

fn extract_html_attr(tag: &str, attr: &str) -> Option<String> {
    let double_quoted = format!(r#"{attr}=""#);
    if let Some(start) = tag.find(&double_quoted) {
        let value_start = start + double_quoted.len();
        let value_end = tag[value_start..].find('"')?;
        return Some(tag[value_start..value_start + value_end].to_string());
    }

    let single_quoted = format!("{attr}='");
    if let Some(start) = tag.find(&single_quoted) {
        let value_start = start + single_quoted.len();
        let value_end = tag[value_start..].find('\'')?;
        return Some(tag[value_start..value_start + value_end].to_string());
    }

    None
}

fn build_cookie_header(headers: &reqwest::header::HeaderMap) -> Option<String> {
    let mut cookies = Vec::new();
    for value in headers.get_all(SET_COOKIE) {
        let Ok(raw_cookie) = value.to_str() else {
            continue;
        };
        let Some(cookie_pair) = raw_cookie.split(';').next() else {
            continue;
        };
        let trimmed = cookie_pair.trim();
        if !trimmed.is_empty() {
            cookies.push(trimmed.to_string());
        }
    }
    if cookies.is_empty() {
        None
    } else {
        Some(cookies.join("; "))
    }
}

fn header_value(
    headers: &reqwest::header::HeaderMap,
    name: reqwest::header::HeaderName,
) -> Option<String> {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(ToString::to_string)
}

fn resolve_url(base: &Url, candidate: &str) -> anyhow::Result<Url> {
    if let Ok(url) = Url::parse(candidate) {
        return Ok(url);
    }
    base.join(candidate)
        .with_context(|| format!("Failed to resolve URL '{candidate}' against '{base}'"))
}

fn trim_for_error(text: &str) -> String {
    text.chars().take(200).collect()
}
