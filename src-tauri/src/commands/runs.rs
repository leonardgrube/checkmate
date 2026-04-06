use crate::config::resolve_data_repo;
use crate::models::{Run, RunMetadata, RunSummary, Section};
use crate::parser;
use crate::paths;
use chrono::Local;
use std::fs;
use std::path::Path;

fn build_unique_run_filename(dir: &Path, timestamp_prefix: &str, title: &str) -> String {
    let title_slug = slug::slugify(title);
    let base_slug = if title_slug.is_empty() {
        "run".to_string()
    } else {
        title_slug
    };

    let mut filename = format!("{timestamp_prefix}_{base_slug}.md");
    if !dir.join(&filename).exists() {
        return filename;
    }

    let mut suffix = 2;
    loop {
        filename = format!("{timestamp_prefix}_{base_slug}-{suffix}.md");
        if !dir.join(&filename).exists() {
            return filename;
        }
        suffix += 1;
    }
}

#[tauri::command]
pub fn save_run(
    app: tauri::AppHandle,
    template_slug: String,
    title: String,
    notes: Option<String>,
    sections: Vec<Section>,
) -> Result<String, String> {
    let repo = resolve_data_repo(&app)?;
    let (template_slug, dir) = paths::runs_dir(&repo, &template_slug)?;
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create runs dir: {e}"))?;

    let now = Local::now();
    let date = now.format("%Y-%m-%d %H:%M").to_string();
    let filename_prefix = now.format("%Y-%m-%d_%H-%M").to_string();
    let filename = build_unique_run_filename(&dir, &filename_prefix, &title);
    let path = dir.join(&filename);

    let run_meta = RunMetadata {
        template: template_slug,
        title: title.clone(),
        date,
        notes,
    };
    let content = parser::serialize_run(&run_meta, &sections);
    fs::write(&path, &content).map_err(|e| format!("Failed to write run: {e}"))?;

    super::git::maybe_commit_paths(
        &app,
        &repo,
        &[path.as_path()],
        &format!("Save run: {title}"),
    )?;

    Ok(filename)
}

#[tauri::command]
pub fn list_runs(app: tauri::AppHandle) -> Result<Vec<RunSummary>, String> {
    let repo = resolve_data_repo(&app)?;
    list_runs_in_repo(&repo)
}

fn list_runs_in_repo(repo: &Path) -> Result<Vec<RunSummary>, String> {
    let runs_dir = repo.join("runs");

    if !runs_dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut summaries = Vec::new();

    // Each subdirectory is a template slug
    let templates = fs::read_dir(&runs_dir).map_err(|e| format!("Failed to read runs dir: {e}"))?;

    for template_entry in templates {
        let template_entry = template_entry.map_err(|e| format!("Failed to read entry: {e}"))?;
        let template_path = template_entry.path();

        if !template_path.is_dir() {
            continue;
        }

        let template_slug = template_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();

        let files = fs::read_dir(&template_path)
            .map_err(|e| format!("Failed to read template dir: {e}"))?;

        for file_entry in files {
            let file_entry = file_entry.map_err(|e| format!("Failed to read entry: {e}"))?;
            let path = file_entry.path();

            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }

            let content = match fs::read_to_string(&path) {
                Ok(content) => content,
                Err(err) => {
                    log::warn!("Skipping unreadable run {}: {err}", path.display());
                    continue;
                }
            };

            let (meta, sections) = match parser::parse_run(&content) {
                Ok(parsed) => parsed,
                Err(err) => {
                    log::warn!("Skipping invalid run {}: {err}", path.display());
                    continue;
                }
            };

            let checked = sections
                .iter()
                .flat_map(|s| &s.items)
                .filter(|i| i.checked)
                .count();
            let total = sections.iter().flat_map(|s| &s.items).count();

            let filename = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_string();

            summaries.push(RunSummary {
                template: template_slug.clone(),
                filename,
                title: meta.title,
                date: meta.date,
                notes: meta.notes,
                checked,
                total,
            });
        }
    }

    summaries.sort_by(|a, b| b.date.cmp(&a.date).then(b.filename.cmp(&a.filename)));
    Ok(summaries)
}

#[tauri::command]
pub fn get_run(app: tauri::AppHandle, template: String, filename: String) -> Result<Run, String> {
    let repo = resolve_data_repo(&app)?;
    let (template, filename, path) = paths::run_path(&repo, &template, &filename)?;

    if !path.exists() {
        return Err(format!("Run not found: {template}/{filename}"));
    }

    let content = fs::read_to_string(&path).map_err(|e| format!("Failed to read run: {e}"))?;

    let (meta, sections) = parser::parse_run(&content)?;

    Ok(Run { meta, sections })
}

#[tauri::command]
pub fn delete_run(app: tauri::AppHandle, template: String, filename: String) -> Result<(), String> {
    let repo = resolve_data_repo(&app)?;
    let (template, filename, path) = paths::run_path(&repo, &template, &filename)?;

    if !path.exists() {
        return Err(format!("Run not found: {template}/{filename}"));
    }

    fs::remove_file(&path).map_err(|e| format!("Failed to delete run: {e}"))?;

    super::git::maybe_commit_paths(
        &app,
        &repo,
        &[path.as_path()],
        &format!("Delete run: {template}/{filename}"),
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("checkmate-runs-{name}-{unique}"))
    }

    #[test]
    fn generates_unique_filename_for_same_day_runs() {
        let dir = temp_dir("unique");
        fs::create_dir_all(&dir).unwrap();

        let first = build_unique_run_filename(&dir, "2026-04-06_14-30", "Deploy App");
        fs::write(dir.join(&first), "first").unwrap();
        let second = build_unique_run_filename(&dir, "2026-04-06_14-30", "Deploy App");

        assert_eq!(first, "2026-04-06_14-30_deploy-app.md");
        assert_eq!(second, "2026-04-06_14-30_deploy-app-2.md");

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn list_runs_skips_invalid_markdown_files() {
        let repo = temp_dir("list-runs");
        let runs_dir = repo.join("runs/deploy");
        fs::create_dir_all(&runs_dir).unwrap();

        fs::write(
            runs_dir.join("2026-04-06_14-30_valid.md"),
            r#"---
template: deploy
title: Deploy Run
date: 2026-04-06 14:30
---

## Checks
- [x] Verify deployment
- [ ] Check logs
"#,
        )
        .unwrap();
        fs::write(
            runs_dir.join("2026-04-06_14-31_invalid.md"),
            "this is not a valid run file",
        )
        .unwrap();

        let runs = list_runs_in_repo(&repo).unwrap();

        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].template, "deploy");
        assert_eq!(runs[0].filename, "2026-04-06_14-30_valid.md");
        assert_eq!(runs[0].title, "Deploy Run");
        assert_eq!(runs[0].checked, 1);
        assert_eq!(runs[0].total, 2);

        let _ = fs::remove_dir_all(repo);
    }
}
