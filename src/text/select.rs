//! Mouse selection that stays inside one panel, and copy to the clipboard.

use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::Color;
use unicode_width::UnicodeWidthStr;

use super::boxtable;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Region {
    Note,
    Delta,
    Panel,
}

#[derive(Clone, Copy, Debug)]
pub struct Selection {
    pub region: Region,
    pub rect: Rect,
    pub anchor: (u16, u16),
    pub head: (u16, u16),
    /// The mouse button is up; the text is final.
    pub done: bool,
}

impl Selection {
    pub fn start(region: Region, rect: Rect, x: u16, y: u16) -> Self {
        let p = clamp(rect, x, y);
        Self { region, rect, anchor: p, head: p, done: false }
    }

    pub fn drag(&mut self, x: u16, y: u16) {
        let (cx, cy) = clamp(self.rect, x, y);
        // Left of the panel on a lower row: the selection ends with the row above.
        self.head = if x < self.rect.x && cy > self.anchor.1 { (self.rect.right().saturating_sub(1), cy - 1) } else { (cx, cy) };
    }

    pub fn is_empty(&self) -> bool {
        self.anchor == self.head
    }

    /// Start and end in reading order.
    fn ordered(&self) -> ((u16, u16), (u16, u16)) {
        let (a, b) = (self.anchor, self.head);
        if (a.1, a.0) <= (b.1, b.0) { (a, b) } else { (b, a) }
    }

    /// The selected columns of row `y`, end exclusive.
    fn cols(&self, y: u16) -> Option<(u16, u16)> {
        let (s, e) = self.ordered();
        if y < s.1 || y > e.1 {
            return None;
        }
        let from = if y == s.1 { s.0 } else { self.rect.x };
        let to = if y == e.1 { e.0 + 1 } else { self.rect.right() };
        Some((from, to.min(self.rect.right())))
    }

    /// The rows of the panel as the screen shows them, and the selection in them, when it starts in
    /// a cell of a table with box borders (PANES-34).
    fn in_table(&self, buf: &Buffer) -> Option<(Vec<String>, boxtable::Range)> {
        let rows: Vec<String> = (self.rect.y..self.rect.bottom()).map(|y| screen_row(buf, self.rect, y)).collect();
        let at = |(x, y): (u16, u16)| ((y - self.rect.y) as usize, (x - self.rect.x) as usize);
        let range = boxtable::resolve(&rows, at(self.anchor), at(self.head));
        range.cols.is_some().then_some((rows, range))
    }

    pub fn paint(&self, buf: &mut Buffer, bg: Color, fg: Color) {
        if let Some((rows, range)) = self.in_table(buf) {
            for (i, (a, b)) in (0..rows.len()).filter_map(|i| range.span(i, 0).map(|s| (i as u16, s))) {
                for x in a as u16..b as u16 {
                    if let Some(c) = buf.cell_mut(Position::new(self.rect.x + x, self.rect.y + i)) {
                        c.set_bg(bg).set_fg(fg);
                    }
                }
            }
            return;
        }
        let (s, e) = self.ordered();
        for y in s.1..=e.1 {
            let Some((from, to)) = self.cols(y) else { continue };
            for x in from..to {
                if let Some(c) = buf.cell_mut(Position::new(x, y)) {
                    c.set_bg(bg).set_fg(fg);
                }
            }
        }
    }

    /// The text under the selection, as the screen shows it.
    pub fn text(&self, buf: &Buffer) -> String {
        if let Some((rows, range)) = self.in_table(buf) {
            return range.text(&rows);
        }
        let (s, e) = self.ordered();
        let mut rows = Vec::new();
        for y in s.1..=e.1 {
            let Some((from, to)) = self.cols(y) else { continue };
            let mut row = String::new();
            let mut x = from;
            while x < to {
                let sym = buf.cell(Position::new(x, y)).map(|c| c.symbol()).unwrap_or(" ");
                row.push_str(sym);
                // A wide character covers the next cell too.
                x += sym.width().max(1) as u16;
            }
            rows.push(row.trim_end().to_string());
        }
        // Drop the common left margin, so indented panels copy cleanly.
        let margin = rows.iter().skip(1).filter(|r| !r.is_empty()).map(|r| r.len() - r.trim_start().len()).min().unwrap_or(0);
        let mut out = String::new();
        for (i, r) in rows.iter().enumerate() {
            if i > 0 {
                out.push('\n');
                out.push_str(r.get(margin.min(r.len() - r.trim_start().len())..).unwrap_or(r));
            } else {
                out.push_str(r);
            }
        }
        out.trim_end().to_string()
    }
}

/// Row `y` of `rect`, one char for each cell: a wide char and the cell that it covers are one char.
fn screen_row(buf: &Buffer, rect: Rect, y: u16) -> String {
    let mut row = String::new();
    let mut x = rect.x;
    while x < rect.right() {
        let sym = buf.cell(Position::new(x, y)).map(|c| c.symbol()).filter(|s| !s.is_empty()).unwrap_or(" ");
        row.push_str(sym);
        x += sym.width().max(1) as u16;
    }
    row
}

fn clamp(r: Rect, x: u16, y: u16) -> (u16, u16) {
    (x.clamp(r.x, r.right().saturating_sub(1)), y.clamp(r.y, r.bottom().saturating_sub(1)))
}

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn base64(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
        let n = (b[0] as u32) << 16 | (b[1] as u32) << 8 | b[2] as u32;
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(B64[(n >> (18 - 6 * i) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

/// Copy with OSC 52 (most terminals, also over SSH) and with the system tool if there is one.
pub fn copy(text: &str) {
    use std::io::Write;
    let mut out = std::io::stdout();
    let _ = write!(out, "\x1b]52;c;{}\x07", base64(text.as_bytes()));
    let _ = out.flush();
    let text = text.to_string();
    std::thread::spawn(move || {
        let tools: [(&str, &[&str]); 4] = [
            ("pbcopy", &[]),
            ("wl-copy", &[]),
            ("xclip", &["-selection", "clipboard"]),
            ("xsel", &["--clipboard", "--input"]),
        ];
        for (cmd, args) in tools {
            let child = std::process::Command::new(cmd)
                .args(args)
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn();
            if let Ok(mut child) = child {
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(text.as_bytes());
                }
                if child.wait().is_ok_and(|s| s.success()) {
                    return;
                }
            }
        }
    });
}

/// Read the system clipboard with the system tool. None over SSH or without a tool.
pub fn paste() -> Option<String> {
    let tools: [(&str, &[&str]); 4] = [("pbpaste", &[]), ("wl-paste", &["--no-newline"]), ("xclip", &["-o", "-selection", "clipboard"]), ("xsel", &["--clipboard", "--output"])];
    for (cmd, args) in tools {
        let out = std::process::Command::new(cmd).args(args).stdin(std::process::Stdio::null()).stderr(std::process::Stdio::null()).output();
        if let Ok(o) = out
            && o.status.success()
        {
            return Some(String::from_utf8_lossy(&o.stdout).into_owned());
        }
    }
    None
}

/// The files of a paste, when the paste is nothing but absolute paths: what a terminal pastes when
/// you drop files on it (`/a/b.png`, `/a/My\ File.md`, `'/a/b c.md'`, `file:///a/b.png`).
pub fn dropped_files(s: &str) -> Option<Vec<std::path::PathBuf>> {
    use std::path::PathBuf;
    let t = s.trim();
    if t.is_empty() {
        return None;
    }
    let home = std::env::var("HOME").unwrap_or_default();
    let clean = |w: &str| -> PathBuf {
        let w = w.strip_prefix("file://").map(|r| r.replace("%20", " ")).unwrap_or_else(|| w.to_string());
        match w.strip_prefix("~/") {
            Some(r) => PathBuf::from(&home).join(r),
            None => PathBuf::from(w),
        }
    };
    let ok = |p: &PathBuf| p.is_absolute() && p.is_file();
    // One path, spaces and all.
    let one = clean(t.trim_matches(|c| c == '\'' || c == '"'));
    if ok(&one) {
        return Some(vec![one]);
    }
    // Words as a shell splits them: quotes and backslashes.
    let mut words = Vec::new();
    let mut cur = String::new();
    let (mut quote, mut esc, mut any) = (None::<char>, false, false);
    for c in t.chars() {
        if esc {
            cur.push(c);
            esc = false;
        } else if c == '\\' && quote != Some('\'') {
            esc = true;
        } else if let Some(q) = quote {
            if c == q { quote = None } else { cur.push(c) }
        } else if c == '\'' || c == '"' {
            quote = Some(c);
            any = true;
        } else if c.is_whitespace() {
            if !cur.is_empty() || any {
                words.push(std::mem::take(&mut cur));
            }
            any = false;
        } else {
            cur.push(c);
        }
    }
    if !cur.is_empty() || any {
        words.push(cur);
    }
    let paths: Vec<PathBuf> = words.iter().map(|w| clean(w)).collect();
    (!paths.is_empty() && paths.iter().all(ok)).then_some(paths)
}

/// An image on the system clipboard, saved as a PNG file. None when there is none, or no tool.
pub fn paste_image() -> Option<std::path::PathBuf> {
    use std::process::{Command, Stdio};
    let dir = crate::setup::config::data_dir().join("pastes");
    std::fs::create_dir_all(&dir).ok()?;
    let file = dir.join(format!("paste-{}.png", chrono::Local::now().format("%Y%m%d-%H%M%S")));
    let run = |cmd: &str, args: &[&str]| Command::new(cmd).args(args).stdin(Stdio::null()).stderr(Stdio::null()).output().ok().filter(|o| o.status.success()).map(|o| o.stdout);
    let saved = |bytes: Vec<u8>| (!bytes.is_empty() && std::fs::write(&file, bytes).is_ok()).then(|| file.clone());
    if cfg!(target_os = "macos") {
        let f = file.display().to_string();
        let script = format!("set png to (the clipboard as «class PNGf»)\nset fh to open for access POSIX file \"{f}\" with write permission\nwrite png to fh\nclose access fh");
        if run("osascript", &["-e", &script]).is_some() && std::fs::metadata(&file).is_ok_and(|m| m.len() > 0) {
            return Some(file);
        }
        return None;
    }
    if run("wl-paste", &["--list-types"]).is_some_and(|t| String::from_utf8_lossy(&t).lines().any(|l| l == "image/png")) {
        return run("wl-paste", &["--type", "image/png"]).and_then(saved);
    }
    if run("xclip", &["-selection", "clipboard", "-t", "TARGETS", "-o"]).is_some_and(|t| String::from_utf8_lossy(&t).lines().any(|l| l == "image/png")) {
        return run("xclip", &["-selection", "clipboard", "-t", "image/png", "-o"]).and_then(saved);
    }
    None
}

#[cfg(test)]
#[path = "select_tests.rs"]
mod tests;
