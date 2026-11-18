//! The file tree pane.
use super::*;
use crate::docs::tree::{Kind, Row, View};

pub(crate) fn draw_tree(f: &mut Frame, app: &mut App, fr: &mut Cache, t: &Theme, area: Rect) {
    fr.tree_rows.clear();
    fr.tree_fold.clear();
    f.render_widget(Block::new().style(Style::new().bg(t.panel)), area);
    // FILES-68: the strip of view letters on the left edge; the hint stands over the tree.
    let inner = Rect { x: area.x + super::tree_strip::STRIP_W, y: area.y, width: area.width.saturating_sub(super::tree_strip::STRIP_W + 1), height: area.height };
    draw_tree_body(f, app, fr, t, inner);
    super::tree_strip::draw_strip(f, app, fr, t, area);
}

/// The head, the rows, and the bottom of the tree.
fn draw_tree_body(f: &mut Frame, app: &mut App, fr: &mut Cache, t: &Theme, inner: Rect) {
    let w = inner.width as usize;
    let dim = Style::new().fg(t.dim);
    let foot = if app.files.tree().view.is_worktree() { worktree_foot(app, t, w) } else { tree_foot(app, t, w) };
    let foot_h = if foot.is_empty() { 0 } else { foot.len() as u16 + 1 };
    let [head, tools, body, foot_area] = Layout::vertical([Constraint::Length(1), Constraint::Length(1), Constraint::Min(1), Constraint::Length(foot_h)]).areas(inner);
    if foot_h > 0 {
        let mut lines = vec![Line::styled("─".repeat(w), Style::new().fg(t.line))];
        lines.extend(foot);
        f.render_widget(Paragraph::new(lines), foot_area);
    }
    tree_head(f, app, fr, t, head);
    // FILES-77: the Notes view has no folders to open.
    if app.files.tree().find.is_none() && app.files.tree().view != View::Notes {
        fold_buttons(f, fr, t, Rect { x: tools.right().saturating_sub(7), width: 7, ..tools });
    }

    let rows = app.files.tree().rows();
    if rows.is_empty() && app.files.tree().find.as_deref().is_some_and(|q| !q.trim().is_empty()) {
        f.render_widget(Paragraph::new(Line::styled(" No file name matches.", dim)), body);
        return;
    }
    if rows.is_empty() {
        let msg = match app.files.tree().view {
            // FILES-79: the walk of the text files runs on a thread.
            View::Text | View::Worktree if app.files.tree().walking() => "Reading the files…".to_string(),
            View::Text => "No text files.".to_string(),
            View::Docs => format!("No .md files under a docs folder.{}", app.keys.key(crate::keys::Pane::Tree, "view").map(|k| format!(" Press {k} to show all.")).unwrap_or_default()),
            View::Structure => "No main materials. Mark entries with main = true in project-structure.toml.".to_string(),
            // FILES-31: the agent makes its worktree when it changes code the first time.
            View::Worktree | View::WorktreeChanged if app.files.tree().worktree.is_none() => {
                format!("Agent session {} has no worktree yet: .worktrees/{} comes when its agent changes code.", app.state.session + 1, app.state.session + 1)
            }
            View::Worktree => "No text files in the worktree.".to_string(),
            // ADR 0009: git reads the changes on a thread, and its error shows.
            View::WorktreeChanged if let Some(e) = app.files.tree().error() => format!("Cannot read the changes against main: {e}"),
            View::WorktreeChanged if app.files.tree().scanning() => "Reading the changes against main…".to_string(),
            View::WorktreeChanged => "No changes against main.".to_string(),
            View::Notes => format!("No notes yet.{}", app.keys.key(crate::keys::Pane::Tree, "new_note").map(|k| format!(" Press {k}.")).unwrap_or_default()),
        };
        f.render_widget(Paragraph::new(markdown::wrap(vec![Span::styled(msg, dim)], w.saturating_sub(1), " ", " ")), body);
        return;
    }
    let h = body.height as usize;
    // Keep the selection in the rows, and in view.
    app.files.fit_tree(rows.len(), h);
    let current = app.files.current_in_tree();
    for (i, row) in rows.iter().enumerate().skip(app.files.tree().scroll).take(h) {
        let r = Rect { x: inner.x, y: body.y + (i - app.files.tree().scroll) as u16, width: inner.width, height: 1 };
        fr.tree_rows.push((r, i));
        let is_current = current.as_deref() == Some(row.rel.as_path());
        let mut line = tree_line(app, t, row, is_current, w);
        if i == app.files.tree().sel && app.focus == Focus::Tree {
            line = line.style(Style::new().bg(t.sel));
        }
        f.render_widget(Paragraph::new(line), r);
    }
}

/// The footer: what the selected item is, and the start of its rules for Claude.
fn tree_foot(app: &App, t: &Theme, w: usize) -> Vec<Line<'static>> {
    let dim = Style::new().fg(t.dim);
    let rows_now = app.files.tree().rows();
    let picked = rows_now.get(app.files.tree().sel.min(rows_now.len().saturating_sub(1))).map(|r| r.rel.clone());
    let mut foot: Vec<Line> = Vec::new();
    let (Some(st), Some(rel)) = (app.files.tree().structure.as_ref(), picked.as_ref()) else { return foot };
    let fw = w.saturating_sub(2);
    if let Some(about) = st.about(rel) {
        foot.extend(markdown::wrap(vec![Span::styled(about.trim().to_string(), Style::new().fg(t.fg))], fw, " ", " ").into_iter().take(4));
    }
    let rules: Vec<&str> = st.covering(rel).iter().filter_map(|e| e.claude.as_deref()).collect();
    if let Some(last) = rules.last() {
        let first = last.trim().split(['\n', '.']).next().unwrap_or("").trim();
        let text = format!("for Claude: {first}{}", if rules.len() > 1 { format!(" (+{} more)", rules.len() - 1) } else { String::new() });
        foot.extend(markdown::wrap(vec![Span::styled(text, dim.add_modifier(Modifier::ITALIC))], fw, " ", " ").into_iter().take(2));
    }
    foot
}

/// The footer of a worktree view: the recap of the whole diff of the worktree (FILES-28).
fn worktree_foot(app: &App, t: &Theme, w: usize) -> Vec<Line<'static>> {
    let dim = Style::new().fg(t.dim);
    let Some(wt) = app.files.tree().worktree.as_ref().map(|p| p.display().to_string()) else { return Vec::new() };
    let recap = app.state.files.get(&wt).and_then(|f| f.recap.as_ref());
    let writing = app.work.job().is_some_and(|j| j.rel == wt);
    let head = match recap {
        Some(r) => format!("RECAP · {}{}", r.at.format("%H:%M"), if writing { " · writing a new one" } else { "" }),
        None if writing => "RECAP · writing it".to_string(),
        None => "RECAP · comes when the worktree has changes".to_string(),
    };
    let mut foot = vec![Line::styled(format!(" {head}"), dim.add_modifier(Modifier::BOLD))];
    let text = recap.map(|r| r.text.as_str()).unwrap_or_default();
    let body = text.lines().flat_map(|l| markdown::wrap(vec![Span::styled(l.trim().to_string(), Style::new().fg(t.fg))], w.saturating_sub(2), " ", "   "));
    foot.extend(body.take(10));
    foot
}

/// The head line: the project and the view, or the file name filter while it is on.
fn tree_head(f: &mut Frame, app: &App, fr: &mut Cache, t: &Theme, head: Rect) {
    let dim = Style::new().fg(t.dim);
    let w = head.width as usize;
    if let Some(q) = app.files.tree().find.clone() {
        let n = app.files.tree().rows().len();
        let count = if q.trim().is_empty() { "enter open · esc".to_string() } else { format!("{n} · esc") };
        let room = (w.saturating_sub(9 + count.width())).max(1);
        // A long filter shows its end; the cursor stays in the field.
        let (text, x) = if q.width() > room {
            let tail: String = q.chars().rev().take(room).collect::<Vec<_>>().into_iter().rev().collect();
            (vec![Span::styled(tail, Style::new().fg(t.fg))], q.cursor_width().saturating_sub(q.width() - room))
        } else {
            (super::field::spans(&q, Style::new().fg(t.fg), Style::new().fg(t.fg).bg(t.sel)), q.cursor_width())
        };
        let line = Line::from([Span::styled("` ", Style::new().fg(t.acc))].into_iter().chain(text).collect::<Vec<_>>());
        f.render_widget(Paragraph::new(line), Rect { x: head.x + 1, width: head.width.saturating_sub(1), ..head });
        let bad = app.files.tree().find_regex && regex::Regex::new(&q).is_err();
        let button = super::find::draw_regex_button(f, app.files.tree().find_regex, bad, t, Rect { width: head.width + 2, ..head });
        fr.find_regex = Some(button);
        let right = button.x.saturating_sub(1);
        f.render_widget(Paragraph::new(Line::styled(count.clone(), dim)), Rect { x: right.saturating_sub(count.width() as u16), width: count.width() as u16, ..head });
        if app.focus == Focus::Tree {
            f.set_cursor_position((head.x + 3 + x as u16, head.y));
        }
        return;
    }
    let label = match app.files.tree().structured() {
        Some(st) => format!("{} · {}", st.project.name.clone().unwrap_or_else(|| "PROJECT".into()).to_uppercase(), st.main().count()),
        None => format!("FILES · {}", app.files.tree().files.len()),
    };
    let filt = format!("{} · {}", app.files.tree().filter_label(), app.keys.first(crate::keys::Pane::Tree, "view"));
    // The name of the project ends with `…` before it touches the name of the view.
    let room = w.saturating_sub(1 + filt.width() + 1);
    let label = if label.width() > room { label.chars().take(room.saturating_sub(1)).collect::<String>() + "…" } else { label };
    f.render_widget(Paragraph::new(Line::styled(label, dim.add_modifier(Modifier::BOLD))), Rect { x: head.x + 1, width: head.width.saturating_sub(1), ..head });
    f.render_widget(Paragraph::new(Line::styled(filt.clone(), dim)), Rect { x: head.right().saturating_sub(filt.width() as u16), width: filt.width() as u16, ..head });
}

/// Under the head, on the right: `[+]` opens every folder, `[-]` closes every folder (FILES-57).
fn fold_buttons(f: &mut Frame, fr: &mut Cache, t: &Theme, area: Rect) {
    let style = Style::new().fg(t.dim);
    for (i, (label, open)) in [("[+]", true), ("[-]", false)].into_iter().enumerate() {
        let r = Rect { x: area.x + 4 * i as u16, width: 3, ..area };
        f.render_widget(Paragraph::new(Line::styled(label, style)), r);
        fr.tree_fold.push((r, open));
    }
}

/// One row: the indent, the icon, the name, and `Δ` for a file that changed.
fn tree_line(app: &App, t: &Theme, row: &Row, is_current: bool, w: usize) -> Line<'static> {
    let indent = "  ".repeat(row.depth);
    let (icon, name_style) = match row.kind {
        // A main material of the project: first class, in the plain text color and bold.
        Kind::Dir { open } if row.section => (if open { "[-] " } else { "[+] " }, Style::new().fg(t.fg).add_modifier(Modifier::BOLD)),
        // `§` and `•`: every font of `zendo setup fonts` has them, so they take one cell.
        Kind::File if row.section && !is_current => (" §  ", Style::new().fg(t.fg).add_modifier(Modifier::BOLD)),
        Kind::Dir { open } => (if open { "[-] " } else { "[+] " }, Style::new().fg(t.dim)),
        Kind::File if is_current => (" •  ", Style::new().fg(t.fg).add_modifier(Modifier::BOLD)),
        Kind::File => ("    ", Style::new().fg(t.fg)),
        // FILES-69: a head of the Notes view.
        Kind::Head => ("", Style::new().fg(t.dim).add_modifier(Modifier::BOLD)),
    };
    // Markdown is the default: its extension shows only in the text view, where files of every kind are.
    let all = app.files.tree().view == View::Text || app.files.tree().view.is_worktree();
    let name = match row.kind {
        _ if row.section => row.name.clone(),
        Kind::Head => row.name.clone(),
        Kind::Dir { .. } => format!("{}/", row.name),
        Kind::File if all => row.name.clone(),
        Kind::File => row.name.trim_end_matches(".md").to_string(),
    };
    // FILES-35: a file that the worktree deleted is struck through.
    let name_style = if app.files.tree().deleted.contains(&row.rel) { Style::new().fg(t.dim).add_modifier(Modifier::CROSSED_OUT) } else { name_style };
    let changed = row.kind == Kind::File && app.files.tree().changed.contains(&row.rel);
    let mark = if changed { " Δ" } else { "" };
    let room = w.saturating_sub(indent.width() + 5 + mark.width());
    let name = if name.width() > room { name.chars().take(room.saturating_sub(1)).collect::<String>() + "…" } else { name };
    let pad = w.saturating_sub(1 + indent.width() + 4 + name.width() + mark.width());
    // The extension in the color of its kind of file.
    let notes = app.files.tree().view == View::Notes;
    let cut = if row.kind == Kind::File && !row.section && !notes { name.rfind('.').filter(|i| *i > 0) } else { None };
    let (stem, ext) = match cut {
        Some(i) => (name[..i].to_string(), name[i..].to_string()),
        None => (name, String::new()),
    };
    Line::from(vec![
        Span::raw(" "),
        Span::raw(indent),
        Span::styled(icon, Style::new().fg(if is_current { t.acc } else { t.dim })),
        Span::styled(stem, name_style),
        Span::styled(ext, name_style.fg(theme::ext_color(&t.ext, &row.name))),
        Span::raw(" ".repeat(pad)),
        Span::styled(mark, Style::new().fg(t.acc2)),
    ])
}
