//! The markers of list items: what Enter puts at the start of the next item (TEXT-8, TEXT-9).

/// The marker of a list item at the start of `line`: its length in chars, with the indent and the
/// whitespace after it, and the marker of the next item. `None`: the line is no list item.
pub(super) fn marker(line: &str) -> Option<(usize, String)> {
    let indent: String = line.chars().take_while(|c| *c == ' ' || *c == '\t').collect();
    let body = &line[indent.len()..];
    let (mark, next) = bullet(body).or_else(|| number(body))?;
    let rest = &body[mark.len()..];
    let space: String = rest.chars().take_while(|c| *c == ' ' || *c == '\t').collect();
    if space.is_empty() {
        return None;
    }
    let len = indent.chars().count() + mark.chars().count() + space.chars().count();
    Some((len, format!("{indent}{next}{space}")))
}

/// A bullet and the bullet of the next item: a done task goes on as an open task.
fn bullet(body: &str) -> Option<(&str, String)> {
    let task = ["- [ ]", "- [x]", "- [X]", "* [ ]", "* [x]", "+ [ ]"].into_iter().find(|t| body.starts_with(t));
    if let Some(t) = task {
        return Some((t, format!("{} [ ]", &t[..1])));
    }
    ["-", "*", "+"].into_iter().find(|b| body.starts_with(b)).map(|b| (b, b.to_string()))
}

/// A number with `.` or `)` after it, and the next number.
fn number(body: &str) -> Option<(&str, String)> {
    let digits = body.chars().take_while(char::is_ascii_digit).count();
    let close = body[digits..].chars().next().filter(|c| matches!(c, '.' | ')'))?;
    let n: u64 = body[..digits].parse().ok()?;
    Some((&body[..digits + 1], format!("{}{close}", n + 1)))
}
