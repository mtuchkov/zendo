//! Fixes by the agent: the project structure file.
use super::*;

pub enum Problem {
    /// No structure file, but project-structure.md is there.
    Missing,
    Parse(String),
    Issues(Vec<crate::docs::structure::Issue>),
}

pub(super) fn counts(st: &crate::docs::structure::Structure) -> String {
    let n = |k: usize, one: &str, many: &str| format!("{k} {}", if k == 1 { one } else { many });
    format!("{}, {}", n(st.entries.len(), "entry", "entries"), n(st.main().count(), "main material", "main materials"))
}

pub fn structure_problem(root: &Path) -> Option<Problem> {
    let path = root.join(crate::docs::structure::FILE);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return root.join("project-structure.md").exists().then_some(Problem::Missing);
    };
    let r = crate::docs::structure::check(&text, root);
    match r.parsed {
        Err(e) => Some(Problem::Parse(e)),
        Ok(_) if !r.issues.is_empty() => Some(Problem::Issues(r.issues)),
        Ok(_) => None,
    }
}

/// Ask on a terminal; `false` when nobody can answer.
pub(super) fn ask_yes(question: &str) -> bool {
    use std::io::{IsTerminal, Write};
    if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
        return false;
    }
    print!("{question}");
    let _ = std::io::stdout().flush();
    let mut line = String::new();
    if std::io::stdin().read_line(&mut line).is_err() {
        return false;
    }
    let a = line.trim().to_lowercase();
    a.is_empty() || a == "y" || a == "yes"
}

pub(super) fn problem_text(p: &Problem) -> String {
    match p {
        Problem::Missing => "The file does not exist yet.".into(),
        Problem::Parse(e) => format!("The file does not parse as TOML:\n{e}"),
        Problem::Issues(list) => list.iter().map(|i| format!("- {}: {}", i.at, i.what)).collect::<Vec<_>>().join("\n"),
    }
}

/// Same file, also when one of the paths does not exist yet.
pub(super) fn same_file(a: &Path, b: &Path) -> bool {
    let canon = |p: &Path| match (p.parent().and_then(|d| d.canonicalize().ok()), p.file_name()) {
        (Some(d), Some(n)) => d.join(n),
        _ => p.to_path_buf(),
    };
    canon(a) == canon(b)
}

/// Fix the structure file: by zendo when no judgement is needed, else by Claude.
/// Claude may edit only the structure file; zendo refuses everything else.
pub(super) async fn fix_structure(paths: &Paths, agent_flag: Option<String>, problem: Problem, o: &mut Out) -> bool {
    let target = paths.root.join(crate::docs::structure::FILE);
    let before = std::fs::read_to_string(&target).ok();
    o.head(&format!("fix {}", crate::docs::structure::FILE));

    // A copy first, outside the repo.
    if let Some(text) = &before {
        let dir = config::data_dir().join("backups");
        let _ = std::fs::create_dir_all(&dir);
        let name = paths.root.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "repo".into());
        let copy = dir.join(format!("{name}-{}-{}", chrono::Local::now().format("%Y%m%d-%H%M%S"), crate::docs::structure::FILE));
        if std::fs::write(&copy, text).is_ok() {
            o.ok("copy", copy.display().to_string());
        }
    }

    // 1. What needs no judgement.
    let mut problem = problem;
    if let Some((fixed, changed)) = before.as_deref().and_then(crate::docs::structure::quick_fix) {
        if std::fs::write(&target, &fixed).is_ok() {
            o.ok("zendo fixed", format!("{} thing{}", changed.len(), if changed.len() == 1 { "" } else { "s" }));
            o.more(changed.join("\n"));
        }
        match structure_problem(&paths.root) {
            None => return report_fix(paths, before.as_deref(), None, o),
            Some(p) => problem = p,
        }
    }

    // 2. Claude, with the instructions from zendo's resources.
    let mode = if matches!(problem, Problem::Missing) {
        format!(
            "The file does not exist yet. Create it from {}: keep its words, put each folder of its tree and \
             tables into an entry, put its rules for agents into `claude`, and its front matter into [meta]. \
             Mark the main documents of the project as `main = true`, in reading order.",
            paths.root.join("project-structure.md").display()
        )
    } else {
        "The file exists. Fix it so that it parses and follows the format. Change only what is needed.".to_string()
    };
    let prompt = crate::docs::structure::FIX_PROMPT
        .replace("{{FILE}}", &target.display().to_string())
        .replace("{{MODE}}", &mode)
        .replace("{{PROBLEMS}}", &problem_text(&problem))
        .replace("{{TREE}}", &crate::docs::structure::folder_list(&paths.root))
        .replace("{{FORMAT}}", crate::docs::structure::FORMAT);

    let only = |p: &Path| same_file(p, &target);
    let rule = format!("only {} may change", crate::docs::structure::FILE);
    let Some(answer) = claude_task(&paths.root, agent_flag, "doctor-fix", prompt, &only, (&rule, "is fixing the file…"), o).await else {
        return false;
    };
    report_fix(paths, before.as_deref(), Some(answer.trim()), o)
}

/// Run one prompt with Claude and wait for the answer. Claude may read files. It may edit only the
/// paths that `allowed` accepts; zendo refuses every other edit and every command.
pub(super) async fn claude_task(root: &Path, agent_flag: Option<String>, key: &str, prompt: String, allowed: &dyn Fn(&Path) -> bool, (rule, doing): (&str, &str), o: &mut Out) -> Option<String> {
    let cfg = UserConfig::load();
    let env = config::agent_env(&cfg);
    let (cmd, _) = config::resolve_agent(agent_flag, &cfg);
    let (h, mut rx) = agent::spawn(cmd, env, root.to_path_buf(), Some(config::data_dir().join("doctor.log")));
    match tokio::time::timeout(Duration::from_secs(60), rx.recv()).await {
        Ok(Some(Event::Ready { agent, .. })) => o.ok("claude", format!("{agent} {doing}")),
        _ => {
            o.fail("claude", "the agent did not start");
            o.hint("Run `zendo doctor` to see why.");
            return None;
        }
    }
    h.send(Cmd::Prompt { key: key.into(), known: None, text: prompt, attach: vec![] });
    let mut answer = String::new();
    let mut seen = std::collections::BTreeSet::new();
    let end = Instant::now() + Duration::from_secs(600);
    loop {
        let left = end.saturating_duration_since(Instant::now());
        match tokio::time::timeout(left, rx.recv()).await {
            Ok(Some(Event::Text { text, .. })) => answer.push_str(&text),
            Ok(Some(Event::Tool { id, title, .. })) => {
                if !title.is_empty() && seen.insert(id) {
                    o.more(format!("· {title}"));
                }
            }
            Ok(Some(Event::Permission { request, title, kind, diffs, choices, .. })) => {
                let read = diffs.is_empty() && matches!(kind.as_str(), "read" | "search");
                let ok = read || (!diffs.is_empty() && diffs.iter().all(|d| allowed(&d.path)));
                let pick = choices.iter().find(|c| c.allow == ok && !c.always).or_else(|| choices.iter().find(|c| c.allow == ok));
                if read {
                } else if ok {
                    o.more(format!("✓ allowed: {title}"));
                } else {
                    o.more(format!("✗ refused ({rule}): {title}"));
                }
                h.send(Cmd::Answer { request, option: pick.map(|c| c.id.clone()) });
            }
            Ok(Some(Event::Blocked { title, .. })) => o.more(format!("✗ refused a command: {title}")),
            Ok(Some(Event::Error { text, .. })) => answer.push_str(&format!("\n(error: {text})")),
            Ok(Some(Event::TurnEnd { .. })) => break,
            Ok(Some(Event::Exited { text })) => {
                o.fail("claude", text);
                return None;
            }
            Ok(Some(_)) => {}
            Ok(None) | Err(_) => {
                o.fail("claude", "no answer in 10 minutes");
                return None;
            }
        }
    }
    Some(answer)
}

/// Show what changed, what Claude said, and check the file again.
pub(super) fn report_fix(paths: &Paths, before: Option<&str>, answer: Option<&str>, o: &mut Out) -> bool {
    let after = std::fs::read_to_string(paths.root.join(crate::docs::structure::FILE)).ok();
    if let Some(a) = answer.filter(|a| !a.is_empty()) {
        o.line("›", "36", "claude", a.lines().next().unwrap_or(""));
        o.more(a.lines().skip(1).take(6).collect::<Vec<_>>().join("\n"));
    }
    match (&after, before) {
        (Some(a), Some(b)) if a != b => {
            let d = crate::docs::delta::unified(b, a, crate::docs::structure::FILE);
            let lines: Vec<&str> = d.lines().skip(2).collect();
            o.ok("changed", format!("{} lines in the diff", lines.len()));
            o.more(lines.iter().take(30).copied().collect::<Vec<_>>().join("\n"));
            if lines.len() > 30 {
                o.more(format!("… and {} more lines", lines.len() - 30));
            }
        }
        (Some(a), None) => o.ok("created", format!("{} · {} lines", crate::docs::structure::FILE, a.lines().count())),
        (None, _) => {
            o.fail("structure", "the file is not there");
            return false;
        }
        _ => o.warn("changed", "nothing"),
    }
    match structure_problem(&paths.root) {
        None => {
            if let Some(Ok(st)) = crate::docs::structure::Structure::load(&paths.root) {
                o.ok("structure", format!("zendo understands it now · {}", counts(&st)));
            }
            true
        }
        Some(p) => {
            o.fail("structure", "zendo still does not understand it");
            o.more(problem_text(&p));
            o.hint("Run `zendo doctor --fix` again, or fix it by hand. The format is in resources/project-structure.md in the zendo repo.");
            false
        }
    }
}
