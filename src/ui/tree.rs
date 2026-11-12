//! The file tree pane.
use super::*;
use crate::docs::tree::{Kind, Row, View};

pub(crate) fn draw_tree(f: &mut Frame, app: &mut App, t: &Theme, area: Rect) {
    app.tree_rows.clear();
    f.render_widget(Block::new().style(Style::new().bg(t.panel)), area);
    let inner = Rect { x: area.x + 1, y: area.y, width: area.width.saturating_sub(2), height: area.height };
    let w = inner.width as usize;
    let dim = Style::new().fg(t.dim);
    let foot = tree_foot(app, t, w);
    let foot_h = if foot.is_empty() { 0 } else { foot.len() as u16 + 1 };
    let [head, _, body, foot_area] = Layout::vertical([Constraint::Length(1), Constraint::Length(1), Constraint::Min(1), Constraint::Length(foot_h)]).areas(inner);
    if foot_h > 0 {
        let mut lines = vec![Line::styled("─".repeat(w), Style::new().fg(t.line))];
        lines.extend(foot);
        f.render_widget(Paragraph::new(lines), foot_area);
    }
    tree_head(f, app, t, head);

    let rows = app.tree.rows();
    if rows.is_empty() && app.tree.find.as_deref().is_some_and(|q| !q.trim().is_empty()) {
        f.render_widget(Paragraph::new(Line::styled(" No file name matches.", dim)), body);
        return;
    }
    if rows.is_empty() {
        let msg = match app.tree.view {
            View::All | View::Folders => "No .md files.",
            View::Text => "No text files.",
            View::Docs => "No .md files under a docs folder. Press v to show all.",
            View::Structure => "No main materials. Mark entries with main = true in project-structure.toml.",
        };
        f.render_widget(Paragraph::new(markdown::wrap(vec![Span::styled(msg, dim)], w.saturating_sub(1), " ", " ")), body);
        return;
    }
    app.tree.sel = app.tree.sel.min(rows.len() - 1);
    let h = body.height as usize;
    // Keep the selection in view.
    if app.tree.sel < app.tree.scroll {
        app.tree.scroll = app.tree.sel;
    } else if app.tree.sel >= app.tree.scroll + h {
        app.tree.scroll = app.tree.sel + 1 - h;
    }
    app.tree.scroll = app.tree.scroll.min(rows.len().saturating_sub(h));
    let current = app.current.as_ref().map(|p| app.store.rel(p).to_path_buf());
    for (i, row) in rows.iter().enumerate().skip(app.tree.scroll).take(h) {
        let r = Rect { x: inner.x, y: body.y + (i - app.tree.scroll) as u16, width: inner.width, height: 1 };
        app.tree_rows.push((r, i));
        let is_current = current.as_deref() == Some(row.rel.as_path());
        let mut line = tree_line(app, t, row, is_current, w);
        if i == app.tree.sel && app.focus == Focus::Tree {
            line = line.style(Style::new().bg(t.sel));
        }
        f.render_widget(Paragraph::new(line), r);
    }
}

/// The footer: what the selected item is, and the start of its rules for Claude.
fn tree_foot(app: &App, t: &Theme, w: usize) -> Vec<Line<'static>> {
    let dim = Style::new().fg(t.dim);
    let rows_now = app.tree.rows();
    let picked = rows_now.get(app.tree.sel.min(rows_now.len().saturating_sub(1))).map(|r| r.rel.clone());
    let mut foot: Vec<Line> = Vec::new();
    let (Some(st), Some(rel)) = (app.tree.structure.as_ref(), picked.as_ref()) else { return foot };
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

/// The head line: the project and the view, or the file name filter while it is on.
fn tree_head(f: &mut Frame, app: &App, t: &Theme, head: Rect) {
    let dim = Style::new().fg(t.dim);
    let w = head.width as usize;
    if let Some(q) = app.tree.find.clone() {
        let n = app.tree.rows().len();
        let count = if q.trim().is_empty() { "enter open · esc".to_string() } else { format!("{n} · esc") };
        let room = (w.saturating_sub(4 + count.width())).max(1);
        let shown: String = if q.width() > room { q.chars().rev().take(room).collect::<Vec<_>>().into_iter().rev().collect() } else { q.clone() };
        let line = Line::from(vec![Span::styled("/ ", Style::new().fg(t.acc)), Span::styled(shown.clone(), Style::new().fg(t.fg))]);
        f.render_widget(Paragraph::new(line), Rect { x: head.x + 1, width: head.width.saturating_sub(1), ..head });
        f.render_widget(Paragraph::new(Line::styled(count.clone(), dim)), Rect { x: head.right().saturating_sub(count.width() as u16), width: count.width() as u16, ..head });
        if app.focus == Focus::Tree {
            f.set_cursor_position((head.x + 3 + shown.width() as u16, head.y));
        }
        return;
    }
    let label = match app.tree.structured() {
        Some(st) => format!("{} · {}", st.project.name.clone().unwrap_or_else(|| "PROJECT".into()).to_uppercase(), st.main().count()),
        None => format!("FILES · {}", app.tree.files.len()),
    };
    f.render_widget(Paragraph::new(Line::styled(label, dim.add_modifier(Modifier::BOLD))), Rect { x: head.x + 1, width: head.width.saturating_sub(1), ..head });
    let filt = format!("{} · {}", app.tree.filter_label(), app.keys.first(crate::keys::Pane::Tree, "view"));
    f.render_widget(Paragraph::new(Line::styled(filt.clone(), dim)), Rect { x: head.right().saturating_sub(filt.width() as u16), width: filt.width() as u16, ..head });
}

/// One row: the indent, the icon, the name, and `Δ` for a file that changed.
fn tree_line(app: &App, t: &Theme, row: &Row, is_current: bool, w: usize) -> Line<'static> {
    let indent = "  ".repeat(row.depth);
    let (icon, name_style) = match row.kind {
        // A main material of the project: first class, in the plain text color and bold.
        Kind::Dir { open } if row.section => (if open { "[-] " } else { "[+] " }, Style::new().fg(t.fg).add_modifier(Modifier::BOLD)),
        Kind::File if row.section && !is_current => (" ◆  ", Style::new().fg(t.fg).add_modifier(Modifier::BOLD)),
        Kind::Dir { open } => (if open { "[-] " } else { "[+] " }, Style::new().fg(t.dim)),
        Kind::File if is_current => (" ●  ", Style::new().fg(t.fg).add_modifier(Modifier::BOLD)),
        Kind::File => ("    ", Style::new().fg(t.fg)),
    };
    // Markdown is the default: its extension shows only in the text view, where files of every kind are.
    let all = app.tree.view == View::Text;
    let name = match row.kind {
        _ if row.section => row.name.clone(),
        Kind::Dir { .. } => format!("{}/", row.name),
        Kind::File if all => row.name.clone(),
        Kind::File => row.name.trim_end_matches(".md").to_string(),
    };
    let changed = row.kind == Kind::File && app.tree.changed.contains(&row.rel);
    let mark = if changed { " Δ" } else { "" };
    let room = w.saturating_sub(indent.width() + 5 + mark.width());
    let name = if name.width() > room { name.chars().take(room.saturating_sub(1)).collect::<String>() + "…" } else { name };
    let pad = w.saturating_sub(1 + indent.width() + 4 + name.width() + mark.width());
    // The extension in the color of its kind of file.
    let cut = if row.kind == Kind::File && !row.section { name.rfind('.').filter(|i| *i > 0) } else { None };
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
