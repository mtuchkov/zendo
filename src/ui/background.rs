//! The background line above the chat input: the background processes of the agent session, with a
//! `×` to stop each (docs/zendo/background.md: BG-1, BG-3, BG-8).
use super::*;

/// For each process: the column of its name, of its `×`, after its `×`, and its process group.
type Spots = Vec<(usize, usize, usize, u32)>;

/// The text of the line and the places of its processes.
pub(crate) fn line(app: &App, w: usize) -> Option<(String, Spots)> {
    let key = app.key();
    let list: Vec<&crate::app::Bg> = app.bg.of(&key).collect();
    if list.is_empty() {
        return None;
    }
    let mut s = format!("⏵ {} running", list.len());
    let mut spots = Vec::new();
    for b in list {
        let time = crate::app::ran(b.started.elapsed());
        let room = w.saturating_sub(s.chars().count() + time.len() + 6);
        if room < 4 {
            break;
        }
        let name: String = if b.label.chars().count() > room { b.label.chars().take(room - 1).chain(['…']).collect() } else { b.label.clone() };
        s.push_str(" · ");
        let at = s.chars().count();
        s.push_str(&format!("{name} {time} "));
        let x = s.chars().count();
        s.push('×');
        spots.push((at, x, x + 1, b.pgid));
    }
    Some((s, spots))
}

/// Draw the line, and keep the places of its names and of its `×` for the mouse.
pub(super) fn draw(f: &mut Frame, app: &App, fr: &mut Cache, t: &Theme, area: Rect) {
    if area.height == 0 {
        return;
    }
    let Some((text, spots)) = line(app, area.width as usize) else { return };
    f.render_widget(Paragraph::new(Line::styled(text, Style::new().fg(t.dim))), area);
    for (name, x, end, pgid) in spots {
        let at = |a: usize, b: usize| Rect { x: area.x + a as u16, width: (b - a) as u16, height: 1, ..area };
        fr.bg_buttons.push((at(name, x), pgid, false));
        fr.bg_buttons.push((at(x, end), pgid, true));
    }
}
