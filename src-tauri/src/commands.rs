use crate::AppState;
use serde::Serialize;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager, State};

#[derive(Clone, Copy)]
enum ProductKind {
    Media,
    Gaming,
    Retirement,
}

impl ProductKind {
    fn key(self) -> &'static str {
        match self {
            Self::Media => "media",
            Self::Gaming => "gaming",
            Self::Retirement => "retirement",
        }
    }
    fn name(self) -> &'static str {
        match self {
            Self::Media => "Overlay Forge Media",
            Self::Gaming => "Overlay Forge Gaming",
            Self::Retirement => "Overlay Forge Retirement",
        }
    }
    fn executable(self) -> &'static str {
        match self {
            Self::Media => "overlay-forge-media.exe",
            Self::Gaming => "overlay-forge-gaming.exe",
            Self::Retirement => "overlay-forge-retirement.exe",
        }
    }
    fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "media" => Some(Self::Media),
            "gaming" => Some(Self::Gaming),
            "retirement" => Some(Self::Retirement),
            _ => None,
        }
    }
}

const PRODUCTS: [ProductKind; 3] = [
    ProductKind::Media,
    ProductKind::Gaming,
    ProductKind::Retirement,
];

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductRecord {
    key: String,
    name: String,
    available: bool,
    launch_path: String,
}

fn candidate_paths(kind: ProductKind) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        paths.push(
            PathBuf::from(local)
                .join(kind.name())
                .join(kind.executable()),
        );
    }
    if let Ok(current) = std::env::current_exe() {
        if let Some(dir) = current.parent() {
            paths.push(dir.join(kind.executable()));
        }
    }
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if let Some(repos) = manifest.parent().and_then(Path::parent) {
        paths.push(
            repos
                .join(format!("overlay-forge-{}", kind.key()))
                .join("src-tauri")
                .join("target")
                .join("debug")
                .join(kind.executable()),
        );
    }
    paths
}

fn resolve(kind: ProductKind) -> Option<PathBuf> {
    candidate_paths(kind)
        .into_iter()
        .find(|path| path.is_file())
}

#[tauri::command]
pub fn list_products() -> Vec<ProductRecord> {
    PRODUCTS
        .into_iter()
        .map(|kind| {
            let path = resolve(kind);
            ProductRecord {
                key: kind.key().into(),
                name: kind.name().into(),
                available: path.is_some(),
                launch_path: path
                    .map(|p| p.to_string_lossy().into_owned())
                    .unwrap_or_default(),
            }
        })
        .collect()
}

fn launch(kind: ProductKind, state: &AppState) -> Result<(), String> {
    let path = resolve(kind).ok_or_else(|| {
        format!(
            "{} is not installed or built in its sibling development repository.",
            kind.name()
        )
    })?;
    std::process::Command::new(&path)
        .spawn()
        .map_err(|e| format!("Could not launch {}: {e}", kind.name()))?;
    state
        .database
        .remember_product(kind.key())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn launch_product(
    product_key: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    let kind =
        ProductKind::parse(&product_key).ok_or_else(|| "Unknown product target.".to_string())?;
    launch(kind, state.inner())?;
    app.exit(0);
    Ok(())
}

pub fn launch_requested_or_last(app: &AppHandle, state: &AppState) -> Result<bool, String> {
    let requested = std::env::args()
        .skip(1)
        .find_map(|arg| ProductKind::parse(arg.trim_start_matches("--product=")));
    let target = requested.or_else(|| {
        state
            .database
            .last_product()
            .ok()
            .flatten()
            .and_then(|key| ProductKind::parse(&key))
    });
    if let Some(kind) = target {
        if resolve(kind).is_some() {
            launch(kind, state)?;
            app.exit(0);
            return Ok(true);
        }
    }
    Ok(false)
}

#[tauri::command]
pub fn shutdown_app(app: AppHandle) {
    app.exit(0)
}

pub fn show_picker(app: &AppHandle) -> Result<(), String> {
    app.get_webview_window("main")
        .ok_or_else(|| "Launcher window is unavailable.".to_string())?
        .show()
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::ProductKind;

    #[test]
    fn registry_accepts_only_closed_product_keys() {
        assert!(matches!(
            ProductKind::parse("media"),
            Some(ProductKind::Media)
        ));
        assert!(matches!(
            ProductKind::parse("gaming"),
            Some(ProductKind::Gaming)
        ));
        assert!(matches!(
            ProductKind::parse("retirement"),
            Some(ProductKind::Retirement)
        ));
        assert!(ProductKind::parse("powershell -Command calc").is_none());
    }
}
