use hockey_comc_inventory::Inventory;
use std::{io::ErrorKind, path::Path};

use crate::config::Config;

pub fn configure(config: &Config, public_files: &Path) -> Option<Inventory> {
    config.hockey_inventory_owner_email.as_ref()?;
    match private_inventory(config, public_files) {
        Ok(inventory) => Some(inventory),
        Err(message) => {
            eprintln!("{message}");
            None
        }
    }
}

fn private_inventory(config: &Config, public_files: &Path) -> Result<Inventory, &'static str> {
    let database = config
        .hockey_inventory_database
        .clone()
        .unwrap_or_else(|| config.data_dir.join("hockey-card-companion.sqlite3"));
    let database = match std::fs::canonicalize(&database) {
        Ok(path) => path,
        Err(error)
            if error.kind() == ErrorKind::NotFound
                && config.hockey_inventory_database.is_none()
                && matches!(std::fs::symlink_metadata(&database), Err(error) if error.kind() == ErrorKind::NotFound) =>
        {
            return Ok(Inventory::new(database));
        }
        _ => return Err("Could not resolve the configured Hockey Cards database."),
    };
    let metadata = std::fs::metadata(&database)
        .map_err(|_| "Could not inspect the configured Hockey Cards database.")?;
    if database.starts_with(public_files) || !metadata.is_file() {
        return Err("Hockey Cards requires a private SQLite file outside the frontend directory.");
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o077 != 0 {
            return Err(
                "The Hockey Cards database must be private to the service user (chmod 600).",
            );
        }
    }
    Ok(Inventory::new(database))
}

use crate::{error::ApiError, routes, Shared};
use axum::{
    extract::{FromRequest, Request, State},
    http::StatusCode,
    Json,
};
use hockey_comc_inventory::{
    market::{MarketQuery, MarketSnapshot},
    sales::{SalesHistory, SalesImport, SalesObservation, SalesPage, SalesQuery},
    InventoryError,
};
use serde::de::DeserializeOwned;
use std::path::PathBuf;

pub struct MarketServices {
    pub sales: Option<SalesHistory>,
    pub snapshot: Option<MarketSnapshot>,
}

impl MarketServices {
    pub fn configure(config: &Config, public_files: &Path) -> Self {
        if config.hockey_inventory_owner_email.is_none() {
            return Self {
                sales: None,
                snapshot: None,
            };
        }
        let sales = config
            .hockey_sales_database
            .as_deref()
            .and_then(|path| private_market_file(path, public_files))
            .map(SalesHistory::new);
        let snapshot = config
            .hockey_market_snapshot
            .as_deref()
            .and_then(|path| private_market_file(path, public_files))
            .and_then(|path| match MarketSnapshot::open(&path) {
                Ok(snapshot) => Some(snapshot),
                Err(_) => {
                    eprintln!("Could not load the Hockey Cards market snapshot.");
                    None
                }
            });
        Self { sales, snapshot }
    }
}

fn private_market_file(path: &Path, public_files: &Path) -> Option<PathBuf> {
    fn inspect(path: &Path, public_files: &Path) -> Result<PathBuf, std::io::Error> {
        let canonical = std::fs::canonicalize(path)?;
        let metadata = std::fs::metadata(&canonical)?;
        let private = {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                metadata.permissions().mode() & 0o077 == 0
            }
            #[cfg(not(unix))]
            true
        };
        if canonical.starts_with(public_files) || !metadata.is_file() || !private {
            return Err(std::io::Error::new(
                ErrorKind::PermissionDenied,
                "Hockey Cards data must be private.",
            ));
        }
        Ok(canonical)
    }
    match inspect(path, public_files) {
        Ok(path) => Some(path),
        Err(_) => {
            eprintln!(
                "Hockey Cards requires a private market data file outside the frontend directory."
            );
            None
        }
    }
}

async fn authorized_input<T: DeserializeOwned>(
    state: &Shared,
    request: Request,
) -> Result<T, ApiError> {
    let (_, session) = routes::authorize_write(state, request.headers())?;
    if !routes::hockey_inventory_allowed(state, &session) {
        return Err(ApiError::forbidden());
    }
    Json::<T>::from_request(request, state)
        .await
        .map(|Json(input)| input)
        .map_err(|_| ApiError(StatusCode::BAD_REQUEST, "Invalid COMC request."))
}

fn api_error(error: InventoryError) -> ApiError {
    match error {
        InventoryError::Invalid(message) => ApiError(StatusCode::BAD_REQUEST, message),
        InventoryError::Unavailable => ApiError::unavailable(),
    }
}

pub async fn market_query(
    State(state): State<Shared>,
    request: Request,
) -> Result<Json<serde_json::Value>, ApiError> {
    let input = authorized_input::<MarketQuery>(&state, request).await?;
    let snapshot = state
        .hockey_market
        .snapshot
        .as_ref()
        .ok_or_else(ApiError::forbidden)?;
    let result = snapshot.query(input).map_err(api_error)?;
    serde_json::to_value(result)
        .map(Json)
        .map_err(|_| ApiError::unavailable())
}

async fn sales_operation<T: Send + 'static>(
    state: Shared,
    operation: impl FnOnce(&SalesHistory) -> Result<T, InventoryError> + Send + 'static,
) -> Result<Json<T>, ApiError> {
    let permit = state
        .workers
        .clone()
        .try_acquire_owned()
        .map_err(|_| ApiError::unavailable())?;
    tokio::task::spawn_blocking(move || {
        let _permit = permit;
        let sales = state
            .hockey_market
            .sales
            .as_ref()
            .ok_or_else(ApiError::forbidden)?;
        operation(sales).map(Json).map_err(api_error)
    })
    .await
    .map_err(|_| ApiError::unavailable())?
}

pub async fn sales_query(
    State(state): State<Shared>,
    request: Request,
) -> Result<Json<SalesPage>, ApiError> {
    let input = authorized_input::<SalesQuery>(&state, request).await?;
    sales_operation(state, move |sales| sales.query(input)).await
}

pub async fn observe_sales(
    State(state): State<Shared>,
    request: Request,
) -> Result<Json<SalesImport>, ApiError> {
    let input = authorized_input::<SalesObservation>(&state, request).await?;
    sales_operation(state, move |sales| sales.save(input)).await
}
