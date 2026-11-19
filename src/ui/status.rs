//! The status bar.
use super::*;

/// The chat pane has the keys.
fn claude(app: &App) -> bool {
    app.threads.shown() && app.focus == Focus::Claude && matches!(app.mode, Mode::Read | Mode::Write)
}

/// The badge, its color, and the key hints with the key of the help first: it is always on the
/// screen. An action without a key has no hint (PANES-29).
fn bar_keys(app: &App, fr: &Cache, t: &Theme) -> (&'static str, Color, Hints) {
    let (badge, color, mut keys) = status_keys(app, fr, t, claude(app));
    keys.retain(|(k, v, _)| v != "keys" && !k.is_empty());
    if let Some(help) = app.keys.key(crate::keys::Pane::Global, "help") {
        keys.insert(0, (help, "keys".into(), vec![(crate::keys::Pane::Global, "help")]));
    }
    (badge, color, keys)
}

/// The status bar, one line: the question of a file takes its place while it is open (PANES-49).
pub(crate) fn draw_bottom(f: &mut Frame, app: &App, fr: &mut Cache, t: &Theme, area: Rect) {
    if app.files.prompt().is_some() {
        draw_file_prompt(f, app, t, area);
    } else {
        draw_status(f, app, fr, t, area);
    }
}

/// The status bar: the key of the help, the key hints, a tip with room, and the right side whole,
/// a message or else the state (PANES-26, PANES-27, TIPS-6).
pub(crate) fn draw_status(f: &mut Frame, app: &App, fr: &mut Cache, t: &Theme, area: Rect) {
    let (badge, color, keys) = bar_keys(app, fr, t);
    let mut spans = vec![Span::raw("  "), Span::styled(format!(" {badge} "), Style::new().bg(color).fg(t.bg).add_modifier(Modifier::BOLD)), Span::raw("  ")];
    let message = app.flash.as_ref().map(|(m, _)| m.clone());
    let right = message.unwrap_or_else(|| status_right(app, fr, claude(app)));
    // The right side shows whole: the hints at the end give way, the help key stays.
    let room = (area.width as usize).saturating_sub(right.width() + 2 + spans.iter().map(|s| s.content.width()).sum::<usize>());
    let fit = keys.iter().scan(0, |w, (k, v, _)| {
        *w += k.width() + v.width() + 4;
        Some(*w)
    });
    let n = fit.take_while(|w| *w <= room).count().max(1);
    let mut x = area.x + spans.iter().map(|s| s.content.width()).sum::<usize>() as u16;
    for (k, v, acts) in keys.into_iter().take(n) {
        // Each key of the hint is a target of a click; the key that a click made a button shows
        // as one (REKEY-1).
        for (rect, target) in super::rekey::hint_targets(app, &k, &acts, x, area.y) {
            fr.key_targets.push((rect, target));
        }
        let button = app.rekey.button().is_some_and(|b| fr.key_targets.iter().any(|(r, t)| *t == b && r.x >= x && r.x < x + k.width() as u16));
        let style = if button { Style::new().fg(t.bg).bg(t.acc2) } else { Style::new().fg(t.fg) };
        x += (k.width() + v.width() + 4) as u16;
        spans.push(Span::styled(k, style));
        spans.push(Span::styled(format!(" {v}   "), Style::new().fg(t.dim)));
    }
    let used: usize = spans.iter().map(|s| s.content.width()).sum();
    let mut pad = (area.width as usize).saturating_sub(used + right.width() + 2);
    // A tip after the hints, when no message shows and it has room: it gives way first (TIPS-6).
    let tip = app.tips.status();
    if app.flash.is_none() && pad >= tip.width() + 3 {
        spans.push(Span::styled(tip, Style::new().fg(t.dim).add_modifier(Modifier::ITALIC)));
        pad -= tip.width();
    }
    spans.push(Span::raw(" ".repeat(pad)));
    spans.push(Span::styled(right, Style::new().fg(t.dim)));
    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

/// The hints: the key, the text, and the actions of the key, one for each key of a group.
type Hints = Vec<(String, String, Vec<(crate::keys::Pane, &'static str)>)>;

/// The badge of the pane in focus and its key hints, from the keymap: the first key of each action.
fn status_keys(app: &App, fr: &Cache, t: &Theme, claude: bool) -> (&'static str, Color, Hints) {
    use crate::keys::Pane;
    let delta = app.work.delta_open() && app.focus == Focus::Delta && matches!(app.mode, Mode::Read | Mode::Write);
    let tree = app.state.tree && app.focus == Focus::Tree && matches!(app.mode, Mode::Read | Mode::Write);
    let hints = |p: Pane, list: &[(&'static str, &str)]| hints(app, p, list);
    let one = |p: Pane, a: &'static str, text: &str| one(app, p, a, text);
    let next = one(Pane::Global, "next_pane", "next pane");
    let with_next = |mut v: Hints| {
        v.push(next.clone());
        v
    };
    if app.pointer.sel().is_some_and(|s| !s.done) {
        return mouse_selection(t);
    }
    if tree {
        return ("FILES", t.acc, with_next(hints(Pane::Tree, &[("down", "move"), ("open", "open"), ("view", "view"), ("search", "name"), ("add_to_chat", "to chat"), ("rename", "rename"), ("move", "move to"), ("delete", "delete"), ("close", "close")])));
    }
    if delta {
        return ("DELTA", t.acc2, with_next(hints(Pane::Delta, &[("base", "other base"), ("ask", "ask claude"), ("recap", "recap"), ("close", "close")])));
    }
    if claude && app.pane_terminal().is_some_and(|n| n >= crate::app::SESSIONS) {
        // A shell in the chat pane gets every other key (PANES-33, TERM-38).
        let v = [(Pane::Chat, "handoff", "full screen"), (Pane::Session, "leave", "back"), (Pane::Global, "next_session", "next session")];
        return ("SHELL", t.acc, v.iter().map(|(p, a, text)| one(*p, a, text)).collect());
    }
    if claude && app.threads.transcript().on() {
        let mut v = hints(Pane::Transcript, &[("down", "move"), ("select_down", "select"), ("copy", "copy"), ("add_to_chat", "to chat"), ("input", "input")]);
        if app.threads.transcript().block().is_some() {
            // The cursor is in a shell block: F5 puts it into a terminal, alt-F5 runs it (TERM-1, TERM-21).
            v.insert(0, one(Pane::Global, "terminal_run", "run"));
            v.insert(0, one(Pane::Global, "terminal", "to shell"));
        }
        return ("TRANSCRIPT", t.acc2, with_next(v));
    }
    // Text is selected where the user writes: the keys of `[selection]` first (PANES-37).
    let selected = |e: &crate::text::editor::Editor| if e.selection().is_some() { hints(Pane::Selected, &[("copy", "copy"), ("cut", "cut")]) } else { Vec::new() };
    if claude {
        let mut v = selected(app.composer().input());
        v.extend(hints(Pane::Chat, &[("send", "send"), ("newline", "new line"), ("new_thread", "new thread"), ("handoff", "Claude Code"), ("back", "back")]));
        v.push(one(Pane::Global, "next_session", "next session"));
        return ("CHAT", t.acc, with_next(v));
    }
    match app.mode {
        Mode::Read if app.reader.anchor().is_some() => ("SELECT", t.acc2, hints(Pane::Read, &[("select_down", "extend"), ("ask", "ask claude"), ("add_to_chat", "add to chat"), ("copy", "copy"), ("back", "cancel")])),
        Mode::Read | Mode::Help => ("READ", t.acc, read_hints(app, fr)),
        Mode::Write => {
            let mut v = selected(app.writer.editor());
            v.extend(hints(Pane::Write, &[("done", "read"), ("chat", "chat"), ("add_to_chat", "to chat"), ("say", "say here"), ("bold", "bold"), ("italic", "italic"), ("underline", "underline"), ("toggle_tree", "files"), ("delta", "delta")]));
            ("WRITE", t.acc2, v)
        }
        Mode::Find => ("FIND", t.acc, list_keys(app, "open")),
        Mode::Theme => {
            let mut v = hints(Pane::Theme, &[("down", "preview"), ("keep", "keep")]);
            v.push(("esc".into(), "cancel".into(), Vec::new()));
            ("THEME", t.acc, v)
        }
    }
}

/// The hints of the actions of `list` of pane `p` that the agent can do.
fn hints(app: &App, p: crate::keys::Pane, list: &[(&'static str, &str)]) -> Hints {
    list.iter().filter(|(a, _)| app.agent_can(a)).map(|(a, text)| one(app, p, a, text)).collect()
}

/// The hint of one action: its first key and `text`.
fn one(app: &App, p: crate::keys::Pane, a: &'static str, text: &str) -> (String, String, Vec<(crate::keys::Pane, &'static str)>) {
    (app.keys.first(p, a), text.to_string(), vec![(p, a)])
}

/// The hints of read mode: the four cursor keys and the four select keys each with one label.
fn read_hints(app: &App, fr: &Cache) -> Hints {
    use crate::keys::Pane;
    let mut v = hints(app, Pane::Read, &[("down", "cursor"), ("select_down", "select"), ("ask", "ask"), ("say", "say here"), ("add_to_chat", "to chat"), ("chat", "chat"), ("write", "write"), ("search", "search"), ("help", "keys")]);
    let four = |names: [&'static str; 4]| {
        let keys = names.map(|n| app.keys.first(Pane::Read, n));
        let label = if keys.iter().any(String::is_empty) { String::new() } else { group_label(&keys.iter().map(String::as_str).collect::<Vec<_>>()) };
        (label, names.map(|n| (Pane::Read, n)).to_vec())
    };
    (v[0].0, v[0].2) = four(["left", "down", "up", "right"]);
    (v[1].0, v[1].2) = four(["select_left", "select_down", "select_up", "select_right"]);
    v.insert(5, one(app, Pane::Global, "toggle_tree", "files"));
    if fr.over_limit.is_some() {
        v.insert(0, one(app, Pane::Read, "split", &format!("split: {} words", fr.word_count)));
    }
    v
}

/// A drag of the mouse: what the release does, and the keys that change it (PANES-45).
fn mouse_selection(t: &Theme) -> (&'static str, Color, Hints) {
    let v = [("release", "copy"), ("alt-release", "to chat"), ("ctrl", "whole lines"), ("any key", "cancel")];
    ("MOUSE SELECTION", t.acc2, v.iter().map(|(k, v)| (k.to_string(), v.to_string(), Vec::new())).collect())
}

/// The right side: a flash, the sync, the agent, or the words of the note.
/// The right side without a message: the sync, the agent, or the words of the note.
fn status_right(app: &App, fr: &Cache, claude: bool) -> String {
    if let Some(s) = app.reading().and_then(crate::worker::reading) {
        return s;
    }
    if let Some(s) = app.sync_label() {
        return s;
    }
    if app.structure_missing() && !claude {
        return format!("no {} · run zendo init", crate::docs::structure::FILE);
    }
    if claude {
        return if app.link.ready() { app.agent_short() } else { String::new() };
    }
    match app.mode {
        Mode::Write => {
            // The words of prose, as the word limit counts them.
            let words = fr.word_count;
            let saved = match (app.writer.editor().dirty, app.writer.saved_at(), app.writer.path()) {
                (true, _, _) => "editing".to_string(),
                (false, Some(at), _) => format!("saved {}", at.format("%H:%M")),
                (false, None, Some(_)) => "saved".into(),
                (false, None, None) => "not saved yet".into(),
            };
            format!("{words} words · {saved}")
        }
        Mode::Find => format!("{} files", app.files.find().items.len()),
        Mode::Theme => THEMES[app.theme].name.to_string(),
        _ if app.note().is_some() && fr.word_limit > 0 => format!("{} / {} words", fr.word_count, fr.word_limit),
        _ => app.note().map(|_| format!("{} words", fr.word_count)).unwrap_or_default(),
    }
}

/// The line of the status bar while a question about a file is open: `delete a.md? y / n`, or a
/// field for a name or a folder, with the folder that `tab` completes to.
pub(crate) fn draw_file_prompt(f: &mut Frame, app: &App, t: &Theme, area: Rect) {
    use crate::app::FilePrompt;
    let Some(p) = &app.files.prompt() else { return };
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
        let yes = app.keys.first(crate::keys::Pane::Question, "yes");
        spans.push(Span::styled(format!("{yes} / n"), Style::new().fg(t.dim)));
        return f.render_widget(Paragraph::new(Line::from(spans)), area);
    };
    let ghost = app.complete_folder(text.text()).and_then(|c| c.strip_prefix(text.text()).map(str::to_string)).unwrap_or_default();
    spans.extend(super::field::spans(text, Style::new().fg(t.fg).bg(t.panel), Style::new().fg(t.fg).bg(t.sel)));
    spans.push(Span::styled(format!("{ghost} "), Style::new().fg(t.dim).bg(t.panel)));
    let key = |a| app.keys.first(crate::keys::Pane::Question, a);
    let tab = if ghost.is_empty() { String::new() } else { format!("   {} complete", key("complete")) };
    spans.push(Span::styled(format!("{tab}   {} ok   esc cancel", key("ok")), Style::new().fg(t.dim)));
    f.set_cursor_position((area.x + (used + text.cursor_width()) as u16, area.y));
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

/// The keys of the search box and of the find panel, from `[list]`; `esc` is fixed (PANES-48).
fn list_keys(app: &App, open: &str) -> Hints {
    let mut v = hints(app, crate::keys::Pane::List, &[("down", "move"), ("open", open)]);
    v.push(("esc".into(), "back".into(), Vec::new()));
    v
}
