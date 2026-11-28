use reqwest::Url;
use serde::Deserialize;
use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
};
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub public_origin: String,
    pub google_client_file: PathBuf,
    pub data_dir: PathBuf,
    pub frontend_dir: PathBuf,
    pub media_database: Option<PathBuf>,
    pub local_llm_owner_email: Option<String>,
    #[serde(default = "default_bind")]
    pub bind: SocketAddr,
    #[serde(default)]
    pub allow_http_localhost: bool,
}
fn default_bind() -> SocketAddr {
    ([127, 0, 0, 1], 4280).into()
}

#[derive(Deserialize)]
struct ClientDownload {
    web: GoogleClient,
}
#[derive(Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct GoogleClient {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uris: Vec<String>,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, String> {
        let bytes = std::fs::read(path).map_err(|_| "Could not read server configuration.")?;
        if bytes.len() > 64 * 1024 {
            return Err("Server configuration is too large.".into());
        }
        let mut config: Self =
            serde_json::from_slice(&bytes).map_err(|_| "Invalid server configuration JSON.")?;
        let url = Url::parse(&config.public_origin).map_err(|_| "Invalid public_origin.")?;
        let local = matches!(
            url.host_str(),
            Some("localhost" | "127.0.0.1" | "[::1]" | "::1")
        );
        if !(url.scheme() == "https"
            || (url.scheme() == "http" && local && config.allow_http_localhost))
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || !matches!(url.path(), "" | "/")
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return Err("public_origin must be one HTTPS origin; HTTP is allowed only for explicit localhost development.".into());
        }
        if !config.bind.ip().is_loopback() {
            return Err("Bind the server to loopback behind your HTTPS reverse proxy.".into());
        }
        config.public_origin = url.origin().ascii_serialization();
        if let Some(email) = &mut config.local_llm_owner_email {
            *email = crate::repository::normalize_email(email)
                .map_err(|_| "local_llm_owner_email must be a complete email address.")?;
        }
        let base = path.parent().unwrap_or(Path::new("."));
        for value in [
            &mut config.google_client_file,
            &mut config.data_dir,
            &mut config.frontend_dir,
        ] {
            if value.is_relative() {
                *value = base.join(&*value);
            }
        }
        if let Some(database) = &mut config.media_database {
            if database.is_relative() {
                *database = base.join(&*database);
            }
        }
        Ok(config)
    }
    pub fn google_client(&self) -> Result<GoogleClient, String> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = std::fs::metadata(&self.google_client_file)
                .map_err(|_| "Could not inspect Google Web client JSON.")?;
            if metadata.permissions().mode() & 0o077 != 0 {
                return Err(
                    "Google Web client JSON must be readable only by its owner (chmod 600).".into(),
                );
            }
        }
        let bytes = Zeroizing::new(
            std::fs::read(&self.google_client_file)
                .map_err(|_| "Could not read Google Web OAuth client JSON.")?,
        );
        if bytes.len() > 64 * 1024 {
            return Err("Google client JSON is too large.".into());
        }
        let client: ClientDownload = serde_json::from_slice(&bytes)
            .map_err(|_| "Use Google's Web application client JSON, not the Desktop client.")?;
        let client = client.web;
        if !client.client_id.ends_with(".apps.googleusercontent.com")
            || client.client_id.len() > 256
            || client.client_id.chars().any(char::is_control)
            || client.client_secret.is_empty()
            || client.client_secret.len() > 4096
            || client.client_secret.chars().any(char::is_control)
            || !client
                .redirect_uris
                .contains(&format!("{}/auth/callback", self.public_origin))
        {
            return Err("Google Web client must contain this server's exact /auth/callback redirect URI and valid client credentials.".into());
        }
        Ok(client)
    }
    pub fn secure(&self) -> bool {
        self.public_origin.starts_with("https://")
    }
    pub fn session_cookie(&self) -> &'static str {
        if self.secure() {
            "__Host-of-session"
        } else {
            "of-dev-session"
        }
    }
    pub fn login_cookie(&self) -> &'static str {
        if self.secure() {
            "__Host-of-login"
        } else {
            "of-dev-login"
        }
    }
    pub fn media_path(&self) -> PathBuf {
        self.media_database
            .clone()
            .unwrap_or_else(|| self.data_dir.join("overlay-forge-media.sqlite3"))
    }
    pub fn expected_host(&self) -> String {
        self.public_origin
            .split("://")
            .nth(1)
            .unwrap_or_default()
            .to_string()
    }
}
