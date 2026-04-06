use std::path::{Component, Path, PathBuf};

fn normalize_component(value: &str, label: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(format!("{label} cannot be empty"));
    }

    let mut components = Path::new(trimmed).components();
    let component = components
        .next()
        .ok_or_else(|| format!("Invalid {label}: {trimmed}"))?;

    if components.next().is_some() {
        return Err(format!("{label} must not contain path separators"));
    }

    match component {
        Component::Normal(_) if trimmed != "." && trimmed != ".." => Ok(trimmed.to_string()),
        _ => Err(format!("Invalid {label}: {trimmed}")),
    }
}

pub fn validate_slug(value: &str, label: &str) -> Result<String, String> {
    let normalized = normalize_component(value, label)?;
    if normalized.ends_with(".md") {
        return Err(format!("{label} must not include a file extension"));
    }
    Ok(normalized)
}

pub fn validate_markdown_filename(value: &str, label: &str) -> Result<String, String> {
    let normalized = normalize_component(value, label)?;
    let path = Path::new(&normalized);

    if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
        return Err(format!("{label} must end with .md"));
    }

    if path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("")
        .is_empty()
    {
        return Err(format!("Invalid {label}: {normalized}"));
    }

    Ok(normalized)
}

pub fn checklist_path(repo: &Path, slug: &str) -> Result<(String, PathBuf), String> {
    let slug = validate_slug(slug, "checklist slug")?;
    let path = repo.join("checklists").join(format!("{slug}.md"));
    Ok((slug, path))
}

pub fn runs_dir(repo: &Path, template_slug: &str) -> Result<(String, PathBuf), String> {
    let template_slug = validate_slug(template_slug, "run template slug")?;
    let dir = repo.join("runs").join(&template_slug);
    Ok((template_slug, dir))
}

pub fn run_path(
    repo: &Path,
    template_slug: &str,
    filename: &str,
) -> Result<(String, String, PathBuf), String> {
    let template_slug = validate_slug(template_slug, "run template slug")?;
    let filename = validate_markdown_filename(filename, "run filename")?;
    let path = repo.join("runs").join(&template_slug).join(&filename);
    Ok((template_slug, filename, path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn rejects_checklist_slug_traversal() {
        let repo = Path::new("/tmp/checkmate-data");
        assert!(checklist_path(repo, "../config").is_err());
        assert!(checklist_path(repo, "nested/path").is_err());
    }

    #[test]
    fn rejects_run_path_traversal() {
        let repo = Path::new("/tmp/checkmate-data");
        assert!(run_path(repo, "../template", "run.md").is_err());
        assert!(run_path(repo, "deploy", "../run.md").is_err());
        assert!(run_path(repo, "deploy", "nested/run.md").is_err());
    }

    #[test]
    fn accepts_valid_run_filename() {
        let repo = Path::new("/tmp/checkmate-data");
        let (_, _, path) = run_path(repo, "deploy", "2026-04-06_14-30_run.md").unwrap();
        assert_eq!(
            path,
            Path::new("/tmp/checkmate-data/runs/deploy/2026-04-06_14-30_run.md")
        );
    }
}
