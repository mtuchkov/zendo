//! The background line above the chat input: the background processes of the agent session, with a
//! `×` to stop each, and under `⏷` the list of all of them (docs/zendo/background.md: BG-1, BG-3,
//! BG-8, BG-11, BG-12).
use super::*;
use crate::app::{Bg, ran};

/// For each process: the column of its name, of its `×`, after its `×`, and its process group.
type Spots = Vec<(usize, usize, usize, u32)>;

/// The most rows of the open list (BG-11).
pub(crate) const LIST_ROWS: usize = 8;
/// A name after the first one shows only with room for this many characters.
const MIN_NAME: usize = 8;

/// The rows of the background area: the line, and the list while it is open.
pub(crate) fn height(app: &App) -> u16 {
    match app.bg.of(&app.key()).count() {
        0 => 0,
        n if app.bg.open => 1 + n.min(LIST_ROWS) as u16,
        _ => 1,
    }
}

fn cut(s: &str, room: usize) -> String {
    if s.chars().count() > room { s.chars().take(room.saturating_sub(1)).chain(['…']).collect() } else { s.to_string() }
}

/// Add `name time ×` to `s`, and its places to `spots`.
fn push_item(s: &mut String, spots: &mut Spots, name: &str, time: &str, pgid: u32) {
    let at = s.chars().count();
    s.push_str(&format!("{name} {time} "));
    let x = s.chars().count();
    s.push('×');
    spots.push((at, x, x + 1, pgid));
}

/// The head of the line: the mark, which opens or closes the list, and the number.
fn head(app: &App, n: usize) -> String {
    format!("{} {n} running", if app.bg.open { '⏷' } else { '⏵' })
}

/// The text of the line and the places of its processes; the processes without room show as `+N`
/// at the end (BG-1, BG-12). While the list is open, the line shows only its head.
pub(crate) fn line(app: &App, w: usize) -> Option<(String, Spots)> {
    let key = app.key();
    let list: Vec<&Bg> = app.bg.of(&key).collect();
    if list.is_empty() {
        return None;
    }
    let n = list.len();
    let mut s = head(app, n);
    let mut spots = Vec::new();
    if app.bg.open {
        return Some((s, spots));
    }
    // Each name gets an equal share while all fit; else the first names take the room.
    let share = w.saturating_sub(s.chars().count()) / n;
    for (i, b) in list.iter().enumerate() {
        let time = ran(b.started.elapsed());
        let more = if i + 1 < n { format!(" · +{}", n - i - 1).chars().count() } else { 0 };
        // ` · `, the spaces around the time, and `×`.
        let fixed = 3 + 1 + time.len() + 1 + 1;
        let full = crate::procs::short(&b.label);
        let room = w.saturating_sub(s.chars().count() + fixed + more);
        // The first process shows also in a narrow line, cut short; the next ones only with room.
        let least = if i == 0 { 4 } else { MIN_NAME };
        if room < least.min(full.chars().count()).max(1) {
            s.push_str(&format!(" · +{}", n - i));
            break;
        }
        let cap = if share >= fixed + MIN_NAME { share - fixed } else { room };
        s.push_str(" · ");
        push_item(&mut s, &mut spots, &cut(&full, room.min(cap)), &time, b.pgid);
    }
    Some((s, spots))
}

/// The rows of the open list: for each process its whole command, its time and `×` (BG-11).
pub(crate) fn rows(app: &App, w: usize) -> Vec<(String, Spots)> {
    let key = app.key();
    app.bg
        .of(&key)
        .take(LIST_ROWS)
        .map(|b| {
            let time = ran(b.started.elapsed());
            let name = cut(&b.label, w.saturating_sub(2 + 1 + time.len() + 2).max(1));
            let (mut s, mut spots) = ("  ".to_string(), Vec::new());
            push_item(&mut s, &mut spots, &name, &time, b.pgid);
            (s, spots)
        })
        .collect()
}

/// Draw the line and the open list, and keep the places of the mark, of the names and of the `×`
/// for the mouse.
pub(super) fn draw(f: &mut Frame, app: &App, fr: &mut Cache, t: &Theme, area: Rect) {
    if area.height == 0 {
        return;
    }
    let w = area.width as usize;
    let Some(first) = line(app, w) else { return };
    let n = app.bg.of(&app.key()).count();
    fr.bg_toggle = Some(Rect { width: head(app, n).chars().count() as u16, height: 1, ..area });
    let all = std::iter::once(first).chain(if app.bg.open { rows(app, w) } else { Vec::new() });
    for (y, (text, spots)) in all.enumerate().take(area.height as usize) {
        let row = Rect { y: area.y + y as u16, height: 1, ..area };
        f.render_widget(Paragraph::new(Line::styled(text, Style::new().fg(t.dim))), row);
        for (name, x, end, pgid) in spots {
            let at = |a: usize, b: usize| Rect { x: row.x + a as u16, width: (b - a) as u16, ..row };
            fr.bg_buttons.push((at(name, x), pgid, false));
            fr.bg_buttons.push((at(x, end), pgid, true));
        }
    }
}
