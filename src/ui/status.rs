//! The status bar.
use super::*;

pub(crate) fn draw_status(f: &mut Frame, app: &App, t: &Theme, area: Rect) {
    let claude = app.panel && app.focus == Focus::Claude && matches!(app.mode, Mode::Read | Mode::Write);
    let (badge, color, keys) = status_keys(app, t, claude);
    let mut spans = vec![Span::raw("  "), Span::styled(format!(" {badge} "), Style::new().bg(color).fg(t.bg).add_modifier(Modifier::BOLD)), Span::raw("  ")];
    for (k, v) in keys {
        spans.push(Span::styled(k, Style::new().fg(t.fg)));
        spans.push(Span::styled(format!(" {v}   "), Style::new().fg(t.dim)));
    }
    let right = status_right(app, claude);
    let used: usize = spans.iter().map(|s| s.content.width()).sum();
    let pad = (area.width as usize).saturating_sub(used + right.width() + 2);
    spans.push(Span::raw(" ".repeat(pad)));
    spans.push(Span::styled(right, Style::new().fg(t.dim)));
    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

type Hints = Vec<(String, String)>;

/// The badge of the pane in focus and its key hints, from the keymap: the first key of each action.
fn status_keys(app: &App, t: &Theme, claude: bool) -> (&'static str, Color, Hints) {
    use crate::keys::Pane;
    let delta = app.delta_open && app.focus == Focus::Delta && matches!(app.mode, Mode::Read | Mode::Write);
    let tree = app.state.tree && app.focus == Focus::Tree && matches!(app.mode, Mode::Read | Mode::Write);
    let hints = |p: Pane, list: &[(&str, &str)]| -> Hints {
        list.iter().filter(|(a, _)| app.agent_can(a)).map(|(a, text)| (app.keys.first(p, a), text.to_string())).filter(|(k, _)| !k.is_empty()).collect()
    };
    let next = (app.keys.first(Pane::Global, "next_pane"), "next pane".to_string());
    let with_next = |mut v: Hints| {
        v.push(next.clone());
        v
    };
    if tree {
        return ("FILES", t.acc, with_next(hints(Pane::Tree, &[("down", "move"), ("open", "open"), ("view", "view"), ("search", "name"), ("add_to_chat", "to chat"), ("rename", "rename"), ("move", "move to"), ("delete", "delete"), ("close", "close")])));
    }
    if delta {
        return ("DELTA", t.acc2, with_next(hints(Pane::Delta, &[("base", "other base"), ("ask", "ask claude"), ("recap", "recap"), ("close", "close")])));
    }
    if claude {
        return ("CLAUDE", t.acc, with_next(hints(Pane::Chat, &[("send", "send"), ("newline", "new line"), ("new_thread", "new thread"), ("handoff", "Claude Code"), ("back", "back")])));
    }
    match app.mode {
        Mode::Read if app.read_anchor.is_some() => ("SELECT", t.acc2, hints(Pane::Read, &[("select_down", "extend"), ("ask", "ask claude"), ("add_to_chat", "add to chat"), ("copy", "copy"), ("back", "cancel")])),
        Mode::Read | Mode::Help => {
            let mut v = hints(Pane::Read, &[("down", "cursor"), ("select_down", "select"), ("ask", "ask"), ("say", "say here"), ("add_to_chat", "to chat"), ("chat", "chat"), ("write", "write"), ("search", "search"), ("help", "keys")]);
            // The four cursor keys and the four select keys, each with one label.
            let four = |names: [&str; 4]| group_label(&names.map(|n| app.keys.first(Pane::Read, n)).iter().map(String::as_str).collect::<Vec<_>>());
            v[0].0 = four(["left", "down", "up", "right"]);
            v[1].0 = four(["select_left", "select_down", "select_up", "select_right"]);
            v.insert(5, (app.keys.first(Pane::Global, "toggle_tree"), "files".into()));
            if app.over_limit.is_some() {
                v.insert(0, (app.keys.first(Pane::Read, "split"), format!("split: {} words", app.word_count)));
            }
            ("READ", t.acc, v)
        }
        Mode::Write => (
            "WRITE",
            t.acc2,
            hints(Pane::Write, &[("done", "read"), ("chat", "chat"), ("add_to_chat", "to chat"), ("say", "say here"), ("bold", "bold"), ("italic", "italic"), ("underline", "underline"), ("toggle_tree", "files"), ("delta", "delta")]),
        ),
        Mode::Notes => ("NOTES", t.acc, hints(Pane::Notes, &[("down", "move"), ("open", "open"), ("new", "new"), ("rename", "rename"), ("tags", "by tag"), ("close", "close")])),
        Mode::Search => ("SEARCH", t.acc, vec![("↑ ↓".into(), "move".into()), ("enter".into(), "open at match".into()), ("esc".into(), "back".into())]),
        Mode::Theme => ("THEME", t.acc, vec![("j k".into(), "preview".into()), ("enter".into(), "keep".into()), ("esc".into(), "cancel".into())]),
    }
}

/// The right side: a flash, the sync, the agent, or the words of the note.
fn status_right(app: &App, claude: bool) -> String {
    if let Some((msg, _)) = &app.flash {
        return msg.clone();
    }
    if let Some(s) = app.sync_label() {
        return s;
    }
    if claude {
        let key = app.key();
        let pending = app.threads.get(&key).and_then(|th| th.pending()).is_some();
        return if pending { "1 proposal open".into() } else if app.agent_name.is_some() { app.agent_short() } else { String::new() };
    }
    match app.mode {
        Mode::Write => {
            // The words of prose, as the word limit counts them.
            let words = app.word_count;
            let saved = match (app.editor.dirty, app.saved_at, &app.editor_path) {
                (true, _, _) => "editing".to_string(),
                (false, Some(at), _) => format!("saved {}", at.format("%H:%M")),
                (false, None, Some(_)) => "saved".into(),
                (false, None, None) => "not saved yet".into(),
            };
            format!("{words} words · {saved}")
        }
        Mode::Notes | Mode::Search => format!("{} notes · {}", app.notes.len(), app.rel(&app.cfg.notes_dir)),
        Mode::Theme => THEMES[app.theme].name.to_string(),
        _ if app.note().is_some() && app.word_limit > 0 => format!("{} / {} words", app.word_count, app.word_limit),
        _ => app.note().map(|_| format!("{} words", app.word_count)).unwrap_or_default(),
    }
}

/// The line under the status bar while a question about a file is open: `delete a.md? y / n`, or a
/// field for a name or a folder, with the folder that `tab` completes to.
pub(crate) fn draw_file_prompt(f: &mut Frame, app: &App, t: &Theme, area: Rect) {
    use crate::app::FilePrompt;
    let Some(p) = &app.file_prompt else { return };
    let name = |rel: &std::path::Path| rel.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let (badge, ask) = match p {
        FilePrompt::Delete(rel) => ("DELETE", format!("delete {}? ", name(rel))),
        FilePrompt::Rename(rel, _) => ("RENAME", format!("{} → ", name(rel))),
        FilePrompt::Move(rel, _) => ("MOVE", format!("{} to folder ", name(rel))),
        FilePrompt::NewFile(_) => ("NEW FILE", String::new()),
        FilePrompt::NewFolder(_) => ("NEW FOLDER", String::new()),
    };
    let mut spans = vec![Span::raw("  "), Span::styled(format!(" {badge} "), Style::new().bg(t.acc2).fg(t.bg).add_modifier(Modifier::BOLD)), Span::raw("  "), Span::styled(ask.clone(), Style::new().fg(t.fg))];
    let used = 2 + badge.width() + 2 + 2 + ask.width();
    let Some(text) = p.field() else {
        spans.push(Span::styled("y / n", Style::new().fg(t.dim)));
        return f.render_widget(Paragraph::new(Line::from(spans)), area);
    };
    let ghost = app.complete_folder(text).and_then(|c| c.strip_prefix(text.as_str()).map(str::to_string)).unwrap_or_default();
    spans.push(Span::styled(text.clone(), Style::new().fg(t.fg).bg(t.panel)));
    spans.push(Span::styled(format!("{ghost} "), Style::new().fg(t.dim).bg(t.panel)));
    let tab = if ghost.is_empty() { "" } else { "   tab complete" };
    spans.push(Span::styled(format!("{tab}   enter ok   esc cancel"), Style::new().fg(t.dim)));
    f.set_cursor_position((area.x + (used + text.width()) as u16, area.y));
    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

/// One label for a group of keys: `h j k l` → `h/j/k/l`, `^⇧h ^⇧j` → `^⇧h/j`.
pub fn group_label(labels: &[&str]) -> String {
    let first: Vec<char> = labels.first().map(|l| l.chars().collect()).unwrap_or_default();
    let prefix = first.len().saturating_sub(1);
    let same = labels.iter().all(|l| l.chars().count() == first.len() && l.chars().take(prefix).eq(first[..prefix].iter().copied()));
    if !same {
        return labels.join("/");
    }
    let tails: Vec<String> = labels.iter().map(|l| l.chars().skip(prefix).collect()).collect();
    format!("{}{}", first[..prefix].iter().collect::<String>(), tails.join("/"))
}
