use std::{
    fs,
    path::PathBuf,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, bail};
use serde::{Deserialize, Serialize};

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
    username: String,
}

pub struct AuthClient {
    keycloak_url: String,
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

fn save_session(token: &str, expires_in: u64, keycloak_url: &str, username: &str) {
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

fn load_session(keycloak_url: &str, username: &str) -> Option<SessionFile> {
    let path = session_path()?;
    let data = fs::read_to_string(path).ok()?;
    let session: SessionFile = serde_json::from_str(&data).ok()?;
    if session.keycloak_url != keycloak_url || session.username != username {
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
    pub fn new(keycloak_url: String, username: String, password: String) -> Self {
        Self {
            keycloak_url,
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

        if let Some(session) = load_session(&self.keycloak_url, &self.username) {
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
        let url = format!(
            "{}/realms/internal/protocol/openid-connect/token",
            self.keycloak_url
        );

        let params = [
            ("client_id", "admin-panel"),
            ("username", &self.username),
            ("password", &self.password),
            ("grant_type", "password"),
            ("scope", "openid profile email"),
        ];

        let resp = self
            .http
            .post(&url)
            .form(&params)
            .send()
            .await
            .context("Failed to reach Keycloak")?;

        let status = resp.status();
        let body = resp
            .text()
            .await
            .context("Failed to read Keycloak response")?;

        if !status.is_success() {
            bail!("Keycloak auth failed ({}): {}", status, body);
        }

        let token_resp: TokenResponse =
            serde_json::from_str(&body).context("Failed to parse Keycloak token response")?;

        let expires_at =
            Instant::now() + Duration::from_secs(token_resp.expires_in.saturating_sub(30));

        save_session(
            &token_resp.access_token,
            token_resp.expires_in,
            &self.keycloak_url,
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
