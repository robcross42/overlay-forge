use crate::db::{
    retirement_protected_id, AppDatabase, RetirementPlanningProfileRecord, RetirementProtectedRecord,
};
use aes_gcm::{aead::{Aead, KeyInit, Payload}, Aes256Gcm};
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine};
use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use zeroize::{Zeroize, Zeroizing};

const KEYRING_SERVICE: &str = "com.overlayforge.desktop.retirement";
const KEYRING_ACCOUNT: &str = "master-key-v1";
const PROFILE_ID: &str = "profile-primary";
const PAYLOAD_VERSION: i64 = 1;

pub type RetirementSessionKey = Mutex<Option<Zeroizing<[u8; 32]>>>;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtectedStoreStatus {
    pub state: String,
    pub message: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetirementProfile {
    pub display_label: String,
    pub age: Option<i64>,
    pub target_age: Option<i64>,
    pub retirement_definition: String,
    pub notes: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetirementFinancialRecord {
    pub id: String,
    pub entity_type: String,
    pub kind: String,
    pub label: String,
    pub institution: String,
    pub amount_cents: i64,
    pub as_of_date: String,
    pub interest_rate_basis_points: Option<i64>,
    pub minimum_payment_cents: Option<i64>,
    pub cadence: String,
    pub expected_change_date: String,
    pub expected_change_amount_cents: Option<i64>,
    pub notes: String,
    pub is_archived: bool,
    pub created_at: String,
    pub modified_at: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetirementProfileInput {
    pub display_label: String,
    pub age: Option<i64>,
    pub target_age: Option<i64>,
    pub retirement_definition: String,
    pub notes: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetirementFinancialRecordInput {
    pub id: Option<String>,
    pub entity_type: String,
    pub kind: String,
    pub label: String,
    pub institution: Option<String>,
    pub amount_cents: i64,
    pub as_of_date: String,
    pub interest_rate_basis_points: Option<i64>,
    pub minimum_payment_cents: Option<i64>,
    pub cadence: Option<String>,
    pub expected_change_date: Option<String>,
    pub expected_change_amount_cents: Option<i64>,
    pub notes: Option<String>,
}

pub fn protected_store_status(
    database: &AppDatabase,
    session_key: &RetirementSessionKey,
) -> Result<ProtectedStoreStatus, String> {
    let store = database
        .get_retirement_secure_store()
        .map_err(|_| "Protected retirement data is unavailable.".to_string())?;
    if store.state == "uninitialized" {
        return Ok(ProtectedStoreStatus {
            state: "uninitialized".to_string(),
            message: "Enable protected retirement data to store planning information locally on this device.".to_string(),
        });
    }
    let unlocked = session_key
        .lock()
        .map_err(|_| "Protected retirement data is unavailable.".to_string())?
        .is_some();
    Ok(ProtectedStoreStatus {
        state: if unlocked { "unlocked" } else { "locked" }.to_string(),
        message: if unlocked {
            "Protected retirement data is unlocked for this app session.".to_string()
        } else {
            "Unlock protected retirement data to view or edit it.".to_string()
        },
    })
}

pub fn initialize(
    database: &AppDatabase,
    session_key: &RetirementSessionKey,
) -> Result<ProtectedStoreStatus, String> {
    if database
        .get_retirement_secure_store()
        .map_err(|_| "Protected retirement data is unavailable.".to_string())?
        .state
        != "uninitialized"
    {
        return Err("Protected retirement data is already initialized. Unlock it instead.".to_string());
    }
    let entry = keyring_entry()?;
    match entry.get_password() {
        Ok(_) => return Err("Protected retirement data is already initialized. Unlock it instead.".to_string()),
        Err(keyring::Error::NoEntry) => {}
        Err(_) => return Err("Secure storage is unavailable. Protected retirement data was not changed.".to_string()),
    }
    let mut key = [0_u8; 32];
    getrandom::fill(&mut key)
        .map_err(|_| "Secure storage is unavailable. Protected retirement data was not changed.".to_string())?;
    entry
        .set_password(&BASE64_STANDARD.encode(key))
        .map_err(|_| "Secure storage is unavailable. Protected retirement data was not changed.".to_string())?;
    set_session_key(session_key, key)?;
    database
        .mark_retirement_secure_store_initialized()
        .map_err(|_| "Protected retirement data is unavailable.".to_string())?;
    migrate_legacy_profile(database, session_key)?;
    protected_store_status(database, session_key)
}

pub fn unlock(
    database: &AppDatabase,
    session_key: &RetirementSessionKey,
) -> Result<ProtectedStoreStatus, String> {
    let store = database
        .get_retirement_secure_store()
        .map_err(|_| "Protected retirement data is unavailable.".to_string())?;
    if store.state == "uninitialized" {
        return Err("Protected retirement data is not initialized.".to_string());
    }
    let password = keyring_entry()?
        .get_password()
        .map_err(|_| "Secure storage is unavailable. Protected retirement data remains locked.".to_string())?;
    let bytes = BASE64_STANDARD
        .decode(password)
        .map_err(|_| "Protected retirement data could not be authenticated.".to_string())?;
    let key: [u8; 32] = bytes
        .try_into()
        .map_err(|_| "Protected retirement data could not be authenticated.".to_string())?;
    set_session_key(session_key, key)?;
    migrate_legacy_profile(database, session_key)?;
    protected_store_status(database, session_key)
}

pub fn lock(database: &AppDatabase, session_key: &RetirementSessionKey) -> Result<ProtectedStoreStatus, String> {
    *session_key
        .lock()
        .map_err(|_| "Protected retirement data is unavailable.".to_string())? = None;
    protected_store_status(database, session_key)
}

pub fn get_profile(database: &AppDatabase, session_key: &RetirementSessionKey) -> Result<RetirementProfile, String> {
    let record = database
        .get_retirement_protected_record(PROFILE_ID, "profile")
        .map_err(|_| "Protected retirement data is unavailable.".to_string())?
        .ok_or_else(|| "Protected retirement data is unavailable.".to_string())?;
    decrypt_record(session_key, &record)
}

pub fn save_profile(
    database: &AppDatabase,
    session_key: &RetirementSessionKey,
    input: RetirementProfileInput,
) -> Result<RetirementProfile, String> {
    let profile = RetirementProfile {
        display_label: bounded_text(&input.display_label, 120, "Profile label")?,
        age: optional_age(input.age)?,
        target_age: optional_age(input.target_age)?,
        retirement_definition: bounded_text(&input.retirement_definition, 2_000, "Retirement definition")?,
        notes: bounded_text(&input.notes, 8_000, "Notes")?,
    };
    if let (Some(age), Some(target_age)) = (profile.age, profile.target_age) {
        if target_age < age {
            return Err("Target age cannot be earlier than current age.".to_string());
        }
    }
    encrypt_and_store(database, session_key, PROFILE_ID, "profile", &profile, false)?;
    Ok(profile)
}

pub fn list_financial_records(
    database: &AppDatabase,
    session_key: &RetirementSessionKey,
    entity_type: &str,
) -> Result<Vec<RetirementFinancialRecord>, String> {
    validate_entity_type(entity_type)?;
    database
        .list_retirement_protected_records(entity_type, false)
        .map_err(|_| "Protected retirement data is unavailable.".to_string())?
        .iter()
        .map(|record| decrypt_record(session_key, record))
        .collect()
}

pub fn save_financial_record(
    database: &AppDatabase,
    session_key: &RetirementSessionKey,
    input: RetirementFinancialRecordInput,
) -> Result<RetirementFinancialRecord, String> {
    validate_entity_type(&input.entity_type)?;
    validate_financial_kind(&input.entity_type, &input.kind)?;
    let id = input
        .id
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| retirement_protected_id(&input.entity_type));
    let record = RetirementFinancialRecord {
        id: id.clone(),
        entity_type: input.entity_type.clone(),
        kind: input.kind,
        label: required_text(&input.label, 160, "Label")?,
        institution: bounded_text(input.institution.as_deref().unwrap_or_default(), 160, "Institution")?,
        amount_cents: non_negative_money(input.amount_cents, "Amount")?,
        as_of_date: valid_iso_date(&input.as_of_date, "As-of date")?,
        interest_rate_basis_points: optional_non_negative(input.interest_rate_basis_points, "Interest rate")?,
        minimum_payment_cents: optional_non_negative(input.minimum_payment_cents, "Minimum payment")?,
        cadence: validate_cadence(input.cadence.as_deref().unwrap_or("monthly"))?,
        expected_change_date: optional_iso_date(input.expected_change_date.as_deref().unwrap_or_default(), "Expected change date")?,
        expected_change_amount_cents: optional_non_negative(input.expected_change_amount_cents, "Expected change amount")?,
        notes: bounded_text(input.notes.as_deref().unwrap_or_default(), 8_000, "Notes")?,
        is_archived: false,
        created_at: String::new(),
        modified_at: String::new(),
    };
    if let Some(existing) = database
        .get_retirement_protected_record(&id, &record.entity_type)
        .map_err(|_| "Protected retirement data is unavailable.".to_string())?
    {
        let prior: RetirementFinancialRecord = decrypt_record(session_key, &existing)?;
        let history_id = retirement_protected_id("financial-history");
        encrypt_and_store(database, session_key, &history_id, "financial_history", &prior, false)?;
    }
    encrypt_and_store(database, session_key, &id, &record.entity_type, &record, false)?;
    get_financial_record(database, session_key, &id, &record.entity_type)
}

pub fn archive_financial_record(
    database: &AppDatabase,
    session_key: &RetirementSessionKey,
    id: &str,
    entity_type: &str,
) -> Result<(), String> {
    validate_entity_type(entity_type)?;
    let existing = database
        .get_retirement_protected_record(id, entity_type)
        .map_err(|_| "Protected retirement data is unavailable.".to_string())?
        .ok_or_else(|| "Protected retirement record was not found.".to_string())?;
    let mut record: RetirementFinancialRecord = decrypt_record(session_key, &existing)?;
    record.is_archived = true;
    encrypt_and_store(database, session_key, id, entity_type, &record, true)
}

fn get_financial_record(
    database: &AppDatabase,
    session_key: &RetirementSessionKey,
    id: &str,
    entity_type: &str,
) -> Result<RetirementFinancialRecord, String> {
    let record = database
        .get_retirement_protected_record(id, entity_type)
        .map_err(|_| "Protected retirement data is unavailable.".to_string())?
        .ok_or_else(|| "Protected retirement record was not found.".to_string())?;
    decrypt_record(session_key, &record)
}

fn migrate_legacy_profile(database: &AppDatabase, session_key: &RetirementSessionKey) -> Result<(), String> {
    if database
        .get_retirement_protected_record(PROFILE_ID, "profile")
        .map_err(|_| "Protected retirement data is unavailable.".to_string())?
        .is_none()
    {
        let legacy: RetirementPlanningProfileRecord = database
            .get_retirement_planning_profile()
            .map_err(|_| "Protected retirement data is unavailable.".to_string())?;
        let profile = RetirementProfile {
            display_label: legacy.name,
            age: None,
            target_age: None,
            retirement_definition: legacy.retirement_definition,
            notes: String::new(),
        };
        let (nonce, ciphertext) = encrypt_value(session_key, PROFILE_ID, "profile", &profile)?;
        database
            .migrate_legacy_retirement_profile(PAYLOAD_VERSION, nonce.as_slice(), &ciphertext)
            .map_err(|_| "Protected retirement data is unavailable.".to_string())?;
        return Ok(());
    }
    database
        .mark_retirement_secure_store_migrated()
        .map_err(|_| "Protected retirement data is unavailable.".to_string())
}

fn encrypt_and_store<T: Serialize>(
    database: &AppDatabase,
    session_key: &RetirementSessionKey,
    id: &str,
    entity_type: &str,
    value: &T,
    is_archived: bool,
) -> Result<(), String> {
    let (nonce, ciphertext) = encrypt_value(session_key, id, entity_type, value)?;
    database
        .upsert_retirement_protected_record(
            id,
            entity_type,
            PAYLOAD_VERSION,
            nonce.as_slice(),
            &ciphertext,
            is_archived,
        )
        .map_err(|_| "Protected retirement data is unavailable.".to_string())
}

fn encrypt_value<T: Serialize>(
    session_key: &RetirementSessionKey,
    id: &str,
    entity_type: &str,
    value: &T,
) -> Result<(Vec<u8>, Vec<u8>), String> {
    let mut plaintext = serde_json::to_vec(value)
        .map_err(|_| "Protected retirement data is unavailable.".to_string())?;
    let key = session_key_copy(session_key)?;
    let cipher = Aes256Gcm::new_from_slice(key.as_ref())
        .map_err(|_| "Protected retirement data is unavailable.".to_string())?;
    let mut nonce_bytes = [0_u8; 12];
    getrandom::fill(&mut nonce_bytes)
        .map_err(|_| "Protected retirement data is unavailable.".to_string())?;
    let nonce = aes_gcm::Nonce::from(nonce_bytes);
    let associated_data = aad(entity_type, id);
    let ciphertext = cipher
        .encrypt(
            &nonce,
            Payload {
                msg: plaintext.as_ref(),
                aad: associated_data.as_bytes(),
            },
        )
        .map_err(|_| "Protected retirement data could not be authenticated.".to_string())?;
    plaintext.zeroize();
    Ok((nonce.as_slice().to_vec(), ciphertext))
}

fn decrypt_record<T: for<'de> Deserialize<'de>>(
    session_key: &RetirementSessionKey,
    record: &RetirementProtectedRecord,
) -> Result<T, String> {
    if record.payload_version != PAYLOAD_VERSION {
        return Err("Protected retirement data could not be authenticated.".to_string());
    }
    let key = session_key_copy(session_key)?;
    let cipher = Aes256Gcm::new_from_slice(key.as_ref())
        .map_err(|_| "Protected retirement data is unavailable.".to_string())?;
    let nonce = aes_gcm::Nonce::try_from(record.nonce.as_slice())
        .map_err(|_| "Protected retirement data could not be authenticated.".to_string())?;
    let associated_data = aad(&record.entity_type, &record.id);
    let mut plaintext = cipher
        .decrypt(
            &nonce,
            Payload {
                msg: record.ciphertext.as_ref(),
                aad: associated_data.as_bytes(),
            },
        )
        .map_err(|_| "Protected retirement data could not be authenticated.".to_string())?;
    let parsed = serde_json::from_slice(&plaintext)
        .map_err(|_| "Protected retirement data could not be authenticated.".to_string())?;
    plaintext.zeroize();
    Ok(parsed)
}

fn session_key_copy(session_key: &RetirementSessionKey) -> Result<Zeroizing<[u8; 32]>, String> {
    let guard = session_key
        .lock()
        .map_err(|_| "Protected retirement data is unavailable.".to_string())?;
    guard
        .as_ref()
        .map(|key| Zeroizing::new(**key))
        .ok_or_else(|| "Retirement data is locked.".to_string())
}

fn set_session_key(session_key: &RetirementSessionKey, key: [u8; 32]) -> Result<(), String> {
    *session_key
        .lock()
        .map_err(|_| "Protected retirement data is unavailable.".to_string())? =
        Some(Zeroizing::new(key));
    Ok(())
}

fn keyring_entry() -> Result<Entry, String> {
    Entry::new(KEYRING_SERVICE, KEYRING_ACCOUNT)
        .map_err(|_| "Secure storage is unavailable. Protected retirement data was not changed.".to_string())
}

fn aad(entity_type: &str, id: &str) -> String {
    format!("overlay-forge:retirement:v1:{entity_type}:{id}")
}

fn validate_entity_type(value: &str) -> Result<(), String> {
    match value {
        "account" | "debt" | "income" => Ok(()),
        _ => Err("Unsupported protected retirement record type.".to_string()),
    }
}

fn validate_financial_kind(entity_type: &str, kind: &str) -> Result<(), String> {
    let allowed = match entity_type {
        "account" => ["rrsp", "tfsa", "cash", "taxable_investment", "other"].as_slice(),
        "debt" => ["mortgage", "line_of_credit", "credit_card", "vehicle_loan", "other"].as_slice(),
        "income" => [
            "employment_salary",
            "employer_match",
            "personal_rrsp_contribution",
            "tfsa_contribution",
            "other",
        ]
        .as_slice(),
        _ => return Err("Unsupported protected retirement record type.".to_string()),
    };
    if allowed.contains(&kind) {
        Ok(())
    } else {
        Err("Unsupported retirement record kind.".to_string())
    }
}

fn validate_cadence(value: &str) -> Result<String, String> {
    match value {
        "weekly" | "biweekly" | "monthly" | "annual" => Ok(value.to_string()),
        _ => Err("Cadence must be weekly, biweekly, monthly, or annual.".to_string()),
    }
}

fn required_text(value: &str, limit: usize, label: &str) -> Result<String, String> {
    let trimmed = bounded_text(value, limit, label)?;
    if trimmed.is_empty() {
        Err(format!("{label} is required."))
    } else {
        Ok(trimmed)
    }
}

fn bounded_text(value: &str, limit: usize, label: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.chars().count() > limit {
        Err(format!("{label} is too long."))
    } else {
        Ok(trimmed.to_string())
    }
}

fn non_negative_money(value: i64, label: &str) -> Result<i64, String> {
    if value < 0 {
        Err(format!("{label} cannot be negative."))
    } else {
        Ok(value)
    }
}

fn optional_non_negative(value: Option<i64>, label: &str) -> Result<Option<i64>, String> {
    value.map(|item| non_negative_money(item, label)).transpose()
}

fn optional_age(value: Option<i64>) -> Result<Option<i64>, String> {
    match value {
        Some(age) if !(0..=120).contains(&age) => Err("Age must be between 0 and 120.".to_string()),
        _ => Ok(value),
    }
}

fn valid_iso_date(value: &str, label: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.len() == 10
        && trimmed.as_bytes().get(4) == Some(&b'-')
        && trimmed.as_bytes().get(7) == Some(&b'-')
        && trimmed.chars().enumerate().all(|(index, character)| {
            matches!(index, 4 | 7) || character.is_ascii_digit()
        })
    {
        Ok(trimmed.to_string())
    } else {
        Err(format!("{label} must use YYYY-MM-DD."))
    }
}

fn optional_iso_date(value: &str, label: &str) -> Result<String, String> {
    if value.trim().is_empty() {
        Ok(String::new())
    } else {
        valid_iso_date(value, label)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session_key() -> RetirementSessionKey {
        Mutex::new(Some(Zeroizing::new([7_u8; 32])))
    }

    #[test]
    fn encryption_uses_distinct_nonces_and_never_keeps_plaintext_in_ciphertext() {
        let key = session_key();
        let value = RetirementProfile {
            display_label: "Private label".to_string(),
            age: Some(45),
            target_age: Some(55),
            retirement_definition: "top-secret retirement definition".to_string(),
            notes: String::new(),
        };
        let (first_nonce, first_ciphertext) =
            encrypt_value(&key, PROFILE_ID, "profile", &value).expect("encryption should work");
        let (second_nonce, _) =
            encrypt_value(&key, PROFILE_ID, "profile", &value).expect("encryption should work");
        assert_ne!(first_nonce, second_nonce);
        assert!(!String::from_utf8_lossy(&first_ciphertext).contains("top-secret"));
    }

    #[test]
    fn encryption_rejects_tampering_and_mismatched_associated_data() {
        let key = session_key();
        let value = RetirementProfile {
            display_label: "Private label".to_string(), age: None, target_age: None,
            retirement_definition: String::new(), notes: String::new(),
        };
        let (nonce, ciphertext) = encrypt_value(&key, PROFILE_ID, "profile", &value).expect("encryption should work");
        let tampered = RetirementProtectedRecord { id: PROFILE_ID.to_string(), entity_type: "profile".to_string(), payload_version: PAYLOAD_VERSION, nonce: nonce.clone(), ciphertext: { let mut data = ciphertext.clone(); data[0] ^= 1; data }, is_archived: false, created_at: String::new(), modified_at: String::new() };
        assert!(decrypt_record::<RetirementProfile>(&key, &tampered).is_err());
        let wrong_aad = RetirementProtectedRecord { id: "other-profile".to_string(), entity_type: "profile".to_string(), payload_version: PAYLOAD_VERSION, nonce, ciphertext, is_archived: false, created_at: String::new(), modified_at: String::new() };
        assert!(decrypt_record::<RetirementProfile>(&key, &wrong_aad).is_err());
    }

    #[test]
    fn locked_session_refuses_data_encryption() {
        let key: RetirementSessionKey = Mutex::new(None);
        let value = RetirementProfile { display_label: String::new(), age: None, target_age: None, retirement_definition: String::new(), notes: String::new() };
        assert!(encrypt_value(&key, PROFILE_ID, "profile", &value).is_err());
    }
}
