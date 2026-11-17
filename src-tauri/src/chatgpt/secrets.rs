//! ChatGPT compatibility facade over the host-only platform secret boundary.
use crate::secret_store::{NativeSecretStore, SecretStore};
use zeroize::Zeroizing;

pub fn protect(host_id: &str, bytes: &[u8]) -> Result<Zeroizing<Vec<u8>>, String> {
    NativeSecretStore
        .protect(host_id, bytes)
        .map_err(|error| error.to_string())
}
pub fn unprotect(host_id: &str, bytes: &[u8]) -> Result<Zeroizing<Vec<u8>>, String> {
    NativeSecretStore
        .unprotect(host_id, bytes)
        .map_err(|error| error.to_string())
}
