use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

fn default_true() -> bool {
    true
}

fn default_compact() -> [u32; 2] {
    [420, 900]
}

fn default_sidebar_width() -> u32 {
    420
}

fn default_sidebar_dock() -> String {
    "right".to_string()
}

fn default_large() -> [u32; 2] {
    [720, 1100]
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WindowPresets {
    #[serde(default = "default_compact")]
    pub compact: [u32; 2],
    #[serde(default = "default_sidebar_width")]
    pub sidebar_width: u32,
    #[serde(default = "default_sidebar_dock")]
    pub sidebar_dock: String,
    #[serde(default = "default_large")]
    pub large: [u32; 2],
}

impl Default for WindowPresets {
    fn default() -> Self {
        Self {
            compact: default_compact(),
            sidebar_width: default_sidebar_width(),
            sidebar_dock: default_sidebar_dock(),
            large: default_large(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AppConfig {
    #[serde(default)]
    pub data_repo_path: Option<String>,
    #[serde(default)]
    pub auto_commit: bool,
    #[serde(default = "default_true")]
    pub always_on_top: bool,
    #[serde(default)]
    pub auto_scroll: bool,
    #[serde(default)]
    pub window_size: Option<String>,
    #[serde(default)]
    pub window_presets: WindowPresets,
    #[serde(default)]
    pub starter_guide_seeded: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            data_repo_path: None,
            auto_commit: false,
            always_on_top: true,
            auto_scroll: false,
            window_size: None,
            window_presets: WindowPresets::default(),
            starter_guide_seeded: false,
        }
    }
}

fn config_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {e}"))?;
    Ok(dir.join("config.json"))
}

pub fn load_config(app: &AppHandle) -> Result<AppConfig, String> {
    let path = config_path(app)?;
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    let content = fs::read_to_string(&path).map_err(|e| format!("Failed to read config: {e}"))?;
    serde_json::from_str(&content).map_err(|e| format!("Failed to parse config: {e}"))
}

pub fn save_config(app: &AppHandle, config: &AppConfig) -> Result<(), String> {
    let path = config_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create config dir: {e}"))?;
    }
    let content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {e}"))?;
    fs::write(&path, content).map_err(|e| format!("Failed to write config: {e}"))
}

/// Default data repo path.
/// In dev, we resolve from CARGO_MANIFEST_DIR (src-tauri/) → ../../checkmate-data.
/// In production, we resolve from the OS app data directory.
pub fn default_data_repo_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(if cfg!(debug_assertions) {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../checkmate-data")
    } else {
        app.path()
            .app_data_dir()
            .map_err(|e| format!("Failed to get app data dir: {e}"))?
            .join("checkmate-data")
    })
}

/// For the default path, the directory does not need to exist yet.
pub fn resolve_data_repo(app: &AppHandle) -> Result<PathBuf, String> {
    let config = load_config(app)?;
    if let Some(path) = config.data_repo_path {
        let p = PathBuf::from(&path);
        if p.is_dir() {
            return Ok(p);
        }
        return Err(format!("Configured data repo path does not exist: {path}"));
    }

    default_data_repo_path(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_auto_commit_deserializes_to_false() {
        let config: AppConfig = serde_json::from_str(
            r#"{
                "always_on_top": true,
                "auto_scroll": false
            }"#,
        )
        .unwrap();

        assert!(!config.auto_commit);
        assert!(config.always_on_top);
    }
}
