//! Native import/storage of Google's downloaded Desktop client configuration.
use crate::{
    db::HostDatabase,
    secret_store::{NativeSecretStore, SecretStore},
};
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Read, path::Path};
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

#[derive(Deserialize)]
struct Download {
    installed: GoogleClientConfig,
}

#[derive(Deserialize, Serialize, Zeroize, ZeroizeOnDrop)]
pub struct GoogleClientConfig {
    client_id: String,
    client_secret: String,
}
impl GoogleClientConfig {
    fn validate(&self) -> Result<(), String> {
        if self.client_id != crate::google_oauth::CLIENT_ID {
            return Err("This JSON belongs to a different OAuth client. Download the configured Overlay Forge Desktop client JSON.".into());
        }
        if self.client_secret.is_empty()
            || self.client_secret.len() > 4096
            || self.client_secret.chars().any(char::is_control)
        {
            return Err("Google Desktop client JSON has no valid client secret.".into());
        }
        Ok(())
    }
    pub fn secret(&self) -> &str {
        &self.client_secret
    }
    pub fn import(db: &HostDatabase, path: &Path) -> Result<(), String> {
        let mut bytes = Zeroizing::new(Vec::new());
        File::open(path)
            .map_err(|_| "Could not open the selected Google client JSON.")?
            .take(64 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| "Could not read the selected Google client JSON.")?;
        if bytes.len() > 64 * 1024 {
            return Err("Google client JSON exceeds the import limit.".into());
        }
        let downloaded: Download=serde_json::from_slice(&bytes)
            .map_err(|_| "Select Google's downloaded Desktop client JSON with an installed configuration, not a Web client or token file.")?;
        downloaded.installed.validate()?;
        let plain = Zeroizing::new(
            serde_json::to_vec(&downloaded.installed)
                .map_err(|_| "Could not prepare Google client storage.")?,
        );
        let host = db
            .ai_host_id()
            .map_err(|_| "Could not resolve protected storage identity.")?;
        let protected = NativeSecretStore
            .protect(&host, &plain)
            .map_err(|error| error.to_string())?;
        db.save_google_client_config(&protected)
            .map_err(|_| "Could not save protected Google client configuration.".into())
    }
    pub fn load(db: &HostDatabase) -> Result<Self, String> {
        let protected = db
            .google_client_config()
            .map_err(|_| "Could not read Google client configuration.")?
            .ok_or("Import the Google Desktop client JSON before signing in.")?;
        let host = db
            .ai_host_id()
            .map_err(|_| "Could not resolve protected storage identity.")?;
        let plain = NativeSecretStore
            .unprotect(&host, &protected)
            .map_err(|error| error.to_string())?;
        let config: Self = serde_json::from_slice(&plain).map_err(|_| {
            "Protected Google client configuration is invalid. Reimport the Desktop client JSON from the sign-in screen."
        })?;
        config.validate()?;
        Ok(config)
    }
}
