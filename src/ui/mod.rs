//! Drawing. One calm column of text, a status bar, and a side panel for Claude.

pub mod theme;
mod chat;
pub(crate) mod cli;
mod code_pane;
mod delta_pane;
mod field;
mod find;
mod float;
pub mod frame;
mod layout;
mod lists;
mod overlays;
mod paint;
mod read;
mod review;
mod rekey;
mod status;
mod tips;
mod thread;
mod tree;
mod write;

use chat::*;
use cli::*;
use code_pane::*;
use delta_pane::*;
use find::*;
use float::*;
use frame::Frame as Cache;
pub use layout::*;
use lists::*;
use overlays::*;
pub use paint::*;
pub use read::*;
use review::*;
use status::*;
pub use thread::*;
use tree::*;
use write::*;

use crate::app::{ALL_KEY, App, Focus, Item, Mode};
use crate::docs::delta;
use crate::text::select::Region;
use crate::text::code;
use crate::text::markdown::{self, Mark};
use crate::docs::notes::Note;
use crate::ui::theme::{THEMES, Theme};
use chrono::{Datelike, Local};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph};
use unicode_width::UnicodeWidthStr;

/// The width of the text column when the config sets none.
pub const COLUMN: u16 = 76;
const SPIN: [&str; 4] = ["·  ", "·· ", "···", " ··"];

/// Draw the screen. The result is the frame for the mouse and the keys, built from `last`: what a
/// draw does not show keeps its last value.
pub fn draw(f: &mut Frame, app: &mut App, last: Cache) -> Cache {
    let mut fr = last;
    draw_into(f, app, &mut fr);
    fr
}

fn draw_into(f: &mut Frame, app: &mut App, fr: &mut Cache) {
    if app.writer.float().is_some() {
        return draw_float(f, app, fr);
    }
    app.drop_old_review();
    let t = *app.theme();
    let area = f.area();
    app.cli.resize(area.height.saturating_sub(1), area.width);
    fr.clear_buttons();
    app.rekey.follow(app.focus, app.mode);
    if app.hold.is_some() {
        fr.regions.clear();
        draw_hold(f, app, &t, area);
        return draw_selection(f, app, &t);
    }
    if let Some(n) = app.cli.view {
        fr.regions.clear();
        draw_cli(f, app, fr, &t, n, area);
        return draw_selection(f, app, &t);
    }
    f.render_widget(Block::new().style(Style::new().bg(t.bg).fg(t.fg)), area);
    // The question of a file, else on a small screen the message or the tip (PANES-49).
    let below = if app.files.prompt().is_some() { None } else { below_line(app, fr, &t, area.width) };
    let line = u16::from(app.files.prompt().is_some() || below.is_some());
    let [top, _, main, rule, status, prompt] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(line),
    ])
    .areas(area);

    fr.regions.clear();
    fr.session_buttons.clear();
    fr.play_buttons.clear();
    fr.new_terminal_button = None;
    fr.tree_fold.clear();
    f.render_widget(Paragraph::new("─".repeat(rule.width as usize)).style(Style::new().fg(t.line)), rule);
    fr.view_height = main.height as usize;

    match app.mode {
        Mode::Notes => draw_notes(f, app, &t, main),
        Mode::Search => draw_search(f, app, &t, main),
        Mode::Find => fr.overlay = Some(draw_find(f, app, &t, main)),
        Mode::Theme => {
            let w = 40.min(main.width / 2);
            let [left, right] = Layout::horizontal([Constraint::Min(1), Constraint::Length(w)]).areas(main);
            draw_read(f, app, fr, &t, left);
            draw_themes(f, app, fr, &t, right);
        }
        Mode::Read | Mode::Write | Mode::Help => {
            draw_panes(f, app, fr, &t, main);
            if app.mode == Mode::Help {
                // Over the panes and the top line, not over the status bar.
                fr.overlay = Some(draw_help(f, app, fr, &t, Rect { height: rule.y - area.y, ..area }));
            }
        }
    }
    // After the panes: the top line shows the blinking sessions whose buttons are not on the screen.
    draw_top(f, app, fr, &t, top);
    draw_bottom(f, app, fr, &t, [status, prompt], below);
    draw_confirm(f, app, &t, area);
    draw_questions(f, app, fr, &t, area);
    tips::draw_tips(f, app, fr, &t, area);
    rekey::draw_key_float(f, app, fr, &t, area);

    draw_selection(f, app, &t);
}

/// The mouse selection on top of the frame; after the release, its text goes to the clipboard.
fn draw_selection(f: &mut Frame, app: &mut App, t: &Theme) {
    let Some(sel) = app.pointer.sel() else { return };
    if app.pointer.take_copy_request() {
        let text = sel.text(f.buffer_mut());
        if sel.chat {
            // After the frame, the text goes to the chat input as a chip (PANES-44).
            app.pointer.send_to_chat(text);
            return;
        } else if !text.is_empty() {
            app.pointer.copy(text);
        }
    }
    sel.paint(f.buffer_mut(), t.acc2, t.bg);
}

/// The panes side by side: the note fills what the others leave; their edges can be dragged.
fn draw_panes(f: &mut Frame, app: &mut App, fr: &mut Cache, t: &Theme, main: Rect) {
    let panes = panes(app, main.width);
    fr.visible = panes.clone();
    app.keep_focus_in(&panes);
    let cons: Vec<Constraint> = panes
        .iter()
        .map(|p| match p {
            Focus::Note => Constraint::Fill(1),
            other => Constraint::Length(pane_width(app, *other, main.width)),
        })
        .collect();
    let areas = Layout::horizontal(cons).split(main);
    fr.pane_rects = panes.iter().copied().zip(areas.iter().copied()).collect();
    for (i, (p, a)) in panes.iter().zip(areas.iter()).enumerate() {
        match p {
            Focus::Tree => draw_tree(f, app, fr, t, *a),
            Focus::Note => draw_left(f, app, fr, t, *a),
            Focus::Delta => draw_delta(f, app, fr, t, *a),
            Focus::Claude => draw_panel(f, app, fr, t, *a),
        }
        // The edge between panes: drag it to resize.
        if i > 0 {
            let dragging = app.pointer.grabbed().is_some_and(|(d, _, _)| d == *p || (d == panes[i - 1] && *p == Focus::Note));
            let style = Style::new().fg(if dragging { t.acc2 } else { t.line }).bg(t.bg);
            let rule: Vec<Line> = (0..a.height).map(|_| Line::styled("│", style)).collect();
            f.render_widget(Paragraph::new(rule), Rect { width: 1, ..*a });
        }
    }
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
