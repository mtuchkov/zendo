//! The layout: which panes fit, their widths, the top bar.
use super::*;

/// The width of a pane: the one you dragged, else a default for the layout.
pub fn pane_width(app: &App, p: Focus, total: u16) -> u16 {
    let key = App::pane_key(p).unwrap_or("note");
    if let Some(w) = app.state.widths.get(key) {
        return *w;
    }
    match p {
        Focus::Tree => (total / 4).clamp(28, 44),
        Focus::Claude if !app.state.wide() => (total * 2 / 5).clamp(40, 60),
        Focus::Claude => (total / 4).clamp(40, 60),
        Focus::Delta => (total / 3).clamp(56, 100),
        Focus::Note => 50,
    }
}

/// Which panes to show, left to right.
/// Standard: one main pane (delta replaces the note while reading) and the chat.
/// Wide screen: every open pane side by side.
pub fn panes(app: &App, width: u16) -> Vec<Focus> {
    let chat = app.threads.shown();
    let delta = app.work.delta_open() && app.files.current().is_some() && app.threads.pinned().is_none();
    let mut p = Vec::new();
    if app.state.wide() {
        // Wide: note | chat | delta.
        p.push(Focus::Note);
        if chat {
            p.push(Focus::Claude);
        }
        if delta {
            p.push(Focus::Delta);
        }
    } else if app.mode == Mode::Write {
        p.push(Focus::Note);
        if chat {
            p.push(Focus::Claude);
        } else if delta {
            p.push(Focus::Delta);
        }
    } else {
        p.push(if delta { Focus::Delta } else { Focus::Note });
        if chat {
            p.push(Focus::Claude);
        }
    }
    // The file tree sits on the left in both layouts.
    if app.state.tree {
        p.insert(0, Focus::Tree);
    }
    let need = |f: &Focus| if *f == Focus::Note { 50 } else { pane_width(app, *f, width) };
    hide_to_fit(p, app.focus, |ps| ps.iter().map(need).sum::<u16>() <= width)
}

/// Too narrow for all panes: hide from the edges, by turns, starting on the right:
/// the rightmost pane right of the main pane, then the leftmost pane left of it, then the
/// rightmost again. The pane in focus stays. The main pane goes last.
pub fn hide_to_fit(mut p: Vec<Focus>, focus: Focus, fits: impl Fn(&[Focus]) -> bool) -> Vec<Focus> {
    let mut right = true;
    while p.len() > 1 && !fits(&p) {
        // The main pane: the note, or the pane in its place (delta in standard).
        let main = p.iter().position(|f| *f == Focus::Note).or_else(|| p.iter().position(|f| *f != Focus::Tree)).unwrap_or(0);
        let on_right = (main + 1..p.len()).rev().find(|i| p[*i] != focus);
        let on_left = (0..main).find(|i| p[*i] != focus);
        let pick = if right { on_right.or(on_left) } else { on_left.or(on_right) };
        let i = match pick {
            Some(i) => i,
            // Only the main pane and the pane in focus are left.
            None => match p.iter().position(|f| *f != focus) {
                Some(i) => i,
                None => break,
            },
        };
        p.remove(i);
        right = !right;
    }
    p
}

pub(crate) fn draw_left(f: &mut Frame, app: &mut App, fr: &mut Cache, t: &Theme, area: Rect) {
    if app.mode == Mode::Write {
        draw_write(f, app, fr, t, area);
    } else {
        draw_read(f, app, fr, t, area);
    }
}

pub(crate) fn column(area: Rect, width: u16) -> Rect {
    let w = width.min(area.width.saturating_sub(4)).max(10);
    Rect { x: area.x + (area.width.saturating_sub(w)) / 2, width: w, ..area }
}

pub(crate) fn draw_top(f: &mut Frame, app: &mut App, fr: &mut Cache, t: &Theme, area: Rect) {
    let left = match app.mode {
        Mode::Write => match app.writer.path() {
            Some(p) => app.rel(p),
            None => {
                let title = crate::docs::notes::title_of(&app.writer.editor().text()).unwrap_or_default();
                let p = app.new_note_path(if title.is_empty() { "note" } else { &title });
                format!("new note → {}", app.rel(&p))
            }
        },
        _ => match app.threads.pinned() {
            Some(ALL_KEY) => format!("{} · all notes", app.rel(&app.cfg.notes_dir)),
            // FILES-45, FILES-47: a held key names the version that it shows.
            _ if app.files.peek().on.is_some() => {
                let doc = app.files.current().and_then(|p| app.doc_rel(p)).unwrap_or_default();
                format!("{} · {doc}", app.files.peek().on.map_or("", |o| o.name()))
            }
            // FILES-34: a file of a worktree names its worktree.
            _ => app.note().map(|n| crate::docs::tree::worktree::label(&app.rel(&n.path))).unwrap_or_else(|| app.rel(&app.cfg.notes_dir)),
        },
    };
    f.render_widget(Paragraph::new(Line::from(vec![Span::raw("  "), Span::raw(left.clone())])).style(Style::new().fg(t.dim)), area);
    let right = draw_top_right(f, app, fr, t, area);

    // CMD-11: a newer zendo shows as a message and the upgrade button, left of the other buttons.
    if app.update.latest().is_some() && matches!(app.mode, Mode::Read | Mode::Write) {
        let w = (UPGRADE_HINT.width() + 1 + UPGRADE.width()) as u16;
        let x = area.right().saturating_sub(right + 5 + w);
        if x > area.x + left.width() as u16 + 4 {
            let line = Line::from(vec![Span::styled(format!("{UPGRADE_HINT} "), Style::new().fg(t.acc)), Span::styled(UPGRADE, switch(t, true))]);
            f.render_widget(Paragraph::new(line), Rect { x, width: w, ..area });
            let b = UPGRADE.width() as u16;
            fr.upgrade_button = Some(Rect { x: x + w - b, width: b, ..area });
        }
    }
}

/// The message of a newer zendo in the top line (CMD-11).
const UPGRADE_HINT: &str = "new version available";
/// The upgrade button in the top line (CMD-11).
pub const UPGRADE: &str = "[upgrade]";

/// The sound button in the top line (ATTN-7).
pub const SOUND: &str = "[sound]";
/// The music button in the top line (LOFI-10).
pub const MUSIC: &str = "[music]";

/// A button of the top line: blue while on, gray while off. Not struck through: Apple Terminal does
/// not show it (ATTN-7, LOFI-10).
fn switch(t: &Theme, on: bool) -> Style {
    Style::new().fg(if on { t.ext.doc } else { t.dim })
}

/// The right of the top line: the branch of the context (FILES-59 to FILES-61), the sessions that
/// blink while their buttons are not on the screen (ATTN-10), the music button (LOFI-10), and the
/// sound button (ATTN-7). Returns its width.
fn draw_top_right(f: &mut Frame, app: &mut App, fr: &mut Cache, t: &Theme, area: Rect) -> u16 {
    let chips: Vec<usize> = if fr.session_buttons.is_empty() { app.attention.blinks().collect() } else { Vec::new() };
    let chip = |n: usize| Style::new().fg(t.bg).bg(pulse_bg(app, t, n, if n >= crate::app::SESSIONS { t.fg } else { t.acc2 }));
    let branch = app.top_branch().map(|b| (b, Style::new().fg(t.dim)));
    let first = usize::from(branch.is_some());
    let mut pieces: Vec<(String, Style)> = branch.into_iter().collect();
    pieces.extend(chips.iter().map(|n| (format!(" {} ", titled(app, *n)), chip(*n))));
    pieces.push((MUSIC.into(), switch(t, app.music.state() == crate::music::State::Plays)));
    pieces.push((SOUND.into(), switch(t, app.attention.sound)));
    let width = (pieces.iter().map(|(s, _)| s.width() + 2).sum::<usize>() - 2) as u16;
    let mut x = area.right().saturating_sub(width + 2);
    for (i, (text, style)) in pieces.into_iter().enumerate() {
        let r = Rect { x, width: (text.width() as u16).min(area.right().saturating_sub(x)), ..area };
        x += r.width + 2;
        f.render_widget(Paragraph::new(Span::styled(text, style)), r);
        match i.checked_sub(first).map(|k| (k, chips.get(k))) {
            Some((_, Some(n))) => fr.session_buttons.push((r, *n)),
            Some((k, None)) if k == chips.len() => fr.music_button = Some(r),
            Some((k, None)) if k == chips.len() + 1 => fr.sound_button = Some(r),
            _ => {}
        }
    }
    width
}

pub(crate) fn date_label(n: &Note) -> String {
    let d = n.created.unwrap_or_else(|| n.modified.date_naive());
    let time = if n.created.is_none_or(|c| c == n.modified.date_naive()) { n.modified.format(" · %H:%M").to_string() } else { String::new() };
    format!("{}{}", d.format("%a %-d %b"), time)
}
