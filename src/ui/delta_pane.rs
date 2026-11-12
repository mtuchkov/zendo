//! The delta pane.
use super::*;

pub(crate) fn draw_delta(f: &mut Frame, app: &mut App, t: &Theme, area: Rect) {
    // A thin rule separates the delta from the pane on its left.
    if area.x > 0 {
        let rule: Vec<Line> = (0..area.height).map(|_| Line::styled("│", Style::new().fg(t.line))).collect();
        f.render_widget(Paragraph::new(rule), Rect { width: 1, ..area });
    }
    let width = area.width.saturating_sub(4).min(110);
    let inner = Rect { x: area.x + (area.width - width) / 2, y: area.y, width, height: area.height };
    let w = inner.width as usize;
    let [head, sub, _, body] = Layout::vertical([Constraint::Length(1), Constraint::Length(1), Constraint::Length(1), Constraint::Min(1)]).areas(inner);
    let dim = Style::new().fg(t.dim);
    let focused = app.focus == Focus::Delta;

    let name = app.current.as_ref().and_then(|p| p.file_name()).map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    f.render_widget(Paragraph::new(Line::styled(name, Style::new().fg(t.fg).add_modifier(Modifier::BOLD))), head);
    let ask = format!("{} ask", app.keys.first(crate::keys::Pane::Delta, "ask"));
    f.render_widget(Paragraph::new(Line::styled(ask.clone(), Style::new().fg(if focused { t.acc } else { t.dim }))), Rect { x: head.right().saturating_sub(ask.width() as u16), width: ask.width() as u16, ..head });

    let (label, old) = app.delta_base();
    let new = app.note_text().unwrap_or_default();
    let d = delta::compute(&old, &new, 2);
    let stats = format!(" · +{} −{}", d.added, d.removed);
    let base_btn = format!("{} {}", app.keys.first(crate::keys::Pane::Delta, "base"), if app.delta_base == crate::app::Base::LastVisit { "vs HEAD" } else { "vs last visit" });
    let room = w.saturating_sub(base_btn.width() + 1);
    let mut sub_text = format!("{label}{stats}");
    if sub_text.width() > room {
        sub_text = sub_text.chars().take(room.saturating_sub(1)).collect::<String>() + "…";
    }
    f.render_widget(Paragraph::new(Line::styled(sub_text, dim)), sub);
    f.render_widget(Paragraph::new(Line::styled(base_btn.clone(), dim)), Rect { x: sub.right().saturating_sub(base_btn.width() as u16), width: base_btn.width() as u16, ..sub });

    app.regions.push((Region::Delta, body));
    if d.is_empty() {
        let msg = vec![
            Line::styled(format!("No changes {label}."), Style::new().fg(t.fg)),
            Line::raw(""),
            Line::styled("b compares with the other base.", dim),
        ];
        f.render_widget(Paragraph::new(msg), body);
        return;
    }
    let lines = delta::render(&d, w, t);
    let max = lines.len().saturating_sub(body.height as usize);
    app.delta_scroll = app.delta_scroll.min(max);
    let view: Vec<Line> = lines.into_iter().skip(app.delta_scroll).take(body.height as usize).collect();
    f.render_widget(Paragraph::new(view), body);
}
