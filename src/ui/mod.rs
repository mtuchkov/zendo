//! Drawing. One calm column of text, a status bar, and a side panel for Claude.

pub mod theme;
mod chat;
pub(crate) mod cli;
mod code_pane;
mod delta_pane;
mod layout;
mod lists;
mod overlays;
mod paint;
mod read;
mod review;
mod status;
mod thread;
mod tree;
mod write;

use chat::*;
use cli::*;
use code_pane::*;
use delta_pane::*;
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

pub fn draw(f: &mut Frame, app: &mut App) {
    app.drop_old_review();
    let t = *app.theme();
    let area = f.area();
    app.cli.resize(area.height.saturating_sub(1), area.width);
    app.attention.button = None;
    if let Some(n) = app.cli.view {
        app.regions.clear();
        draw_cli(f, app, &t, n, area);
        return draw_selection(f, app, &t);
    }
    f.render_widget(Block::new().style(Style::new().bg(t.bg).fg(t.fg)), area);
    let ask = u16::from(app.file_prompt.is_some());
    let [top, _, main, rule, status, prompt] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(ask),
    ])
    .areas(area);

    app.regions.clear();
    app.session_buttons.clear();
    app.play_buttons.clear();
    f.render_widget(Paragraph::new("─".repeat(rule.width as usize)).style(Style::new().fg(t.line)), rule);
    app.view_height = main.height as usize;

    match app.mode {
        Mode::Notes => draw_notes(f, app, &t, main),
        Mode::Search => draw_search(f, app, &t, main),
        Mode::Theme => {
            let w = 40.min(main.width / 2);
            let [left, right] = Layout::horizontal([Constraint::Min(1), Constraint::Length(w)]).areas(main);
            draw_read(f, app, &t, left);
            draw_themes(f, app, &t, right);
        }
        Mode::Read | Mode::Write | Mode::Help => {
            draw_panes(f, app, &t, main);
            if app.mode == Mode::Help {
                draw_help(f, app, &t, area);
            }
        }
    }
    // After the panes: the top line shows the blinking sessions whose buttons are not on the screen.
    draw_top(f, app, &t, top);
    draw_status(f, app, &t, status);
    draw_file_prompt(f, app, &t, prompt);

    draw_selection(f, app, &t);
}

/// The mouse selection on top of the frame; after the release, its text goes to the clipboard.
fn draw_selection(f: &mut Frame, app: &mut App, t: &Theme) {
    let Some(sel) = app.sel else { return };
    if std::mem::take(&mut app.copy_pending) {
        let text = sel.text(f.buffer_mut());
        if !text.is_empty() {
            app.clip = Some(text);
        }
    }
    sel.paint(f.buffer_mut(), t.acc2, t.bg);
}

/// The panes side by side: the note fills what the others leave; their edges can be dragged.
fn draw_panes(f: &mut Frame, app: &mut App, t: &Theme, main: Rect) {
    let panes = panes(app, main.width);
    app.visible = panes.clone();
    if !panes.contains(&app.focus) {
        app.focus = panes[0];
    }
    let cons: Vec<Constraint> = panes
        .iter()
        .map(|p| match p {
            Focus::Note => Constraint::Fill(1),
            other => Constraint::Length(pane_width(app, *other, main.width)),
        })
        .collect();
    let areas = Layout::horizontal(cons).split(main);
    app.pane_rects = panes.iter().copied().zip(areas.iter().copied()).collect();
    for (i, (p, a)) in panes.iter().zip(areas.iter()).enumerate() {
        match p {
            Focus::Tree => draw_tree(f, app, t, *a),
            Focus::Note => draw_left(f, app, t, *a),
            Focus::Delta => draw_delta(f, app, t, *a),
            Focus::Claude => draw_panel(f, app, t, *a),
        }
        // The edge between panes: drag it to resize.
        if i > 0 {
            let dragging = app.drag.is_some_and(|(d, _, _)| d == *p || (d == panes[i - 1] && *p == Focus::Note));
            let style = Style::new().fg(if dragging { t.acc2 } else { t.line }).bg(t.bg);
            let rule: Vec<Line> = (0..a.height).map(|_| Line::styled("│", style)).collect();
            f.render_widget(Paragraph::new(rule), Rect { width: 1, ..*a });
        }
    }
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
