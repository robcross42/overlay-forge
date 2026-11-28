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
