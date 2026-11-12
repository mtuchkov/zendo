//! A small markdown renderer for notes. It knows the few things notes use:
//! headings, bullets, code fences, quotes, `code`, **bold**, [[links]],
//! #tags and EARS keywords. It wraps text itself, so callers know the height.

mod blocks;
mod code;
mod inline;
mod table;
mod wrap;

use blocks::*;
use code::*;
pub use inline::*;
use table::*;
pub use wrap::*;

use crate::ui::theme::Theme;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use crate::text::highlight::Kind;
use std::collections::HashMap;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Mark {
    None,
    Added,
    Removed,
}

#[derive(Clone)]
pub struct Rendered {
    pub lines: Vec<Line<'static>>,
    /// For each source line, the first rendered row.
    pub rows: Vec<usize>,
    /// For each rendered row, its words of prose: front matter, code and borders have none.
    pub words: Vec<usize>,
}

/// The words of a row: pieces with a letter or a digit.
pub fn count_words(text: &str) -> usize {
    text.split_whitespace().filter(|w| w.chars().any(char::is_alphanumeric)).count()
}

const EARS: [&str; 6] = ["WHEN", "WHILE", "IF", "THEN", "WHERE", "SHALL"];

/// Render with a cache: each frame draws the same text again, and a fast scroll draws many frames.
pub fn render(src: &[(String, Mark)], width: usize, t: &Theme, skip_title: bool) -> Rendered {
    cached((src, width, t.name, t.panel, skip_title), || render_text(src, width, t, skip_title, None))
}

/// Write mode: the same look as read mode, and the line `raw` (the cursor line) as its source text,
/// wrapped as the editor wraps it. The title is not in it: the head of the note shows it.
pub fn render_live(src: &[(String, Mark)], width: usize, t: &Theme, raw: Option<usize>) -> Rendered {
    cached((src, width, t.name, t.panel, raw, "live"), || render_text(src, width, t, true, raw))
}

/// The render of a text from the cache, or a new one.
fn cached(key: impl std::hash::Hash, make: impl FnOnce() -> Rendered) -> Rendered {
    use std::hash::Hasher;
    thread_local! {
        static CACHE: std::cell::RefCell<std::collections::HashMap<u64, Rendered>> = Default::default();
    }
    let mut h = std::collections::hash_map::DefaultHasher::new();
    key.hash(&mut h);
    let key = h.finish();
    if let Some(r) = CACHE.with(|c| c.borrow().get(&key).cloned()) {
        return r;
    }
    let r = make();
    CACHE.with(|c| {
        let mut c = c.borrow_mut();
        if c.len() >= 512 {
            c.clear();
        }
        c.insert(key, r.clone());
    });
    r
}

fn render_text(src: &[(String, Mark)], width: usize, t: &Theme, skip_title: bool, raw_line: Option<usize>) -> Rendered {
    let logicals = join(src, raw_line);
    // The cursor line in write mode, as an index of the logical lines.
    let raw_line = raw_line.and_then(|r| {
        let mut seen = 0;
        logicals.iter().position(|l| {
            seen += l.n;
            seen > r
        })
    });
    let mut r = Renderer { width: width.max(10), t, skip_title, out: Vec::new(), fence: false, title_skipped: !skip_title, skip_blank: false, code: code_pieces(&logicals) };
    let mut rows = Vec::with_capacity(src.len());
    let mut skip_until = 0;
    // The rows of the logical line before: prose or not.
    let mut prose: Vec<bool> = Vec::new();
    let mut open: Option<bool> = None;
    for (i, logical) in logicals.iter().enumerate() {
        if let Some(p) = open.take() {
            prose.resize(r.out.len(), p);
        }
        if i < skip_until {
            continue;
        }
        let raw = &logical.text;
        open = Some(!logical.verbatim && !r.fence && !raw.trim_start().starts_with("```") && !is_rule(raw.trim()));
        if let Some((end, table)) = table_at(&logicals, i, r.width, t, raw_line) {
            for (k, start) in table.rows.into_iter().enumerate() {
                for _ in 0..logicals[i + k].n {
                    rows.push(r.out.len() + start);
                }
            }
            r.out.extend(table.lines);
            skip_until = end;
            continue;
        }
        for _ in 0..logical.n {
            rows.push(r.out.len());
        }
        if raw_line == Some(i) {
            if logical.text.trim_start().starts_with("```") {
                r.fence = !r.fence;
            }
            r.out.extend(raw_rows(&logical.text, r.width, t));
            continue;
        }
        r.line(i, logical);
    }
    if let Some(p) = open.take() {
        prose.resize(r.out.len(), p);
    }
    let out = r.out;
    prose.resize(out.len(), false);
    let words = out.iter().zip(&prose).map(|(l, p)| if *p { count_words(&l.spans.iter().map(|s| s.content.as_ref()).collect::<String>()) } else { 0 }).collect();
    Rendered { lines: out, rows, words }
}

/// A source line as it is, wrapped as the editor wraps it, so the cursor stands on its char.
pub fn raw_rows(text: &str, width: usize, t: &Theme) -> Vec<Line<'static>> {
    crate::text::editor::wrap_line(text, width)
        .into_iter()
        .map(|(a, b)| Line::styled(text.chars().skip(a).take(b - a).collect::<String>(), Style::new().fg(t.fg)))
        .collect()
}

/// The state of one render: the lines so far, and whether a code fence is open.
struct Renderer<'a> {
    width: usize,
    t: &'a Theme,
    skip_title: bool,
    out: Vec<Line<'static>>,
    fence: bool,
    title_skipped: bool,
    skip_blank: bool,
    code: HashMap<usize, Vec<(crate::text::highlight::Kind, String)>>,
}

impl Renderer<'_> {
    /// The color of a diff mark on top of a style.
    fn gutter(&self, s: Style, mark: Mark) -> Style {
        match mark {
            Mark::None => s,
            Mark::Added => s.fg(self.t.add),
            Mark::Removed => s.fg(self.t.del).add_modifier(Modifier::CROSSED_OUT),
        }
    }

    fn line(&mut self, i: usize, logical: &Logical) {
        let (t, width) = (self.t, self.width);
        let (raw, mark) = (&logical.text, logical.mark);
        if logical.verbatim {
            let spans = match self.code.get(&i) {
                Some(p) => p.iter().map(|(k, s)| Span::styled(s.clone(), k.style(Style::new().fg(t.dim), t))).collect(),
                None => vec![Span::styled(raw.clone(), Style::new().fg(t.dim))],
            };
            self.out.extend(wrap(spans, width, "", "  "));
            return;
        }
        let trimmed = raw.trim_start();
        if trimmed.starts_with("```") {
            self.fence = !self.fence;
            let label = trimmed.trim_start_matches('`').trim();
            let label = if !self.fence || label.is_empty() { vec![] } else { vec![Span::styled(format!(" {label}"), Style::new().fg(t.dim))] };
            self.out.push(pad(label, width, Style::new().bg(t.panel)));
            return;
        }
        if self.fence {
            return self.code_line(i, raw, mark);
        }
        if !self.title_skipped && !trimmed.is_empty() {
            self.title_skipped = true;
            if trimmed.starts_with("# ") {
                // The caller draws the title and meta line itself.
                self.skip_blank = true;
                return;
            }
        }
        if trimmed.is_empty() {
            if self.skip_blank {
                self.skip_blank = false;
            } else {
                self.out.push(Line::raw(""));
            }
            return;
        }
        self.skip_blank = false;
        // In read mode the meta line shows the tags, so a line of only tags is hidden.
        if self.skip_title && mark == Mark::None && only_tags(trimmed) {
            self.skip_blank = true;
            return;
        }
        self.prose(raw, mark);
    }

    fn code_line(&mut self, i: usize, raw: &str, mark: Mark) {
        let (t, width) = (self.t, self.width);
        let style = self.gutter(Style::new().fg(t.fg).bg(t.panel), mark);
        let pieces: Vec<(Style, String)> = match self.code.get(&i).filter(|_| mark == Mark::None) {
            Some(p) => std::iter::once((style, "  ".to_string())).chain(p.iter().map(|(k, s)| (k.style(style, t), s.clone()))).collect(),
            None => vec![(style, format!("  {raw}"))],
        };
        for row in hard_wrap_styled(pieces, width) {
            self.out.push(pad(row, width, Style::new().bg(t.panel)));
        }
    }

    /// A heading, a list item, a quote or a paragraph.
    fn prose(&mut self, raw: &str, mark: Mark) {
        let (t, width) = (self.t, self.width);
        let trimmed = raw.trim_start();
        let base = self.gutter(Style::new().fg(t.fg), mark);
        if let Some((level, text)) = heading(trimmed) {
            let color = if level == 1 { t.fg } else { t.acc };
            let style = self.gutter(Style::new().fg(color).add_modifier(Modifier::BOLD), mark);
            self.out.extend(wrap(vec![Span::styled(text.to_string(), style)], width, "", ""));
            return;
        }
        let indent = raw.len() - trimmed.len();
        let pad_in = " ".repeat(indent.min(8));
        if let Some(rest) = bullet(trimmed) {
            let first = format!("{pad_in}• ");
            let hang = " ".repeat(first.width());
            let lines = wrap(inline(rest, base, t), width, &first, &hang);
            self.out.extend(lines.into_iter().enumerate().map(|(i, mut l)| {
                if i == 0
                    && let Some(s) = l.spans.first_mut() {
                        s.style = Style::new().fg(t.dim);
                    }
                l
            }));
            return;
        }
        if let Some(rest) = trimmed.strip_prefix("> ").or_else(|| trimmed.strip_prefix('>')) {
            let spans = inline(rest, self.gutter(Style::new().fg(t.dim), mark), t);
            self.out.extend(wrap(spans, width, "│ ", "│ ").into_iter().map(|mut l| {
                if let Some(s) = l.spans.first_mut() {
                    s.style = Style::new().fg(t.line);
                }
                l
            }));
            return;
        }
        self.out.extend(wrap(inline(trimmed, base, t), width, &pad_in, &pad_in));
    }
}

pub fn plain(src: &str) -> Vec<(String, Mark)> {
    src.lines().map(|l| (l.to_string(), Mark::None)).collect()
}

/// Merge old and new text into one list of marked lines.
pub fn diff_lines(old: &str, new: &str) -> Vec<(String, Mark)> {
    use similar::{ChangeTag, TextDiff};
    let diff = TextDiff::from_lines(old, new);
    diff.iter_all_changes()
        .map(|c| {
            let text = c.value().trim_end_matches('\n').to_string();
            let mark = match c.tag() {
                ChangeTag::Equal => Mark::None,
                ChangeTag::Insert => Mark::Added,
                ChangeTag::Delete => Mark::Removed,
            };
            (text, mark)
        })
        .collect()
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
