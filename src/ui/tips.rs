//! The tip menu over the screen (docs/zendo/tips.md, TIPS-1 to TIPS-5).
use super::*;

/// The tip menu: the tips, the option `Do not show tips at start` and `[ OK ]`; both take a click.
pub(crate) fn draw_tips(f: &mut Frame, app: &App, fr: &mut Cache, t: &Theme, area: Rect) {
    if !app.tips.open {
        return;
    }
    let w = 74.min(area.width);
    let mut lines: Vec<Line> = crate::app::tips::TIPS.iter().flat_map(|tip| markdown::wrap(vec![Span::styled(format!("• {tip}"), Style::new().fg(t.fg))], w.saturating_sub(4) as usize, "  ", "    ")).collect();
    let option = format!("{} Do not show tips at start", if app.tips.off { "[x]" } else { "[ ]" });
    let ok = "[ OK ]";
    lines.extend([Line::raw(""), Line::from(vec![Span::raw("  "), Span::styled(option.clone(), Style::new().fg(t.dim))]), Line::raw("")]);
    lines.push(Line::from(vec![Span::raw("  "), Span::styled(ok, Style::new().fg(t.bg).bg(t.ext.doc))]));
    let rows = lines.len() as u16;
    let h = (rows + 2).min(area.height);
    let r = Rect { x: area.x + (area.width - w) / 2, y: area.y + (area.height - h) / 2, width: w, height: h };
    f.render_widget(Clear, r);
    let block = Block::bordered().title(" tips ").border_style(Style::new().fg(t.ext.doc)).style(Style::new().bg(t.panel));
    f.render_widget(Paragraph::new(lines).block(block), r);
    // On a screen too small for the whole menu, only `enter` closes it.
    let button = |y: u16, s: &str| Rect { x: r.x + 3, y, width: s.width() as u16, height: 1 };
    fr.tip_buttons = (h == rows + 2).then(|| (button(r.bottom() - 4, &option), button(r.bottom() - 2, ok)));
}
