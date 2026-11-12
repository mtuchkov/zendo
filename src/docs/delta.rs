//! Delta: what changed in a note, drawn like `git-delta`: line numbers, full-width
//! red and green rows, and the changed words in a stronger color.

use crate::text::markdown;
use crate::ui::theme::Theme;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use similar::{Algorithm, ChangeTag, TextDiff};
use unicode_width::UnicodeWidthStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tag {
    Same,
    Del,
    Add,
    /// A gap between hunks.
    Gap,
}

#[derive(Clone, Debug)]
pub struct Row {
    pub old: Option<usize>,
    pub new: Option<usize>,
    pub tag: Tag,
    /// Text parts; `true` marks the changed words.
    pub parts: Vec<(bool, String)>,
}

pub struct Delta {
    pub rows: Vec<Row>,
    pub added: usize,
    pub removed: usize,
}

impl Delta {
    pub fn is_empty(&self) -> bool {
        self.added == 0 && self.removed == 0
    }
}

pub fn compute(old: &str, new: &str, context: usize) -> Delta {
    let diff = TextDiff::configure().algorithm(Algorithm::Patience).diff_lines(old, new);
    let mut rows = Vec::new();
    let (mut added, mut removed) = (0, 0);
    for (i, group) in diff.grouped_ops(context).iter().enumerate() {
        if i > 0 {
            rows.push(Row { old: None, new: None, tag: Tag::Gap, parts: vec![] });
        }
        for op in group {
            for c in diff.iter_inline_changes(op) {
                let tag = match c.tag() {
                    ChangeTag::Equal => Tag::Same,
                    ChangeTag::Delete => {
                        removed += 1;
                        Tag::Del
                    }
                    ChangeTag::Insert => {
                        added += 1;
                        Tag::Add
                    }
                };
                let parts = c
                    .values()
                    .iter()
                    .map(|(emph, s)| (*emph, s.trim_end_matches(['\n', '\r']).to_string()))
                    .filter(|(_, s)| !s.is_empty())
                    .collect();
                rows.push(Row { old: c.old_index().map(|i| i + 1), new: c.new_index().map(|i| i + 1), tag, parts });
            }
        }
    }
    Delta { rows, added, removed }
}

fn rgb(c: Color) -> (f32, f32, f32) {
    match c {
        Color::Rgb(r, g, b) => (r as f32, g as f32, b as f32),
        _ => (0.0, 0.0, 0.0),
    }
}

/// Mix two theme colors: `t` = 0 gives `a`, 1 gives `b`.
pub fn mix(a: Color, b: Color, t: f32) -> Color {
    let (a, b) = (rgb(a), rgb(b));
    let m = |x: f32, y: f32| (x + (y - x) * t).round().clamp(0.0, 255.0) as u8;
    Color::Rgb(m(a.0, b.0), m(a.1, b.1), m(a.2, b.2))
}

pub fn render(d: &Delta, width: usize, t: &Theme) -> Vec<Line<'static>> {
    let num_w = d.rows.iter().filter_map(|r| r.new.or(r.old)).max().unwrap_or(1).to_string().len().max(3);
    let gutter = num_w + 3;
    let text_w = width.saturating_sub(gutter).max(10);
    let (del_bg, del_em) = (mix(t.bg, t.del, 0.20), mix(t.bg, t.del, 0.42));
    let (add_bg, add_em) = (mix(t.bg, t.add, 0.18), mix(t.bg, t.add, 0.40));
    let mut out = Vec::new();
    for r in &d.rows {
        if r.tag == Tag::Gap {
            out.push(Line::styled(format!("{:>w$}  ⋯", "", w = num_w), Style::new().fg(t.dim)));
            continue;
        }
        let (sign, bg, em, fg) = match r.tag {
            Tag::Del => ("-", Some(del_bg), del_em, t.fg),
            Tag::Add => ("+", Some(add_bg), add_em, t.fg),
            _ => (" ", None, t.bg, t.fg),
        };
        let base = match bg {
            Some(bg) => Style::new().fg(fg).bg(bg),
            None => Style::new().fg(fg),
        };
        let spans: Vec<Span<'static>> = r
            .parts
            .iter()
            .map(|(emph, s)| {
                let style = if *emph && bg.is_some() { base.bg(em).add_modifier(Modifier::BOLD) } else { base };
                Span::styled(s.clone(), style)
            })
            .collect();
        let no = r.new.or(r.old).map(|n| n.to_string()).unwrap_or_default();
        let first = format!("{no:>num_w$} {sign} ");
        let rest = format!("{:>num_w$} {sign} ", "");
        let num_style = Style::new().fg(if bg.is_some() { fg } else { t.dim });
        let wrapped = markdown::wrap(spans, text_w + gutter, &first, &rest);
        for mut l in wrapped {
            if let Some(g) = l.spans.first_mut() {
                g.style = match bg {
                    Some(bg) => num_style.bg(bg),
                    None => num_style,
                };
            }
            if let Some(bg) = bg {
                let used: usize = l.spans.iter().map(|s| s.content.width()).sum();
                l.spans.push(Span::styled(" ".repeat(width.saturating_sub(used)), Style::new().bg(bg)));
            }
            out.push(l);
        }
    }
    out
}

/// A unified diff as text, for prompts.
pub fn unified(old: &str, new: &str, label: &str) -> String {
    TextDiff::from_lines(old, new).unified_diff().context_radius(2).header(&format!("{label} (before)"), &format!("{label} (now)")).to_string()
}

#[cfg(test)]
#[path = "delta_tests.rs"]
mod tests;
