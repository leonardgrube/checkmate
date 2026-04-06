mod commands;
mod config;
mod models;
mod onboarding;
mod parser;
mod paths;
mod watcher;

pub fn run() {
    tauri::Builder::default()
        .manage(watcher::ChecklistWatcherState::default())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Apply saved window size on startup
            if let Ok(cfg) = config::load_config(app.handle()) {
                let _ = commands::config::apply_always_on_top(app.handle(), cfg.always_on_top);
                if let Some(ref size) = cfg.window_size {
                    let _ = commands::config::apply_window_size(app.handle(), size);
                }
            }

            if let Err(err) = onboarding::bootstrap_default_repo(app.handle()) {
                log::warn!("Failed to bootstrap default data repo: {err}");
            }

            if let Err(err) = watcher::restart(app.handle()) {
                log::warn!("Checklist watcher not started: {err}");
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::checklists::list_checklists,
            commands::checklists::add_sample_checklist,
            commands::checklists::get_checklist,
            commands::checklists::create_checklist,
            commands::checklists::update_checklist,
            commands::checklists::delete_checklist,
            commands::runs::save_run,
            commands::runs::list_runs,
            commands::runs::get_run,
            commands::runs::delete_run,
            commands::git::check_is_git_repo,
            commands::config::get_config,
            commands::config::update_config,
            commands::config::set_window_size,
            commands::config::get_default_data_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
