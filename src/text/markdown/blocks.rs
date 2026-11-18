//! Source lines joined into logical lines, and the kind of each block line.
use super::*;

/// One logical line: source lines that markdown reads as one paragraph or list item.
pub(super) struct Logical {
    pub(super) text: String,
    pub(super) mark: Mark,
    /// Source lines in it.
    pub(super) n: usize,
    /// Front matter: shown as it is, dim.
    pub(super) verbatim: bool,
}

pub(super) fn is_rule(t: &str) -> bool {
    let c: String = t.chars().filter(|c| !c.is_whitespace()).collect();
    c.len() >= 3 && (c.chars().all(|x| x == '-') || c.chars().all(|x| x == '*') || c.chars().all(|x| x == '_'))
}

/// A line that starts a block of its own and never continues the line before it.
pub(super) fn starts_block(t: &str) -> bool {
    t.is_empty() || heading(t).is_some() || bullet(t).is_some() || t.starts_with('>') || t.starts_with("```") || t.starts_with('|') || t.starts_with('<') || is_rule(t)
}

/// Join hard-wrapped lines: markdown reads them as one paragraph, so the reader rewraps them
/// to the width of the pane. Code, tables, headings, front matter and hard breaks stay.
pub(super) fn join(src: &[(String, Mark)]) -> Vec<Logical> {
    let mut out: Vec<Logical> = Vec::new();
    let mut fence = false;
    let mut front = src.first().is_some_and(|l| l.0.trim() == "---");
    for (i, (raw, mark)) in src.iter().enumerate() {
        let t = raw.trim_start();
        let new = |verbatim| Logical { text: raw.clone(), mark: *mark, n: 1, verbatim };
        if front {
            out.push(new(true));
            if i > 0 && t.trim() == "---" {
                front = false;
            }
            continue;
        }
        if t.starts_with("```") {
            fence = !fence;
            out.push(new(false));
            continue;
        }
        if fence {
            out.push(new(false));
            continue;
        }
        let quote_more = t.starts_with('>');
        let joins = out.last().is_some_and(|l| {
            let lt = l.text.trim_start();
            let open = !lt.is_empty() && !l.verbatim && l.mark == *mark && heading(lt).is_none() && !lt.starts_with("```") && !lt.starts_with('|') && !lt.starts_with('<') && !is_rule(lt);
            let hard_break = l.text.ends_with("  ") || l.text.ends_with('\\');
            let quote = lt.starts_with('>');
            open && !hard_break && if quote { quote_more } else { !starts_block(t) }
        });
        if joins {
            let l = out.last_mut().expect("checked");
            let more = if quote_more { t.trim_start_matches('>').trim() } else { t.trim() };
            l.text = format!("{} {more}", l.text.trim_end());
            l.n += 1;
        } else {
            out.push(new(false));
        }
    }
    out
}

pub(super) fn only_tags(s: &str) -> bool {
    s.split_whitespace().all(|w| w.len() > 1 && w.starts_with('#') && !w.starts_with("##"))
}

pub(super) fn heading(s: &str) -> Option<(usize, &str)> {
    let level = s.chars().take_while(|c| *c == '#').count();
    if (1..=6).contains(&level) && s[level..].starts_with(' ') {
        Some((level, s[level..].trim()))
    } else {
        None
    }
}

/// The box of a task item: open or done; `None` for a line that is no task.
pub fn task_box(s: &str) -> Option<char> {
    let s = s.trim_start();
    let rest = ["- ", "* ", "+ "].into_iter().find_map(|p| s.strip_prefix(p))?;
    match rest.get(..4)? {
        "[ ] " => Some(TASK_OPEN),
        "[x] " | "[X] " => Some(TASK_DONE),
        _ => None,
    }
}

/// The line with its task checked, or open again; `None` for a line that is no task (TEXT-14).
pub fn toggle_task(line: &str) -> Option<String> {
    let at = line.len() - line.trim_start().len() + 2;
    let next = if task_box(line)? == TASK_OPEN { "[x]" } else { "[ ]" };
    Some(format!("{}{next}{}", &line[..at], &line[at + 3..]))
}

/// The box of an open task and of a done task on the screen.
pub const TASK_OPEN: char = '☐';
pub const TASK_DONE: char = '☑';

pub(super) fn bullet(s: &str) -> Option<&str> {
    for p in ["- [ ] ", "- [x] ", "- [X] ", "* [ ] ", "* [x] ", "+ [ ] ", "+ [x] ", "- ", "* ", "+ "] {
        if let Some(r) = s.strip_prefix(p) {
            return Some(r);
        }
    }
    let digits = s.chars().take_while(|c| c.is_ascii_digit()).count();
    if digits > 0 && s[digits..].starts_with(". ") {
        return Some(&s[digits + 2..]);
    }
    None
}
