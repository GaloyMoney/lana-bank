use std::{
    collections::BTreeMap,
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
    #[serde(default)]
    refresh_token: String,
    #[serde(default)]
    refresh_expires_in: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SessionFile {
    access_token: String,
    expires_at: u64,
    #[serde(default)]
    refresh_token: String,
    #[serde(default)]
    refresh_expires_at: u64,
    #[serde(default = "default_admin_url")]
    admin_url: String,
    keycloak_url: String,
    #[serde(default = "default_keycloak_client_id")]
    keycloak_client_id: String,
    username: String,
    #[serde(default)]
    password: String,
}

#[derive(Debug, Clone)]
pub struct SavedLoginProfile {
    pub admin_url: String,
    pub keycloak_url: String,
    pub keycloak_client_id: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone)]
pub struct SavedSessionInfo {
    pub session_path: PathBuf,
    pub admin_url: String,
    pub keycloak_url: String,
    pub keycloak_client_id: String,
    pub username: String,
    pub password: String,
    pub password_set: bool,
    pub expires_at: u64,
    pub is_expired: bool,
}

pub struct AuthClient {
    admin_url: String,
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

fn home_session_path() -> Option<PathBuf> {
    std::env::var("HOME")
        .ok()
        .map(|home| PathBuf::from(home).join(".config/lana-admin/session.json"))
}

fn local_session_path() -> Option<PathBuf> {
    std::env::current_dir()
        .ok()
        .map(|dir| dir.join(".lana-admin/session.json"))
}

fn session_paths() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Some(local) = local_session_path() {
        out.push(local);
    }
    if let Some(home) = home_session_path()
        && out.iter().all(|p| p != &home)
    {
        out.push(home);
    }
    out
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

fn default_admin_url() -> String {
    "https://admin.staging.galoy.io/graphql".to_string()
}

fn read_session_file() -> Option<SessionFile> {
    session_paths().into_iter().find_map(|path| {
        let data = fs::read_to_string(path).ok()?;
        serde_json::from_str(&data).ok()
    })
}

pub fn load_saved_login_profile() -> anyhow::Result<SavedLoginProfile> {
    let info = load_saved_session_info()?;
    Ok(SavedLoginProfile {
        admin_url: info.admin_url,
        keycloak_url: info.keycloak_url,
        keycloak_client_id: info.keycloak_client_id,
        username: info.username,
        password: info.password,
    })
}

pub fn load_saved_session_info() -> anyhow::Result<SavedSessionInfo> {
    let paths = session_paths();
    if paths.is_empty() {
        bail!("Unable to resolve any session path");
    }

    for path in paths {
        let Ok(data) = fs::read_to_string(&path) else {
            continue;
        };
        let session: SessionFile = serde_json::from_str(&data).with_context(|| {
            format!(
                "Failed to parse saved login profile at {}. Run `lana-admin login ...` again.",
                path.display()
            )
        })?;
        return Ok(SavedSessionInfo {
            session_path: path,
            admin_url: session.admin_url,
            keycloak_url: session.keycloak_url,
            keycloak_client_id: session.keycloak_client_id,
            username: session.username,
            password: session.password.clone(),
            password_set: !session.password.is_empty(),
            expires_at: session.expires_at,
            is_expired: now_epoch() >= session.expires_at,
        });
    }

    let checked = session_paths()
        .into_iter()
        .map(|p| p.display().to_string())
        .collect::<Vec<_>>()
        .join(", ");
    bail!(
        "No saved login profile found. Checked: {}. Run `lana-admin login ...` first.",
        checked
    );
}

fn save_session(
    token: &str,
    expires_in: u64,
    refresh_token: &str,
    refresh_expires_in: u64,
    admin_url: &str,
    keycloak_url: &str,
    keycloak_client_id: &str,
    username: &str,
    password: &str,
) {
    let session = SessionFile {
        access_token: token.to_string(),
        expires_at: now_epoch() + expires_in.saturating_sub(30),
        refresh_token: refresh_token.to_string(),
        refresh_expires_at: now_epoch() + refresh_expires_in.saturating_sub(30),
        admin_url: admin_url.to_string(),
        keycloak_url: keycloak_url.to_string(),
        keycloak_client_id: keycloak_client_id.to_string(),
        username: username.to_string(),
        password: password.to_string(),
    };
    write_session_files(&session);
}

fn write_session_files(session: &SessionFile) {
    let Ok(json) = serde_json::to_string_pretty(session) else {
        return;
    };

    // Write local state for workspace-driven automation and keep HOME cache
    // for compatibility when commands run from other directories.
    for path in session_paths() {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(&path, &json);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
        }
    }
}

fn load_matching_session(
    admin_url: &str,
    keycloak_url: &str,
    keycloak_client_id: &str,
    username: &str,
) -> Option<SessionFile> {
    let session = read_session_file()?;
    if session.admin_url != admin_url
        || session.keycloak_url != keycloak_url
        || session.keycloak_client_id != keycloak_client_id
        || session.username != username
    {
        return None;
    }
    Some(session)
}

pub fn clear_session() {
    for path in session_paths() {
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
            admin_url: admin_url.clone(),
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

        if let Some(session) = load_matching_session(
            &self.admin_url,
            &self.keycloak_url,
            &self.keycloak_client_id,
            &self.username,
        ) {
            if now_epoch() < session.expires_at {
                write_session_files(&session);
                let remaining = session.expires_at.saturating_sub(now_epoch());
                self.cached = Some(CachedToken {
                    access_token: session.access_token.clone(),
                    expires_at: Instant::now() + Duration::from_secs(remaining),
                });
                return Ok(session.access_token);
            }

            if !session.refresh_token.is_empty() && now_epoch() < session.refresh_expires_at {
                match self.fetch_token_refresh_grant(&session.refresh_token).await {
                    Ok(token_resp) => return self.save_and_cache_token(token_resp),
                    Err((status, body)) => {
                        eprintln!(
                            "Saved refresh token failed ({status}). Falling back to login flow: {body}"
                        );
                        self.invalidate();
                    }
                }
            }
        }

        self.fetch_token().await
    }

    pub fn invalidate(&mut self) {
        self.cached = None;
        clear_session();
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

    async fn fetch_token_refresh_grant(
        &self,
        refresh_token: &str,
    ) -> Result<TokenResponse, (reqwest::StatusCode, String)> {
        let url = format!(
            "{}/realms/internal/protocol/openid-connect/token",
            self.keycloak_url
        );

        let params = [
            ("client_id", self.keycloak_client_id.as_str()),
            ("refresh_token", refresh_token),
            ("grant_type", "refresh_token"),
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
                    format!("Failed to reach Keycloak refresh endpoint: {e}"),
                )
            })?;

        let status = resp.status();
        let body = resp.text().await.map_err(|e| {
            (
                reqwest::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to read Keycloak refresh response: {e}"),
            )
        })?;

        if !status.is_success() {
            return Err((status, body));
        }

        serde_json::from_str(&body).map_err(|e| {
            (
                reqwest::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to parse Keycloak refresh response: {e}"),
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

        let mut cookies = BTreeMap::new();
        let mut next_request =
            NonInteractiveRequest::Get(Url::parse(&auth_url).context("Invalid authorization URL")?);
        const MAX_STEPS: usize = 8;

        for step in 0..MAX_STEPS {
            let mut req = match next_request {
                NonInteractiveRequest::Get(ref url) => browserless_http.get(url.clone()),
                NonInteractiveRequest::Post { ref url, ref form } => {
                    browserless_http.post(url.clone()).form(form)
                }
            };
            if let Some(cookie) = cookie_header_from_jar(&cookies) {
                req = req.header(COOKIE, cookie);
            }

            let resp = req
                .send()
                .await
                .with_context(|| format!("PKCE step {} failed", step + 1))?;

            let resp_url = resp.url().clone();
            let resp_headers = resp.headers().clone();
            let resp_status = resp.status();
            merge_set_cookie_headers(&mut cookies, &resp_headers);

            if resp_status.is_redirection() {
                let location = header_value(&resp_headers, LOCATION)
                    .context("Redirect missing Location header during PKCE flow")?;
                let redirect_url = resolve_url(&resp_url, &location)
                    .context("Invalid redirect URL during PKCE flow")?;
                if let Some(code) = maybe_extract_auth_code(&redirect_url, &state)? {
                    return self.exchange_auth_code(&code, &code_verifier).await;
                }
                next_request = NonInteractiveRequest::Get(redirect_url);
                continue;
            }

            let body = resp
                .text()
                .await
                .context("Failed to read PKCE response body")?;
            if !resp_status.is_success() {
                bail!(
                    "PKCE flow failed ({resp_status}) at step {}: {}",
                    step + 1,
                    trim_for_error(&body)
                );
            }

            if let Some(code) = maybe_extract_auth_code(&resp_url, &state)? {
                return self.exchange_auth_code(&code, &code_verifier).await;
            }

            if let Some(login_form) = extract_login_form(&body) {
                let action_url = resolve_url(&resp_url, &login_form.action)
                    .context("Invalid login form action URL")?;
                let mut form = login_form.hidden_inputs;
                upsert_form_field(&mut form, "username", &self.username);
                // Keycloak accepts this as the submit button name for default login theme.
                upsert_form_field(&mut form, "login", "Sign In");
                next_request = NonInteractiveRequest::Post {
                    url: action_url,
                    form,
                };
                continue;
            }

            bail!(
                "Could not continue non-interactive PKCE flow at step {} ({resp_status}). Response snippet: {}",
                step + 1,
                trim_for_error(&body)
            );
        }

        bail!("Non-interactive PKCE login reached maximum steps ({MAX_STEPS})")
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
            &token_resp.refresh_token,
            token_resp.refresh_expires_in,
            &self.admin_url,
            &self.keycloak_url,
            &self.keycloak_client_id,
            &self.username,
            &self.password,
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

fn maybe_extract_auth_code(callback: &Url, expected_state: &str) -> anyhow::Result<Option<String>> {
    let params = extract_params_from_url(callback);
    if let Some(error) = params.get("error") {
        let description = params
            .get("error_description")
            .map(String::as_str)
            .unwrap_or("no description");
        bail!("Browser login failed: {error} ({description})");
    }
    let Some(code) = params.get("code") else {
        return Ok(None);
    };
    let returned_state = params
        .get("state")
        .context("Missing 'state' parameter in redirected URL")?;
    if returned_state != expected_state {
        bail!("State mismatch in redirected URL");
    }
    Ok(Some(code.to_string()))
}

struct LoginForm {
    action: String,
    hidden_inputs: Vec<(String, String)>,
}

enum NonInteractiveRequest {
    Get(Url),
    Post {
        url: Url,
        form: Vec<(String, String)>,
    },
}

fn extract_login_form(html: &str) -> Option<LoginForm> {
    let mut cursor = 0usize;
    while let Some(form_start_rel) = html[cursor..].find("<form") {
        let form_start = cursor + form_start_rel;
        let open_tag_end = html[form_start..].find('>')?;
        let open_tag = &html[form_start..=form_start + open_tag_end];
        let form_content_start = form_start + open_tag_end + 1;
        let close_rel = html[form_content_start..].find("</form>")?;
        let form_content_end = form_content_start + close_rel;
        let form_content = &html[form_content_start..form_content_end];
        if open_tag.contains("kc-form-login")
            && let Some(action) = extract_html_attr(open_tag, "action")
        {
            return Some(LoginForm {
                action: action.replace("&amp;", "&"),
                hidden_inputs: extract_hidden_inputs(form_content),
            });
        }
        cursor = form_content_end + "</form>".len();
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

fn extract_hidden_inputs(form_html: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut cursor = 0usize;
    while let Some(input_start_rel) = form_html[cursor..].find("<input") {
        let input_start = cursor + input_start_rel;
        let Some(input_end_rel) = form_html[input_start..].find('>') else {
            break;
        };
        let tag = &form_html[input_start..=input_start + input_end_rel];
        cursor = input_start + input_end_rel + 1;
        let Some(name) = extract_html_attr(tag, "name") else {
            continue;
        };
        let input_type = extract_html_attr(tag, "type")
            .unwrap_or_default()
            .to_ascii_lowercase();
        if input_type != "hidden" {
            continue;
        }
        let value = extract_html_attr(tag, "value").unwrap_or_default();
        out.push((name, value));
    }
    out
}

fn upsert_form_field(form: &mut Vec<(String, String)>, key: &str, value: &str) {
    if let Some((_, existing)) = form.iter_mut().find(|(k, _)| k == key) {
        existing.clear();
        existing.push_str(value);
        return;
    }
    form.push((key.to_string(), value.to_string()));
}

fn merge_set_cookie_headers(
    jar: &mut BTreeMap<String, String>,
    headers: &reqwest::header::HeaderMap,
) {
    for value in headers.get_all(SET_COOKIE) {
        let Ok(raw_cookie) = value.to_str() else {
            continue;
        };
        let Some(cookie_pair) = raw_cookie.split(';').next() else {
            continue;
        };
        let Some((name, cookie_value)) = cookie_pair.split_once('=') else {
            continue;
        };
        let name = name.trim();
        if !name.is_empty() {
            jar.insert(name.to_string(), cookie_value.trim().to_string());
        }
    }
}

fn cookie_header_from_jar(jar: &BTreeMap<String, String>) -> Option<String> {
    if jar.is_empty() {
        None
    } else {
        Some(
            jar.iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join("; "),
        )
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
