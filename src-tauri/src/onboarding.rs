use crate::config::{default_data_repo_path, load_config, save_config, AppConfig};
use std::fs;
use std::path::Path;
use tauri::AppHandle;

const STARTER_GUIDE_SLUG: &str = "getting-started";
const STARTER_GUIDE_CONTENT: &str = include_str!("../resources/getting-started.md");

pub fn bootstrap_default_repo(app: &AppHandle) -> Result<(), String> {
    let mut config = load_config(app)?;
    if config.data_repo_path.is_some() {
        return Ok(());
    }

    let repo = default_data_repo_path(app)?;
    let seeded = bootstrap_default_repo_at_path(&repo, &mut config)?;

    if seeded {
        save_config(app, &config)?;
    }

    Ok(())
}

pub fn add_starter_guide(app: &AppHandle) -> Result<String, String> {
    let mut config = load_config(app)?;
    let repo = crate::config::resolve_data_repo(app)?;
    let slug = add_starter_guide_at_path(&repo, &mut config, false)?;
    save_config(app, &config)?;
    Ok(slug)
}

fn bootstrap_default_repo_at_path(repo: &Path, config: &mut AppConfig) -> Result<bool, String> {
    let checklists_dir = repo.join("checklists");
    let runs_dir = repo.join("runs");
    let has_checklists = has_checklists(&checklists_dir)?;

    fs::create_dir_all(&checklists_dir)
        .map_err(|e| format!("Failed to create checklists dir: {e}"))?;
    fs::create_dir_all(&runs_dir).map_err(|e| format!("Failed to create runs dir: {e}"))?;

    if has_checklists || config.starter_guide_seeded {
        return Ok(false);
    }

    add_starter_guide_at_path(repo, config, false)?;

    Ok(true)
}

fn add_starter_guide_at_path(
    repo: &Path,
    config: &mut AppConfig,
    overwrite: bool,
) -> Result<String, String> {
    let checklists_dir = repo.join("checklists");
    let runs_dir = repo.join("runs");
    fs::create_dir_all(&checklists_dir)
        .map_err(|e| format!("Failed to create checklists dir: {e}"))?;
    fs::create_dir_all(&runs_dir).map_err(|e| format!("Failed to create runs dir: {e}"))?;

    let guide_path = checklists_dir.join(format!("{STARTER_GUIDE_SLUG}.md"));
    if guide_path.exists() && !overwrite {
        return Err("Starter guide already exists".to_string());
    }

    fs::write(&guide_path, STARTER_GUIDE_CONTENT)
        .map_err(|e| format!("Failed to write starter guide: {e}"))?;
    config.starter_guide_seeded = true;

    Ok(STARTER_GUIDE_SLUG.to_string())
}

fn has_checklists(checklists_dir: &Path) -> Result<bool, String> {
    if !checklists_dir.is_dir() {
        return Ok(false);
    }

    let entries =
        fs::read_dir(checklists_dir).map_err(|e| format!("Failed to read checklists dir: {e}"))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {e}"))?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("md") {
            return Ok(true);
        }
    }

    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_repo_path(name: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("checkmate-{name}-{unique}"))
    }

    #[test]
    fn seeds_when_default_repo_is_missing() {
        let repo = temp_repo_path("seed-missing");
        let mut config = AppConfig::default();

        let seeded = bootstrap_default_repo_at_path(&repo, &mut config).unwrap();

        assert!(seeded);
        assert!(repo.join("checklists").is_dir());
        assert!(repo.join("runs").is_dir());
        assert!(repo.join("checklists/getting-started.md").is_file());
        assert!(config.starter_guide_seeded);

        let _ = fs::remove_dir_all(repo);
    }

    #[test]
    fn seeds_when_default_repo_exists_but_has_no_checklists() {
        let repo = temp_repo_path("seed-empty");
        fs::create_dir_all(repo.join("runs")).unwrap();
        let mut config = AppConfig::default();

        let seeded = bootstrap_default_repo_at_path(&repo, &mut config).unwrap();

        assert!(seeded);
        assert!(repo.join("checklists/getting-started.md").is_file());

        let _ = fs::remove_dir_all(repo);
    }

    #[test]
    fn does_not_seed_when_checklists_already_exist() {
        let repo = temp_repo_path("existing");
        fs::create_dir_all(repo.join("checklists")).unwrap();
        fs::write(
            repo.join("checklists/existing.md"),
            "---\ntitle: \"Existing\"\n---\n",
        )
        .unwrap();
        let mut config = AppConfig::default();

        let seeded = bootstrap_default_repo_at_path(&repo, &mut config).unwrap();

        assert!(!seeded);
        assert!(!config.starter_guide_seeded);
        assert!(!repo.join("checklists/getting-started.md").exists());

        let _ = fs::remove_dir_all(repo);
    }

    #[test]
    fn does_not_reseed_after_previous_seed() {
        let repo = temp_repo_path("no-reseed");
        let mut config = AppConfig {
            starter_guide_seeded: true,
            ..AppConfig::default()
        };

        let seeded = bootstrap_default_repo_at_path(&repo, &mut config).unwrap();

        assert!(!seeded);
        assert!(repo.join("checklists").is_dir());
        assert!(!repo.join("checklists/getting-started.md").exists());

        let _ = fs::remove_dir_all(repo);
    }

    #[test]
    fn manual_add_recreates_starter_guide() {
        let repo = temp_repo_path("manual-add");
        let mut config = AppConfig {
            starter_guide_seeded: true,
            ..AppConfig::default()
        };

        let slug = add_starter_guide_at_path(&repo, &mut config, true).unwrap();

        assert_eq!(slug, "getting-started");
        assert!(repo.join("checklists/getting-started.md").is_file());

        let _ = fs::remove_dir_all(repo);
    }

    #[test]
    fn manual_add_does_not_overwrite_existing_starter_guide() {
        let repo = temp_repo_path("manual-add-existing");
        fs::create_dir_all(repo.join("checklists")).unwrap();
        fs::write(
            repo.join("checklists/getting-started.md"),
            "---\ntitle: Existing\n---\n",
        )
        .unwrap();
        let mut config = AppConfig::default();

        let err = add_starter_guide_at_path(&repo, &mut config, false).unwrap_err();

        assert_eq!(err, "Starter guide already exists");
        assert_eq!(
            fs::read_to_string(repo.join("checklists/getting-started.md")).unwrap(),
            "---\ntitle: Existing\n---\n"
        );
        assert!(!config.starter_guide_seeded);

        let _ = fs::remove_dir_all(repo);
    }
}
