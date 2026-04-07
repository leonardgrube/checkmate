use crate::config::{load_config, AppConfig};
use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::process::Command;
use tauri::AppHandle;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

fn git_command() -> Command {
    #[cfg(target_os = "windows")]
    {
        let mut cmd = Command::new("git");
        cmd.creation_flags(CREATE_NO_WINDOW);
        cmd
    }
    #[cfg(not(target_os = "windows"))]
    Command::new("git")
}

fn run_git<I, S>(repo_path: &Path, args: I) -> Result<String, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = git_command()
        .env("LC_ALL", "C")
        .arg("-C")
        .arg(repo_path)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run git: {e}"))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(stderr)
    }
}

fn git_exit_status<I, S>(repo_path: &Path, args: I) -> Result<i32, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let status = git_command()
        .env("LC_ALL", "C")
        .arg("-C")
        .arg(repo_path)
        .args(args)
        .status()
        .map_err(|e| format!("Failed to run git: {e}"))?;

    Ok(status.code().unwrap_or(-1))
}

pub fn is_git_repo(repo_path: &Path) -> bool {
    if !repo_path.is_dir() {
        return false;
    }

    run_git(repo_path, ["rev-parse", "--git-dir"]).is_ok()
}

fn stage_paths(repo_path: &Path, paths: &[&Path]) -> Result<(), String> {
    if paths.is_empty() {
        return Ok(());
    }

    let mut args: Vec<OsString> = vec!["add".into(), "-A".into(), "--".into()];
    for path in paths {
        let relative = path
            .strip_prefix(repo_path)
            .map_err(|_| format!("Path is outside repo: {}", path.display()))?;
        args.push(relative.as_os_str().to_os_string());
    }

    run_git(repo_path, args)?;
    Ok(())
}

fn commit_staged(repo_path: &Path, message: &str) -> Result<(), String> {
    match git_exit_status(repo_path, ["diff", "--cached", "--quiet"]) {
        Ok(0) => Ok(()),
        Ok(1) => match run_git(repo_path, ["commit", "-m", message]) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Git commit failed: {e}")),
        },
        Ok(code) => Err(format!("Git diff --cached failed with exit code {code}")),
        Err(e) => Err(e),
    }
}

pub fn commit_paths(repo_path: &Path, paths: &[&Path], message: &str) -> Result<(), String> {
    stage_paths(repo_path, paths)?;
    commit_staged(repo_path, message)
}

pub fn maybe_commit_paths(
    app: &AppHandle,
    repo_path: &Path,
    paths: &[&Path],
    message: &str,
) -> Result<(), String> {
    let config = load_config(app).unwrap_or_else(|_| AppConfig::default());
    maybe_commit_with_config_best_effort(&config, repo_path, paths, message);
    Ok(())
}

fn maybe_commit_with_config(
    config: &AppConfig,
    repo_path: &Path,
    paths: &[&Path],
    message: &str,
) -> Result<(), String> {
    if !config.auto_commit || !is_git_repo(repo_path) {
        return Ok(());
    }

    commit_paths(repo_path, paths, message)
}

fn maybe_commit_with_config_best_effort(
    config: &AppConfig,
    repo_path: &Path,
    paths: &[&Path],
    message: &str,
) {
    if let Err(err) = maybe_commit_with_config(config, repo_path, paths, message) {
        log::warn!("Auto-commit failed for {}: {}", repo_path.display(), err);
    }
}

#[tauri::command]
pub async fn check_is_git_repo(path: String) -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(move || Ok(is_git_repo(Path::new(&path))))
        .await
        .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    fn temp_repo_path(name: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("checkmate-git-{name}-{unique}"))
    }

    fn run_raw_git(repo_path: &Path, args: &[&str]) {
        let status = Command::new("git")
            .arg("-C")
            .arg(repo_path)
            .args(args)
            .status()
            .unwrap();
        assert!(status.success());
    }

    #[cfg(unix)]
    fn install_failing_pre_commit_hook(repo_path: &Path) {
        let hook_path = repo_path.join(".git/hooks/pre-commit");
        fs::write(&hook_path, "#!/bin/sh\nexit 1\n").unwrap();

        let mut permissions = fs::metadata(&hook_path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&hook_path, permissions).unwrap();
    }

    fn auto_commit_config() -> AppConfig {
        AppConfig {
            auto_commit: true,
            ..AppConfig::default()
        }
    }

    #[test]
    fn skips_auto_commit_for_non_git_folder() {
        let repo = temp_repo_path("plain");
        fs::create_dir_all(&repo).unwrap();
        let note = repo.join("notes.md");
        fs::write(&note, "hello").unwrap();

        maybe_commit_with_config(
            &AppConfig::default(),
            &repo,
            &[note.as_path()],
            "Test commit",
        )
        .unwrap();

        assert!(!repo.join(".git").exists());

        let _ = fs::remove_dir_all(repo);
    }

    #[test]
    fn commits_when_auto_commit_enabled_and_repo_is_git() {
        let repo = temp_repo_path("git");
        fs::create_dir_all(&repo).unwrap();
        run_raw_git(&repo, &["init"]);
        run_raw_git(&repo, &["config", "user.email", "test@example.com"]);
        run_raw_git(&repo, &["config", "user.name", "checkmate test"]);
        let note = repo.join("notes.md");
        fs::write(&note, "hello").unwrap();

        maybe_commit_with_config(
            &auto_commit_config(),
            &repo,
            &[note.as_path()],
            "Initial commit",
        )
        .unwrap();

        let log = run_git(&repo, ["log", "--oneline", "-1"]).unwrap();
        assert!(log.contains("Initial commit"));

        let _ = fs::remove_dir_all(repo);
    }

    #[test]
    fn skips_commit_when_git_repo_has_no_changes() {
        let repo = temp_repo_path("noop");
        fs::create_dir_all(&repo).unwrap();
        run_raw_git(&repo, &["init"]);
        run_raw_git(&repo, &["config", "user.email", "test@example.com"]);
        run_raw_git(&repo, &["config", "user.name", "checkmate test"]);
        let note = repo.join("notes.md");
        fs::write(&note, "hello").unwrap();
        run_raw_git(&repo, &["add", "-A"]);
        run_raw_git(&repo, &["commit", "-m", "Initial commit"]);

        maybe_commit_with_config(
            &AppConfig::default(),
            &repo,
            &[note.as_path()],
            "No-op commit",
        )
        .unwrap();

        let log = run_git(&repo, ["log", "--oneline"]).unwrap();
        assert_eq!(log.lines().count(), 1);

        let _ = fs::remove_dir_all(repo);
    }

    #[test]
    fn commits_only_selected_paths() {
        let repo = temp_repo_path("scoped");
        fs::create_dir_all(repo.join("checklists")).unwrap();
        fs::create_dir_all(repo.join("runs/deploy")).unwrap();
        run_raw_git(&repo, &["init"]);
        run_raw_git(&repo, &["config", "user.email", "test@example.com"]);
        run_raw_git(&repo, &["config", "user.name", "checkmate test"]);

        let checklist = repo.join("checklists/deploy.md");
        let run = repo.join("runs/deploy/2026-04-06_14-30_deploy.md");
        let notes = repo.join("notes.md");
        fs::write(&checklist, "first").unwrap();
        fs::write(&run, "run").unwrap();
        fs::write(&notes, "notes").unwrap();
        run_raw_git(&repo, &["add", "-A"]);
        run_raw_git(&repo, &["commit", "-m", "Initial commit"]);

        fs::write(&checklist, "updated checklist").unwrap();
        fs::write(&notes, "updated notes").unwrap();

        maybe_commit_with_config(
            &auto_commit_config(),
            &repo,
            &[checklist.as_path()],
            "Update checklist only",
        )
        .unwrap();

        let committed = run_git(&repo, ["show", "--name-only", "--format=", "HEAD"]).unwrap();
        assert!(committed.lines().any(|line| line == "checklists/deploy.md"));
        assert!(!committed.lines().any(|line| line == "notes.md"));

        let status = run_git(&repo, ["status", "--short"]).unwrap();
        assert!(status.lines().any(|line| line.contains("notes.md")));

        let _ = fs::remove_dir_all(repo);
    }

    #[test]
    fn best_effort_auto_commit_swallows_commit_failures() {
        let repo = temp_repo_path("best-effort");
        fs::create_dir_all(&repo).unwrap();
        run_raw_git(&repo, &["init"]);
        run_raw_git(&repo, &["config", "user.email", "test@example.com"]);
        run_raw_git(&repo, &["config", "user.name", "checkmate test"]);
        install_failing_pre_commit_hook(&repo);

        let note = repo.join("notes.md");
        fs::write(&note, "hello").unwrap();

        maybe_commit_with_config_best_effort(
            &auto_commit_config(),
            &repo,
            &[note.as_path()],
            "Initial commit",
        );

        assert!(note.exists());

        let status = run_git(&repo, ["status", "--short"]).unwrap();
        assert!(status.lines().any(|line| line.contains("notes.md")));

        let _ = fs::remove_dir_all(repo);
    }
}
