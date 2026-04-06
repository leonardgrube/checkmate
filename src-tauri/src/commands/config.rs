use crate::config::{default_data_repo_path, load_config, save_config, AppConfig, WindowPresets};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::Manager;

#[derive(Serialize, Deserialize, Clone)]
pub struct SettingsConfigUpdate {
    #[serde(default)]
    pub data_repo_path: Option<String>,
    pub auto_commit: bool,
    pub always_on_top: bool,
    pub auto_scroll: bool,
    pub window_presets: WindowPresets,
}

#[tauri::command]
pub fn get_config(app: tauri::AppHandle) -> Result<AppConfig, String> {
    load_config(&app)
}

fn normalize_data_repo_path(path: Option<String>) -> Result<Option<String>, String> {
    let Some(path) = path else {
        return Ok(None);
    };

    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    if !Path::new(trimmed).is_dir() {
        return Err(format!("Path does not exist: {trimmed}"));
    }

    Ok(Some(trimmed.to_string()))
}

fn merge_settings(existing: AppConfig, update: SettingsConfigUpdate) -> AppConfig {
    AppConfig {
        data_repo_path: update.data_repo_path,
        auto_commit: update.auto_commit,
        always_on_top: update.always_on_top,
        auto_scroll: update.auto_scroll,
        window_presets: update.window_presets,
        window_size: existing.window_size,
        starter_guide_seeded: existing.starter_guide_seeded,
    }
}

#[tauri::command]
pub fn update_config(
    app: tauri::AppHandle,
    mut config: SettingsConfigUpdate,
) -> Result<(), String> {
    config.data_repo_path = normalize_data_repo_path(config.data_repo_path)?;
    let existing = load_config(&app)?;
    let merged = merge_settings(existing, config);
    save_config(&app, &merged)?;
    apply_always_on_top(&app, merged.always_on_top)?;
    if let Err(err) = crate::watcher::restart(&app) {
        log::warn!("Failed to restart checklist watcher: {err}");
        let _ = crate::watcher::clear(&app);
    }
    Ok(())
}

#[tauri::command]
pub fn get_default_data_path(app: tauri::AppHandle) -> Result<String, String> {
    let path = default_data_repo_path(&app)?;
    path.canonicalize()
        .unwrap_or(path)
        .to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "Path contains invalid UTF-8".to_string())
}

#[tauri::command]
pub fn set_window_size(app: tauri::AppHandle, size: String) -> Result<(), String> {
    apply_window_size(&app, &size)?;

    let mut config = load_config(&app)?;
    config.window_size = Some(size);
    save_config(&app, &config)
}

pub fn apply_always_on_top(app: &tauri::AppHandle, always_on_top: bool) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or("Failed to get main window")?;

    window
        .set_always_on_top(always_on_top)
        .map_err(|e| format!("Failed to set always on top: {e}"))
}

pub fn apply_window_size(app: &tauri::AppHandle, size: &str) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or("Failed to get main window")?;

    let config = load_config(app)?;
    let presets = &config.window_presets;

    match size {
        "compact" => {
            let [w, h] = presets.compact;
            let _ = window.set_size(tauri::LogicalSize::new(w as f64, h as f64));
        }
        "sidebar" => {
            if let Ok(Some(monitor)) = window.current_monitor() {
                let screen = monitor.size();
                let scale = monitor.scale_factor();
                let mon_pos = monitor.position();
                let screen_w = screen.width as f64;
                let screen_h = screen.height as f64;
                let win_w = presets.sidebar_width as f64 * scale;
                let x = if presets.sidebar_dock == "left" {
                    mon_pos.x as f64
                } else {
                    mon_pos.x as f64 + screen_w - win_w
                };
                let y = mon_pos.y as f64;
                let phys_size = tauri::PhysicalSize::new(win_w as u32, screen_h as u32);
                let phys_pos = tauri::PhysicalPosition::new(x as i32, y as i32);
                let _ = window.set_size(phys_size);
                // Position after WM finishes processing the resize
                let win_clone = window.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(150));
                    let _ = win_clone.set_position(phys_pos);
                });
            }
        }
        "large" => {
            let [w, h] = presets.large;
            let _ = window.set_size(tauri::LogicalSize::new(w as f64, h as f64));
        }
        _ => {}
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_preserves_non_settings_fields() {
        let existing = AppConfig {
            data_repo_path: Some("/tmp/original".to_string()),
            auto_commit: true,
            always_on_top: true,
            auto_scroll: false,
            window_size: Some("sidebar".to_string()),
            window_presets: WindowPresets::default(),
            starter_guide_seeded: true,
        };
        let update = SettingsConfigUpdate {
            data_repo_path: Some("/tmp/new".to_string()),
            auto_commit: false,
            always_on_top: false,
            auto_scroll: true,
            window_presets: WindowPresets {
                sidebar_dock: "left".to_string(),
                ..WindowPresets::default()
            },
        };

        let merged = merge_settings(existing, update);
        assert_eq!(merged.data_repo_path.as_deref(), Some("/tmp/new"));
        assert!(!merged.auto_commit);
        assert!(!merged.always_on_top);
        assert!(merged.auto_scroll);
        assert_eq!(merged.window_presets.sidebar_dock, "left");
        assert_eq!(merged.window_size.as_deref(), Some("sidebar"));
        assert!(merged.starter_guide_seeded);
    }

    #[test]
    fn normalize_empty_data_repo_path_to_none() {
        assert_eq!(normalize_data_repo_path(None).unwrap(), None);
        assert_eq!(normalize_data_repo_path(Some(String::new())).unwrap(), None);
        assert_eq!(
            normalize_data_repo_path(Some("   ".to_string())).unwrap(),
            None
        );
    }
}
