//! `zendo notes`: move notes into the notes repo, and consolidate them by origin.
use super::*;

/// `zendo notes move` and `zendo notes consolidate`.
pub async fn notes(paths: &Paths, agent_flag: Option<String>, action: &str) -> bool {
    let mut o = Out::new();
    o.head(&format!("zendo notes {action}"));
    let Some(n) = UserConfig::load().notes else {
        o.fail("notes repo", "off");
        o.hint("Turn it on: zendo setup --notes-repo <folder or git URL>");
        return false;
    };
    match action {
        "move" => move_notes(paths, agent_flag, &n, &mut o).await,
        _ => consolidate(paths, agent_flag, &n, &mut o).await,
    }
}

pub(super) fn list(files: &[PathBuf]) -> String {
    files.iter().map(|f| format!("- {}", f.display())).collect::<Vec<_>>().join("\n")
}

/// Delete the notes that Claude emptied; the count.
pub(super) fn delete_emptied(files: &[PathBuf]) -> usize {
    files.iter().filter(|f| std::fs::read_to_string(f).is_ok_and(|t| t.trim().is_empty()) && std::fs::remove_file(f).is_ok()).count()
}

pub(super) fn inside(p: &Path, dir: &Path) -> bool {
    let canon = |p: &Path| p.canonicalize().ok().or_else(|| p.parent().and_then(|d| d.canonicalize().ok()).zip(p.file_name()).map(|(d, n)| d.join(n)));
    match (canon(p), dir.canonicalize().ok()) {
        (Some(p), Some(d)) => p.starts_with(d),
        _ => false,
    }
}

/// Commit the notes repo, then pull and push, and say how it went.
pub(super) fn sync_now(n: &config::NotesRepo, message: &str, o: &mut Out) {
    match crate::setup::sync::commit(&n.path, &n.path, message) {
        Err(e) => o.fail("commit", e),
        Ok(_) => match crate::setup::sync::pull_push(&n.path) {
            crate::setup::sync::State::Synced => o.ok("sync", "pushed"),
            s => o.warn("sync", s.label().unwrap_or_default()),
        },
    }
}

pub(super) async fn move_notes(paths: &Paths, agent_flag: Option<String>, n: &config::NotesRepo, o: &mut Out) -> bool {
    let files: Vec<PathBuf> = crate::setup::sync::notes_in(&paths.repo_notes).into_iter().filter(|f| std::fs::read_to_string(f).is_ok_and(|t| !t.trim().is_empty())).collect();
    if files.is_empty() {
        o.ok("move", format!("no notes in {}", paths.repo_notes.display()));
        return true;
    }
    let to = match n.prepare(&paths.root) {
        Ok(d) => d,
        Err(e) => {
            o.fail("notes repo", e.to_string());
            return false;
        }
    };
    o.ok("move", format!("{} notes from {} to {}", files.len(), paths.repo_notes.display(), to.display()));
    let prompt = MOVE_PROMPT.replace("{{TO}}", &to.display().to_string()).replace("{{ROOT}}", &paths.root.display().to_string()).replace("{{MACHINE}}", &n.machine).replace("{{FILES}}", &list(&files));
    let allowed = |p: &Path| inside(p, &to) || files.iter().any(|f| same_file(p, f));
    let Some(answer) = claude_task(&paths.root, agent_flag, "notes-move", prompt, &allowed, ("only the target folder and the listed notes may change", "is moving the notes…"), o).await else {
        return false;
    };
    report_answer(&answer, o);
    let gone = delete_emptied(&files);
    o.ok("moved", format!("{gone} of {} notes; {} left in {}", files.len(), files.len() - gone, paths.repo_notes.display()));
    if gone > 0 {
        o.hint("The moved notes are gone from this repository: commit that change here.");
    }
    sync_now(n, &format!("notes: move {} notes of {} into {}", gone, paths.root.display(), n.machine), o);
    true
}

pub(super) async fn consolidate(paths: &Paths, agent_flag: Option<String>, n: &config::NotesRepo, o: &mut Out) -> bool {
    let Some(origin) = crate::setup::sync::origin_of(&paths.root) else {
        o.fail("origin", format!("{} has no git origin", paths.root.display()));
        o.hint("Consolidation finds the folders of one project by its origin. Run it in a repository with `origin`.");
        return false;
    };
    let to = match n.prepare(&paths.root) {
        Ok(d) => d,
        Err(e) => {
            o.fail("notes repo", e.to_string());
            return false;
        }
    };
    // Get the folders of the other machines first.
    if let crate::setup::sync::State::Conflict(e) = crate::setup::sync::pull_push(&n.path) {
        o.fail("sync", format!("conflict in the notes repo: {e}"));
        return false;
    }
    let here = to.canonicalize().unwrap_or(to.clone());
    let sources: Vec<(PathBuf, crate::setup::sync::Location, Vec<PathBuf>)> = crate::setup::sync::locations(&n.path)
        .into_iter()
        .filter(|(d, l)| l.origin.as_deref() == Some(origin.as_str()) && d.canonicalize().unwrap_or(d.clone()) != here)
        .map(|(d, l)| {
            let f = crate::setup::sync::notes_in(&d);
            (d, l, f)
        })
        .filter(|(_, _, f)| !f.is_empty())
        .collect();
    if sources.is_empty() {
        o.ok("consolidate", format!("{origin}: no other folders with notes"));
        return true;
    }
    let files: Vec<PathBuf> = sources.iter().flat_map(|(_, _, f)| f.clone()).collect();
    o.ok("consolidate", format!("{origin}: {} notes in {} other folders into {}", files.len(), sources.len(), to.display()));
    let text = sources.iter().map(|(d, l, f)| format!("{} (machine {}, repository at {}):\n{}", d.display(), l.machine, l.path, list(f))).collect::<Vec<_>>().join("\n\n");
    let prompt = CONSOLIDATE_PROMPT
        .replace("{{ORIGIN}}", &origin)
        .replace("{{TO}}", &to.display().to_string())
        .replace("{{ROOT}}", &paths.root.display().to_string())
        .replace("{{MACHINE}}", &n.machine)
        .replace("{{SOURCES}}", &text);
    let allowed = |p: &Path| inside(p, &to) || files.iter().any(|f| same_file(p, f));
    let Some(answer) = claude_task(&paths.root, agent_flag, "notes-consolidate", prompt, &allowed, ("only the target folder and the listed notes may change", "is consolidating the notes…"), o).await else {
        return false;
    };
    report_answer(&answer, o);
    let gone = delete_emptied(&files);
    // A folder with no notes left: remove its location file, so that it does not come up again.
    for (d, _, _) in &sources {
        if crate::setup::sync::notes_in(d).is_empty() {
            let _ = std::fs::remove_file(d.join(crate::setup::sync::LOCATION_FILE));
            let _ = std::fs::remove_dir(d);
        }
    }
    o.ok("consolidated", format!("{gone} of {} notes moved into {}", files.len(), to.display()));
    sync_now(n, &format!("notes: consolidate {origin} into {} {}", n.machine, paths.root.display()), o);
    true
}

pub(super) fn report_answer(answer: &str, o: &mut Out) {
    let a = answer.trim();
    if !a.is_empty() {
        o.line("›", "36", "claude", a.lines().next().unwrap_or(""));
        o.more(a.lines().skip(1).take(6).collect::<Vec<_>>().join("\n"));
    }
}
