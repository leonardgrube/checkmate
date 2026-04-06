use crate::config::resolve_data_repo;
use crate::models::{ChecklistChangeEvent, ChecklistChangeKind};
use notify::event::{ModifyKind, RenameMode};
use notify::{
    Config as NotifyConfig, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};

const CHECKLIST_EVENT_NAME: &str = "checklists://changed";
const WATCH_DEBOUNCE_MS: u64 = 150;

#[derive(Default)]
pub struct ChecklistWatcherState {
    inner: Mutex<Option<ChecklistWatcher>>,
}

struct ChecklistWatcher {
    _watcher: RecommendedWatcher,
}

pub fn restart(app: &AppHandle) -> Result<(), String> {
    let repo = resolve_data_repo(app)?;
    let state = app.state::<ChecklistWatcherState>();
    let watcher = ChecklistWatcher::new(app.clone(), repo)?;
    let mut guard = state
        .inner
        .lock()
        .map_err(|_| "Failed to lock watcher state".to_string())?;
    *guard = Some(watcher);
    Ok(())
}

pub fn clear(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<ChecklistWatcherState>();
    let mut guard = state
        .inner
        .lock()
        .map_err(|_| "Failed to lock watcher state".to_string())?;
    *guard = None;
    Ok(())
}

impl ChecklistWatcher {
    fn new(app: AppHandle, repo: PathBuf) -> Result<Self, String> {
        let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
        let mut watcher = RecommendedWatcher::new(
            move |result| {
                let _ = tx.send(result);
            },
            NotifyConfig::default(),
        )
        .map_err(|e| format!("Failed to create checklist watcher: {e}"))?;

        watcher
            .watch(&repo, RecursiveMode::Recursive)
            .map_err(|e| format!("Failed to watch {}: {e}", repo.display()))?;

        std::thread::spawn(move || {
            let mut pending = Vec::new();

            loop {
                let next = if pending.is_empty() {
                    match rx.recv() {
                        Ok(event) => event,
                        Err(_) => break,
                    }
                } else {
                    match rx.recv_timeout(Duration::from_millis(WATCH_DEBOUNCE_MS)) {
                        Ok(event) => event,
                        Err(mpsc::RecvTimeoutError::Timeout) => {
                            emit_pending(&app, &repo, &mut pending);
                            continue;
                        }
                        Err(mpsc::RecvTimeoutError::Disconnected) => {
                            emit_pending(&app, &repo, &mut pending);
                            break;
                        }
                    }
                };

                pending.push(next);
                let started = Instant::now();

                while started.elapsed() < Duration::from_millis(WATCH_DEBOUNCE_MS) {
                    let remaining =
                        Duration::from_millis(WATCH_DEBOUNCE_MS).saturating_sub(started.elapsed());
                    match rx.recv_timeout(remaining) {
                        Ok(event) => pending.push(event),
                        Err(mpsc::RecvTimeoutError::Timeout) => break,
                        Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    }
                }

                emit_pending(&app, &repo, &mut pending);
            }
        });

        Ok(Self { _watcher: watcher })
    }
}

fn emit_pending(app: &AppHandle, repo: &Path, pending: &mut Vec<notify::Result<Event>>) {
    let mut deduped = HashSet::new();

    for item in pending.drain(..) {
        match item {
            Ok(event) => {
                for change in map_event(repo, &event) {
                    deduped.insert(change);
                }
            }
            Err(err) => {
                log::warn!("Checklist watcher error: {err}");
            }
        }
    }

    for change in deduped {
        if let Err(err) = app.emit(CHECKLIST_EVENT_NAME, change) {
            log::warn!("Failed to emit checklist change event: {err}");
        }
    }
}

fn map_event(repo: &Path, event: &Event) -> Vec<ChecklistChangeEvent> {
    match &event.kind {
        EventKind::Create(_) => map_paths(repo, &event.paths, ChecklistChangeKind::Created),
        EventKind::Remove(_) => map_paths(repo, &event.paths, ChecklistChangeKind::Deleted),
        EventKind::Modify(ModifyKind::Name(RenameMode::Both)) => {
            if event.paths.len() >= 2 {
                let mut changes = Vec::new();
                changes.extend(map_paths(
                    repo,
                    &event.paths[..1],
                    ChecklistChangeKind::Deleted,
                ));
                changes.extend(map_paths(
                    repo,
                    &event.paths[1..2],
                    ChecklistChangeKind::Created,
                ));
                changes
            } else {
                map_paths(repo, &event.paths, ChecklistChangeKind::Updated)
            }
        }
        EventKind::Modify(ModifyKind::Name(_)) => {
            map_paths(repo, &event.paths, ChecklistChangeKind::Updated)
        }
        EventKind::Modify(_) => map_paths(repo, &event.paths, ChecklistChangeKind::Updated),
        _ => Vec::new(),
    }
}

fn map_paths(
    repo: &Path,
    paths: &[PathBuf],
    kind: ChecklistChangeKind,
) -> Vec<ChecklistChangeEvent> {
    let checklists_dir = repo.join("checklists");

    paths
        .iter()
        .filter_map(|path| {
            to_slug(&checklists_dir, path).map(|slug| ChecklistChangeEvent { kind, slug })
        })
        .collect()
}

fn to_slug(checklists_dir: &Path, path: &Path) -> Option<String> {
    if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
        return None;
    }

    if path.parent()? != checklists_dir {
        return None;
    }

    path.file_stem()
        .and_then(|stem| stem.to_str())
        .map(|slug| slug.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use notify::event::{CreateKind, RemoveKind};

    #[test]
    fn maps_markdown_event_to_slug() {
        let repo = PathBuf::from("/tmp/checkmate-data");
        let event = Event {
            kind: EventKind::Create(CreateKind::File),
            paths: vec![repo.join("checklists/deploy-node.md")],
            attrs: Default::default(),
        };

        let mapped = map_event(&repo, &event);
        assert_eq!(
            mapped,
            vec![ChecklistChangeEvent {
                kind: ChecklistChangeKind::Created,
                slug: "deploy-node".to_string(),
            }]
        );
    }

    #[test]
    fn ignores_non_checklist_paths() {
        let repo = PathBuf::from("/tmp/checkmate-data");
        let event = Event {
            kind: EventKind::Remove(RemoveKind::File),
            paths: vec![repo.join("runs/deploy-node/run.md")],
            attrs: Default::default(),
        };

        assert!(map_event(&repo, &event).is_empty());
    }
}
