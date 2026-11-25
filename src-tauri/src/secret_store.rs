//! Host-only secret protection. Module contracts must never expose this interface.
//! DPAPI records retain their existing representation; Linux records are versioned
//! AES-GCM envelopes with their key held separately in an unlocked Secret Service.
use std::fmt;
use zeroize::Zeroizing;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SecretStoreError {
    Unavailable,
    #[cfg(any(target_os = "linux", test))]
    Locked,
    #[cfg(any(target_os = "linux", test))]
    MissingKey,
    InvalidRecord,
    #[cfg(not(any(windows, target_os = "linux")))]
    UnsupportedPlatform,
}
impl fmt::Display for SecretStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Unavailable => "Protected credential storage is unavailable. No plaintext fallback is permitted.",
            #[cfg(any(target_os = "linux", test))]
            Self::Locked => "Protected credential storage is locked. Unlock it in your desktop keyring and retry.",
            #[cfg(any(target_os = "linux", test))]
            Self::MissingKey => "The credential encryption key is missing. Restore the matching keyring or reconnect; no replacement key was created.",
            Self::InvalidRecord => "Protected credential data could not be authenticated for this host.",
            #[cfg(not(any(windows, target_os = "linux")))]
            Self::UnsupportedPlatform => "Protected credential storage is unsupported on this platform.",
        })
    }
}
impl std::error::Error for SecretStoreError {}

pub trait SecretStore {
    fn protect(&self, host_id: &str, plain: &[u8]) -> Result<Zeroizing<Vec<u8>>, SecretStoreError>;
    fn unprotect(
        &self,
        host_id: &str,
        record: &[u8],
    ) -> Result<Zeroizing<Vec<u8>>, SecretStoreError>;
}

pub struct NativeSecretStore;
impl SecretStore for NativeSecretStore {
    fn protect(&self, host_id: &str, plain: &[u8]) -> Result<Zeroizing<Vec<u8>>, SecretStoreError> {
        validate_host_id(host_id)?;
        platform::protect(host_id, plain)
    }
    fn unprotect(
        &self,
        host_id: &str,
        record: &[u8],
    ) -> Result<Zeroizing<Vec<u8>>, SecretStoreError> {
        validate_host_id(host_id)?;
        platform::unprotect(host_id, record)
    }
}
fn validate_host_id(host_id: &str) -> Result<(), SecretStoreError> {
    let value = host_id
        .strip_prefix("urn:uuid:")
        .ok_or(SecretStoreError::InvalidRecord)?;
    uuid::Uuid::parse_str(value).map_err(|_| SecretStoreError::InvalidRecord)?;
    Ok(())
}

#[cfg(windows)]
mod platform {
    use super::*;
    use zeroize::Zeroize;
    pub fn protect(_: &str, bytes: &[u8]) -> Result<Zeroizing<Vec<u8>>, SecretStoreError> {
        crypt(bytes, true)
    }
    pub fn unprotect(_: &str, bytes: &[u8]) -> Result<Zeroizing<Vec<u8>>, SecretStoreError> {
        crypt(bytes, false)
    }
    fn crypt(bytes: &[u8], protect: bool) -> Result<Zeroizing<Vec<u8>>, SecretStoreError> {
        use std::ptr::{null, null_mut};
        use windows_sys::Win32::{
            Foundation::LocalFree,
            Security::Cryptography::{
                CryptProtectData, CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
            },
        };
        let input = CRYPT_INTEGER_BLOB {
            cbData: u32::try_from(bytes.len()).map_err(|_| SecretStoreError::InvalidRecord)?,
            pbData: bytes.as_ptr() as *mut u8,
        };
        let mut output = CRYPT_INTEGER_BLOB {
            cbData: 0,
            pbData: null_mut(),
        };
        let ok = unsafe {
            if protect {
                CryptProtectData(
                    &input,
                    null(),
                    null(),
                    null(),
                    null(),
                    CRYPTPROTECT_UI_FORBIDDEN,
                    &mut output,
                )
            } else {
                CryptUnprotectData(
                    &input,
                    null_mut(),
                    null(),
                    null(),
                    null(),
                    CRYPTPROTECT_UI_FORBIDDEN,
                    &mut output,
                )
            }
        };
        if ok == 0 {
            return Err(SecretStoreError::Unavailable);
        }
        // DPAPI allocates output; copy, erase and free its allocation.
        let copied = unsafe {
            let bytes = std::slice::from_raw_parts_mut(output.pbData, output.cbData as usize);
            let copied = Zeroizing::new(bytes.to_vec());
            bytes.zeroize();
            LocalFree(output.pbData as *mut _);
            copied
        };
        Ok(copied)
    }
}

#[cfg(any(target_os = "linux", test))]
mod envelope {
    use super::*;
    use aes_gcm::{
        aead::{Aead, Payload},
        Aes256Gcm, KeyInit, Nonce,
    };
    use rand::RngCore;
    const HEADER: &[u8] = b"OFS-LINUX-AESGCM\0\x01";
    const NONCE_LEN: usize = 12;
    const TAG_LEN: usize = 16;
    pub trait KeyProvider {
        fn key(&self, host_id: &str, create: bool) -> Result<Zeroizing<Vec<u8>>, SecretStoreError>;
    }
    // Used only by the host. Providers may create a missing key only for a new
    // write; decrypting old records must never initialize or replace a lost key.
    pub fn protect(
        provider: &impl KeyProvider,
        host_id: &str,
        plain: &[u8],
    ) -> Result<Zeroizing<Vec<u8>>, SecretStoreError> {
        let key = provider.key(host_id, true)?;
        let cipher =
            Aes256Gcm::new_from_slice(&key).map_err(|_| SecretStoreError::InvalidRecord)?;
        let mut nonce = [0u8; NONCE_LEN];
        rand::rng().fill_bytes(&mut nonce);
        let ciphertext = cipher
            .encrypt(
                Nonce::from_slice(&nonce),
                Payload {
                    msg: plain,
                    aad: host_id.as_bytes(),
                },
            )
            .map_err(|_| SecretStoreError::InvalidRecord)?;
        let mut record = Zeroizing::new(Vec::with_capacity(
            HEADER.len() + NONCE_LEN + ciphertext.len(),
        ));
        record.extend_from_slice(HEADER);
        record.extend_from_slice(&nonce);
        record.extend_from_slice(&ciphertext);
        Ok(record)
    }
    pub fn unprotect(
        provider: &impl KeyProvider,
        host_id: &str,
        record: &[u8],
    ) -> Result<Zeroizing<Vec<u8>>, SecretStoreError> {
        if !record.starts_with(HEADER) || record.len() < HEADER.len() + NONCE_LEN + TAG_LEN {
            return Err(SecretStoreError::InvalidRecord);
        }
        let key = provider.key(host_id, false)?;
        let cipher =
            Aes256Gcm::new_from_slice(&key).map_err(|_| SecretStoreError::InvalidRecord)?;
        let nonce_end = HEADER.len() + NONCE_LEN;
        let plain = cipher
            .decrypt(
                Nonce::from_slice(&record[HEADER.len()..nonce_end]),
                Payload {
                    msg: &record[nonce_end..],
                    aad: host_id.as_bytes(),
                },
            )
            .map_err(|_| SecretStoreError::InvalidRecord)?;
        Ok(Zeroizing::new(plain))
    }
}

#[cfg(target_os = "linux")]
mod platform {
    use super::envelope::{self, KeyProvider};
    use super::*;
    use rand::RngCore;
    use secret_service::{blocking::SecretService, EncryptionType};
    use std::{collections::HashMap, sync::Mutex};
    // The host is single-instance. Serialize first-key creation and reads locally;
    // duplicate matching entries from external writers are rejected, never chosen.
    static KEY_ACCESS: Mutex<()> = Mutex::new(());
    struct DesktopKeyring;
    impl KeyProvider for DesktopKeyring {
        fn key(&self, host_id: &str, create: bool) -> Result<Zeroizing<Vec<u8>>, SecretStoreError> {
            let _guard = KEY_ACCESS
                .lock()
                .map_err(|_| SecretStoreError::Unavailable)?;
            let service = SecretService::connect(EncryptionType::Dh)
                .map_err(|_| SecretStoreError::Unavailable)?;
            let collection = service
                .get_default_collection()
                .map_err(|_| SecretStoreError::Unavailable)?;
            if collection
                .is_locked()
                .map_err(|_| SecretStoreError::Unavailable)?
            {
                return Err(SecretStoreError::Locked);
            }
            // Lookup metadata contains only stable opaque identity, never email or
            // account names. Secret Service attributes are not confidential.
            let attributes = HashMap::from([
                ("application", "com.overlayforge.desktop"),
                ("purpose", "credential-envelope-key-v1"),
                ("host-id", host_id),
            ]);
            let mut items = collection
                .search_items(attributes.clone())
                .map_err(|_| SecretStoreError::Unavailable)?;
            if items.is_empty() && create {
                let mut key = Zeroizing::new(vec![0u8; 32]);
                rand::rng().fill_bytes(&mut key);
                // Never overwrite an existing key. Detect ambiguous external
                // concurrent creation on the subsequent read and fail closed.
                collection
                    .create_item(
                        "Overlay Forge credential encryption key",
                        attributes.clone(),
                        &key,
                        false,
                        "application/octet-stream",
                    )
                    .map_err(|_| SecretStoreError::Unavailable)?;
                items = collection
                    .search_items(attributes)
                    .map_err(|_| SecretStoreError::Unavailable)?;
            }
            match items.len() {
                0 => Err(SecretStoreError::MissingKey),
                1 => {
                    if items[0]
                        .is_locked()
                        .map_err(|_| SecretStoreError::Unavailable)?
                    {
                        return Err(SecretStoreError::Locked);
                    }
                    let key = Zeroizing::new(
                        items[0]
                            .get_secret()
                            .map_err(|_| SecretStoreError::Unavailable)?,
                    );
                    if key.len() != 32 {
                        return Err(SecretStoreError::InvalidRecord);
                    }
                    Ok(key)
                }
                _ => Err(SecretStoreError::InvalidRecord),
            }
        }
    }
    pub fn protect(host_id: &str, bytes: &[u8]) -> Result<Zeroizing<Vec<u8>>, SecretStoreError> {
        envelope::protect(&DesktopKeyring, host_id, bytes)
    }
    pub fn unprotect(host_id: &str, bytes: &[u8]) -> Result<Zeroizing<Vec<u8>>, SecretStoreError> {
        envelope::unprotect(&DesktopKeyring, host_id, bytes)
    }
}

#[cfg(not(any(windows, target_os = "linux")))]
mod platform {
    use super::*;
    pub fn protect(_: &str, _: &[u8]) -> Result<Zeroizing<Vec<u8>>, SecretStoreError> {
        Err(SecretStoreError::UnsupportedPlatform)
    }
    pub fn unprotect(_: &str, _: &[u8]) -> Result<Zeroizing<Vec<u8>>, SecretStoreError> {
        Err(SecretStoreError::UnsupportedPlatform)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    const HOST: &str = "urn:uuid:6144696e-e26c-4609-88d1-4c4e308e0495";
    struct Fixture {
        error: Option<SecretStoreError>,
        creates: Cell<usize>,
    }
    impl envelope::KeyProvider for Fixture {
        fn key(&self, _: &str, create: bool) -> Result<Zeroizing<Vec<u8>>, SecretStoreError> {
            if create {
                self.creates.set(self.creates.get() + 1);
            }
            match self.error {
                Some(error) => Err(error),
                None => Ok(Zeroizing::new(vec![42u8; 32])),
            }
        }
    }
    #[test]
    fn envelope_authenticates_identity_and_rejects_corruption() {
        let provider = Fixture {
            error: None,
            creates: Cell::new(0),
        };
        let plain = b"synthetic-only-secret";
        let mut record = envelope::protect(&provider, HOST, plain).unwrap();
        assert!(!record.windows(plain.len()).any(|bytes| bytes == plain));
        assert_eq!(
            &*envelope::unprotect(&provider, HOST, &record).unwrap(),
            plain
        );
        assert_eq!(
            envelope::unprotect(&provider, "another-host", &record).unwrap_err(),
            SecretStoreError::InvalidRecord
        );
        let last = record.len() - 1;
        record[last] ^= 1;
        assert_eq!(
            envelope::unprotect(&provider, HOST, &record).unwrap_err(),
            SecretStoreError::InvalidRecord
        );
        for length in 0..record.len() {
            assert!(envelope::unprotect(&provider, HOST, &record[..length]).is_err());
        }
    }
    #[test]
    fn locked_unavailable_and_missing_keys_fail_closed_without_creation_on_read() {
        let available = Fixture {
            error: None,
            creates: Cell::new(0),
        };
        let record = envelope::protect(&available, HOST, b"synthetic").unwrap();
        for error in [
            SecretStoreError::Locked,
            SecretStoreError::Unavailable,
            SecretStoreError::MissingKey,
        ] {
            let provider = Fixture {
                error: Some(error),
                creates: Cell::new(0),
            };
            assert_eq!(
                envelope::unprotect(&provider, HOST, &record).unwrap_err(),
                error
            );
            assert_eq!(provider.creates.get(), 0);
            assert_eq!(
                envelope::protect(&provider, HOST, b"synthetic").unwrap_err(),
                error
            );
        }
    }
    #[test]
    fn invalid_identity_and_foreign_format_are_rejected() {
        assert!(validate_host_id(HOST).is_ok());
        assert!(validate_host_id("account@example.com").is_err());
        let provider = Fixture {
            error: None,
            creates: Cell::new(0),
        };
        assert_eq!(
            envelope::unprotect(&provider, HOST, b"foreign-platform-record").unwrap_err(),
            SecretStoreError::InvalidRecord
        );
        assert_eq!(provider.creates.get(), 0);
    }
    #[cfg(windows)]
    #[test]
    fn dpapi_round_trip_and_tamper_rejection() {
        let encrypted = NativeSecretStore
            .protect(HOST, b"synthetic-test-value-not-a-credential")
            .unwrap();
        assert_ne!(&*encrypted, b"synthetic-test-value-not-a-credential");
        assert_eq!(
            &*NativeSecretStore.unprotect(HOST, &encrypted).unwrap(),
            b"synthetic-test-value-not-a-credential"
        );
        let mut invalid = encrypted.to_vec();
        let last = invalid.len() - 1;
        invalid[last] ^= 1;
        assert!(NativeSecretStore.unprotect(HOST, &invalid).is_err());
    }
}
