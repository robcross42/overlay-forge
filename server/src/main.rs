mod config;
mod error;
mod oauth;
mod repository;
mod routes;
mod security;

use config::{Config, GoogleClient};
use repository::{normalize_email, AccessRepository};
use std::{path::PathBuf, sync::Arc};

pub struct ServerState {
    config: Config,
    google: GoogleClient,
    repository: AccessRepository,
    media: overlay_forge_media_lib::runtime::MediaRuntime,
    http: reqwest::blocking::Client,
    attempts: oauth::Attempts,
    workers: Arc<tokio::sync::Semaphore>,
}
pub type Shared = Arc<ServerState>;

fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    if args.next().as_deref() != Some("--config") {
        return Err("Usage: overlay-forge-server --config PATH [serve | approve EMAIL | revoke EMAIL | list-access]".into());
    }
    let path = PathBuf::from(args.next().ok_or("Missing configuration path.")?);
    let config = Config::load(&path)?;
    private_data_directory(&config.data_dir)?;
    let repository = AccessRepository::new(&config.data_dir.join("browser-access.sqlite3"))
        .map_err(|_| "Could not open server access database.")?;
    let command = args.next().unwrap_or_else(|| "serve".into());
    match command.as_str() {
        "approve" | "revoke" => {
            let email = normalize_email(&args.next().ok_or("Missing email address.")?)
                .map_err(str::to_string)?;
            if args.next().is_some() {
                return Err("Unexpected command arguments.".into());
            }
            if command == "approve" {
                repository.approve(&email)
            } else {
                repository.revoke(&email)
            }
            .map_err(|_| "Could not update approval.")?;
            println!("Email approval updated.");
            return Ok(());
        }
        "list-access" => {
            if args.next().is_some() {
                return Err("Unexpected command arguments.".into());
            }
            for (email, approved, bound) in
                repository.list().map_err(|_| "Could not read approvals.")?
            {
                println!(
                    "{email}\t{}\t{}",
                    if approved { "approved" } else { "revoked" },
                    if bound {
                        "Google account pinned"
                    } else {
                        "not used yet"
                    }
                );
            }
            return Ok(());
        }
        "serve" if args.next().is_none() => {}
        _ => return Err("Unknown command or unexpected arguments.".into()),
    }
    if !config.frontend_dir.join("index.html").is_file() {
        return Err("Build the frontend and configure frontend_dir to its dist directory.".into());
    }
    let public_files = std::fs::canonicalize(&config.frontend_dir)
        .map_err(|_| "Could not resolve frontend directory.")?;
    for private_path in [&config.data_dir, &config.google_client_file] {
        let private_path = std::fs::canonicalize(private_path)
            .map_err(|_| "Could not resolve private server paths.")?;
        if private_path.starts_with(&public_files) {
            return Err(
                "Keep server data and Google credentials outside the frontend directory.".into(),
            );
        }
    }
    let google = config.google_client()?;
    let media_path = config.media_path();
    if let Some(parent) = media_path.parent() {
        std::fs::create_dir_all(parent).map_err(|_| "Could not create Media data directory.")?;
    }
    if let Some(parent) = media_path.parent() {
        let parent =
            std::fs::canonicalize(parent).map_err(|_| "Could not resolve Media directory.")?;
        if parent.starts_with(&public_files) {
            return Err("Keep the Media database outside the frontend directory.".into());
        }
    }
    let media = overlay_forge_media_lib::runtime::MediaRuntime::new(media_path)
        .map_err(|_| "Could not open Media database.")?;
    let http = reqwest::blocking::Client::builder()
        .https_only(true)
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(std::time::Duration::from_secs(5))
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|_| "Could not initialize Google transport.")?;
    let bind = config.bind;
    let state = Arc::new(ServerState {
        config,
        google,
        repository,
        media,
        http,
        attempts: oauth::Attempts::new(),
        workers: Arc::new(tokio::sync::Semaphore::new(8)),
    });
    let runtime = tokio::runtime::Runtime::new().map_err(|_| "Could not start server runtime.")?;
    let result = runtime.block_on(async {
        let listener = tokio::net::TcpListener::bind(bind)
            .await
            .map_err(|_| "Could not bind server; check whether the port is already in use.")?;
        println!("Overlay Forge browser server listening on {bind}");
        axum::serve(listener, routes::router(state.clone()))
            .with_graceful_shutdown(async {
                let _ = tokio::signal::ctrl_c().await;
            })
            .await
            .map_err(|_| "Server stopped after a transport error.")
    });
    result.map_err(str::to_string)
}
fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn private_data_directory(path: &std::path::Path) -> Result<(), String> {
    let mut builder = std::fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder
        .create(path)
        .map_err(|_| "Could not create server data directory.")?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata =
            std::fs::metadata(path).map_err(|_| "Could not inspect server data directory.")?;
        if metadata.permissions().mode() & 0o077 != 0 {
            return Err("Server data directory must be private to its owner (chmod 700).".into());
        }
    }
    Ok(())
}
