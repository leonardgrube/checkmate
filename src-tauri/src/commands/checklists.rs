use crate::config::resolve_data_repo;
use crate::models::{Checklist, ChecklistSummary, Section};
use crate::onboarding;
use crate::parser;
use crate::paths;
use std::fs;
use std::path::{Path, PathBuf};

fn build_new_checklist_path(dir: &Path, title: &str) -> Result<(String, PathBuf), String> {
    let file_slug = slug::slugify(title);
    if file_slug.is_empty() {
        return Err("Checklist title must include at least one letter or number".to_string());
    }

    let path = dir.join(format!("{file_slug}.md"));
    if path.exists() {
        return Err(format!("Checklist '{file_slug}' already exists"));
    }

    Ok((file_slug, path))
}

#[tauri::command]
pub fn list_checklists(app: tauri::AppHandle) -> Result<Vec<ChecklistSummary>, String> {
    onboarding::bootstrap_default_repo(&app)?;
    let repo = resolve_data_repo(&app)?;
    let dir = repo.join("checklists");

    if !dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut summaries = Vec::new();
    let entries = fs::read_dir(&dir).map_err(|e| format!("Failed to read checklists dir: {e}"))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {e}"))?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }

        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(err) => {
                log::warn!("Skipping unreadable checklist {}: {err}", path.display());
                continue;
            }
        };

        let meta = match parser::parse_meta(&content) {
            Ok(meta) => meta,
            Err(err) => {
                log::warn!("Skipping invalid checklist {}: {err}", path.display());
                continue;
            }
        };
        let slug = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();

        summaries.push(ChecklistSummary {
            slug,
            title: meta.title,
            description: meta.description,
            tags: meta.tags,
        });
    }

    summaries.sort_by(|a, b| a.title.cmp(&b.title));
    Ok(summaries)
}

#[tauri::command]
pub fn add_sample_checklist(app: tauri::AppHandle) -> Result<String, String> {
    let repo = resolve_data_repo(&app)?;
    let slug = onboarding::add_starter_guide(&app)?;
    let (_, path) = paths::checklist_path(&repo, &slug)?;

    super::git::maybe_commit_paths(
        &app,
        &repo,
        &[path.as_path()],
        "Add sample checklist: Getting Started with checkmate",
    )?;

    Ok(slug)
}

#[tauri::command]
pub fn get_checklist(app: tauri::AppHandle, slug: String) -> Result<Checklist, String> {
    let repo = resolve_data_repo(&app)?;
    let (slug, path) = paths::checklist_path(&repo, &slug)?;

    if !path.exists() {
        return Err(format!("Checklist '{slug}' not found"));
    }

    let content =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read checklist: {e}"))?;

    let (meta, sections) = parser::parse_checklist(&content)?;

    Ok(Checklist {
        slug,
        meta,
        sections,
    })
}

#[tauri::command]
pub fn create_checklist(
    app: tauri::AppHandle,
    title: String,
    description: Option<String>,
    tags: Vec<String>,
    sections: Vec<Section>,
) -> Result<String, String> {
    let repo = resolve_data_repo(&app)?;
    let dir = repo.join("checklists");
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create checklists dir: {e}"))?;

    let (file_slug, path) = build_new_checklist_path(&dir, &title)?;

    let meta = crate::models::ChecklistMeta {
        title,
        description,
        tags,
    };
    let content = parser::serialize_checklist(&meta, &sections);
    fs::write(&path, &content).map_err(|e| format!("Failed to write checklist: {e}"))?;

    super::git::maybe_commit_paths(
        &app,
        &repo,
        &[path.as_path()],
        &format!("Add checklist: {}", meta.title),
    )?;

    Ok(file_slug)
}

#[tauri::command]
pub fn update_checklist(
    app: tauri::AppHandle,
    slug: String,
    title: String,
    description: Option<String>,
    tags: Vec<String>,
    sections: Vec<Section>,
) -> Result<(), String> {
    let repo = resolve_data_repo(&app)?;
    let (slug, path) = paths::checklist_path(&repo, &slug)?;

    if !path.exists() {
        return Err(format!("Checklist '{slug}' not found"));
    }

    let meta = crate::models::ChecklistMeta {
        title,
        description,
        tags,
    };
    let content = parser::serialize_checklist(&meta, &sections);
    fs::write(&path, &content).map_err(|e| format!("Failed to write checklist: {e}"))?;

    super::git::maybe_commit_paths(
        &app,
        &repo,
        &[path.as_path()],
        &format!("Update checklist: {}", meta.title),
    )?;

    Ok(())
}

#[tauri::command]
pub fn delete_checklist(app: tauri::AppHandle, slug: String) -> Result<(), String> {
    let repo = resolve_data_repo(&app)?;
    let (slug, path) = paths::checklist_path(&repo, &slug)?;

    if !path.exists() {
        return Err(format!("Checklist '{slug}' not found"));
    }

    let content =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read checklist: {e}"))?;
    let (meta, _) = parser::parse_checklist(&content)?;

    fs::remove_file(&path).map_err(|e| format!("Failed to delete checklist: {e}"))?;

    super::git::maybe_commit_paths(
        &app,
        &repo,
        &[path.as_path()],
        &format!("Delete checklist: {}", meta.title),
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("checkmate-checklists-{name}-{unique}"))
    }

    #[test]
    fn rejects_duplicate_checklist_paths() {
        let dir = temp_dir("duplicate");
        fs::create_dir_all(&dir).unwrap();
        let (_, path) = build_new_checklist_path(&dir, "Deploy App").unwrap();
        fs::write(&path, "existing").unwrap();

        let err = build_new_checklist_path(&dir, "Deploy App").unwrap_err();
        assert!(err.contains("already exists"));

        let _ = fs::remove_dir_all(dir);
    }
}
