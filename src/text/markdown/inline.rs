//! Inline markdown: emphasis, code, links, chips of EARS keywords.
use super::*;

/// Inline styling: `code`, **bold**, [[link]], [text](url), #tag, EARS words.
pub fn inline(s: &str, base: Style, t: &Theme) -> Vec<Span<'static>> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut buf = String::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    let flush = |buf: &mut String, spans: &mut Vec<Span<'static>>| {
        if !buf.is_empty() {
            spans.extend(words(buf, base, t));
            buf.clear();
        }
    };
    while i < chars.len() {
        if let Some((marked, n)) = emphasis(&chars, i, base, t).or_else(|| link(&chars, i, base, t)) {
            flush(&mut buf, &mut spans);
            spans.extend(marked);
            i += n;
            continue;
        }
        buf.push(chars[i]);
        i += 1;
    }
    flush(&mut buf, &mut spans);
    spans
}

/// Code, bold, italic and underline at `i`: the spans and the chars they take.
fn emphasis(chars: &[char], i: usize, base: Style, t: &Theme) -> Option<(Vec<Span<'static>>, usize)> {
    let c = chars[i];
    let rest: String = chars[i..].iter().collect();
    if c == '`'
        && let Some(end) = rest[1..].find('`') {
            let code = &rest[1..1 + end];
            return Some((vec![Span::styled(format!(" {code} "), base.bg(t.panel))], code.chars().count() + 2));
        }
    if rest.starts_with("***")
        && let Some(end) = rest[3..].find("***") {
            let inner = &rest[3..3 + end];
            return Some((inline(inner, base.add_modifier(Modifier::BOLD | Modifier::ITALIC), t), inner.chars().count() + 6));
        }
    if rest.starts_with("<u>")
        && let Some(end) = rest[3..].find("</u>") {
            let inner = &rest[3..3 + end];
            return Some((inline(inner, base.add_modifier(Modifier::UNDERLINED), t), inner.chars().count() + 7));
        }
    // *italic* and _italic_: the mark touches the word on both sides; _ only at a word edge.
    let edge = i == 0 || !chars[i - 1].is_alphanumeric();
    if (c == '*' && !rest.starts_with("**") || c == '_' && edge)
        && chars.get(i + 1).is_some_and(|n| !n.is_whitespace() && *n != c)
        && let Some(end) = rest[1..].find(c)
        && end > 0
        && !rest[1..1 + end].ends_with(' ')
        && (c == '*' || !rest[2 + end..].starts_with(|n: char| n.is_alphanumeric())) {
            let inner = &rest[1..1 + end];
            return Some((inline(inner, base.add_modifier(Modifier::ITALIC), t), inner.chars().count() + 2));
        }
    if rest.starts_with("**")
        && let Some(end) = rest[2..].find("**") {
            let inner = &rest[2..2 + end];
            return Some((inline(inner, base.add_modifier(Modifier::BOLD), t), inner.chars().count() + 4));
        }
    None
}

/// `[[name]]` and `[text](url)` at `i`: the link text and the chars it takes.
fn link(chars: &[char], i: usize, base: Style, t: &Theme) -> Option<(Vec<Span<'static>>, usize)> {
    let link = base.fg(t.acc2).add_modifier(Modifier::UNDERLINED);
    let rest: String = chars[i..].iter().collect();
    if rest.starts_with("[[")
        && let Some(end) = rest.find("]]") {
            let name = &rest[2..end];
            return Some((vec![Span::styled(name.to_string(), link)], name.chars().count() + 4));
        }
    if chars[i] == '['
        && let Some(close) = rest.find("](")
        && let Some(end) = rest[close..].find(')') {
            let text = &rest[1..close];
            return Some((vec![Span::styled(text.to_string(), link)], rest[..close + end + 1].chars().count()));
        }
    None
}

/// Split plain text into words so tags and EARS keywords get their own style.
pub(super) fn words(s: &str, base: Style, t: &Theme) -> Vec<Span<'static>> {
    let mut out: Vec<Span<'static>> = Vec::new();
    let mut plain = String::new();
    for piece in s.split_inclusive(' ') {
        let word = piece.trim_end();
        let bare = word.trim_end_matches([',', '.', ':', ';']);
        let special = if bare.len() > 1 && bare.starts_with('#') && bare[1..].chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            Some(base.fg(t.acc2))
        } else if EARS.contains(&bare) {
            Some(base.fg(t.dim))
        } else {
            None
        };
        match special {
            Some(style) => {
                if !plain.is_empty() {
                    out.push(Span::styled(std::mem::take(&mut plain), base));
                }
                out.push(Span::styled(bare.to_string(), style));
                plain.push_str(&piece[bare.len()..]);
            }
            None => plain.push_str(piece),
        }
    }
    if !plain.is_empty() {
        out.push(Span::styled(plain, base));
    }
    out
}
