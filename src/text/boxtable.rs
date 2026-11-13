//! A selection that starts in a cell of a table with box borders (`┌ ─ ┬ │ ├ ┼ └`, heavy, double,
//! dashed and rounded too: the Box Drawing block of Unicode) keeps to the
//! columns of the table (PANES-34). The table is found in the rows around the start (PANES-35);
//! tables drawn with `|`, `-` and `+` are plain text (PANES-36). Positions are display columns: a
//! wide char covers two.

use unicode_width::UnicodeWidthChar;

/// How far up and down from the start of a selection the borders of a table can be (PANES-35).
pub const REACH: usize = 100;

// The Box Drawing block (U+2500 to U+257F) by the arms of each char, after its Unicode name. The
// diagonals `╱ ╲ ╳` are in no class.
/// Up and down only: the vertical lines.
const VERTICAL: &str = "│┃┆┇┊┋╎╏║╽╿";
/// Left and right only: the horizontal lines.
const HORIZONTAL: &str = "─━┄┅┈┉╌╍═╴╶╸╺╼╾";
/// Down and no up: the top border.
const TOP: &str = "┌┍┎┏┐┑┒┓┬┭┮┯┰┱┲┳╒╓╔╕╖╗╤╥╦╭╮╷╻";
/// Up and no down: the bottom border.
const BOTTOM: &str = "└┕┖┗┘┙┚┛┴┵┶┷┸┹┺┻╘╙╚╛╜╝╧╨╩╯╰╵╹";
/// Up, down and a side: a rule inside the table, such as `├` or `┼`.
const MIDDLE: &str = "├┝┞┟┠┡┢┣┤┥┦┧┨┩┪┫┼┽┾┿╀╁╂╃╄╅╆╇╈╉╊╋╞╟╠╡╢╣╪╫╬";

/// A char of the lines of a table, of any class.
fn is_box(c: char) -> bool {
    [VERTICAL, HORIZONTAL, TOP, BOTTOM, MIDDLE].iter().any(|class| class.contains(c))
}

fn is_vertical(c: char) -> bool {
    VERTICAL.contains(c)
}

fn is_middle(c: char) -> bool {
    MIDDLE.contains(c)
}

fn width(c: char) -> usize {
    c.width().unwrap_or(0)
}

/// The char that covers display column `col` of `row`.
pub fn char_at(row: &str, col: usize) -> Option<char> {
    let mut x = 0;
    row.chars().find(|c| {
        let hit = width(*c) > 0 && col < x + width(*c);
        x += width(*c);
        hit
    })
}

/// The display column of char `i` of `row`.
pub fn col_of(row: &str, i: usize) -> usize {
    row.chars().take(i).map(width).sum()
}

/// The index of the char that covers display column `col`; past the end, the number of chars.
pub fn index_at(row: &str, col: usize) -> usize {
    let mut x = 0;
    let hit = row.chars().position(|c| {
        let hit = width(c) > 0 && col < x + width(c);
        x += width(c);
        hit
    });
    hit.unwrap_or_else(|| row.chars().count())
}

/// The display columns `from..to` of `row`.
fn cut(row: &str, from: usize, to: usize) -> String {
    let mut x = 0;
    row.chars()
        .filter(|c| {
            let inside = x >= from && x < to;
            x += width(*c);
            inside
        })
        .collect()
}

fn row_width(row: &str) -> usize {
    row.chars().map(width).sum()
}

/// A table with box borders: its rows inside the borders, and the columns of its vertical lines.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Table {
    pub top: usize,
    pub bottom: usize,
    pub lines: Vec<usize>,
}

/// The table around display column `col` of row `r`: `col` is in a cell, between two vertical lines,
/// and the line on its left goes to a border of the table within REACH rows (PANES-35). Where the
/// rows end first (a pane shows a part of the table), the other border is enough.
pub fn table_at<S: AsRef<str>>(rows: &[S], r: usize, col: usize) -> Option<Table> {
    let row = rows.get(r)?.as_ref();
    if is_box(char_at(row, col)?) {
        return None;
    }
    let mut x = 0;
    let lines: Vec<usize> = row.chars().filter_map(|c| {
        let at = x;
        x += width(c);
        is_vertical(c).then_some(at)
    }).collect();
    let left = *lines.iter().rev().find(|l| **l < col)?;
    lines.iter().find(|l| **l > col)?;
    let (top, top_border) = edge(rows, r, left, true)?;
    let (bottom, bottom_border) = edge(rows, r, left, false)?;
    (top_border || bottom_border).then_some(Table { top, bottom, lines })
}

/// Row `r` is a rule inside table `t`, such as `├───┼───┤`.
pub(crate) fn is_rule<S: AsRef<str>>(rows: &[S], r: usize, t: &Table) -> bool {
    rows.get(r).and_then(|row| char_at(row.as_ref(), t.lines[0])).is_some_and(is_middle)
}

/// From row `r` along the vertical line at `col`, up or down: the last row inside the table, and
/// whether a border ends it. None when the line breaks.
fn edge<S: AsRef<str>>(rows: &[S], r: usize, col: usize, up: bool) -> Option<(usize, bool)> {
    let end = if up { TOP } else { BOTTOM };
    let mut last = r;
    for step in 1..=REACH {
        let Some(row) = (if up { r.checked_sub(step) } else { Some(r + step) }).and_then(|i| rows.get(i)) else { break };
        match char_at(row.as_ref(), col) {
            Some(c) if end.contains(c) => return Some((last, true)),
            Some(c) if is_vertical(c) || is_middle(c) => last = if up { r - step } else { r + step },
            _ => return None,
        }
    }
    Some((last, false))
}

/// A selection over rows of text, start before end, the end column in it. In a table, `cols` keeps
/// it to the columns `from..to` between the cell of its start and the cell of its end, and it skips
/// the rules of the table.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Range {
    pub start: (usize, usize),
    pub end: (usize, usize),
    pub cols: Option<(usize, usize)>,
    rules: Vec<usize>,
}

impl Range {
    /// The selected display columns of row `r`, end exclusive; `len` is the width of a plain row.
    pub fn span(&self, r: usize, len: usize) -> Option<(usize, usize)> {
        if r < self.start.0 || r > self.end.0 || self.rules.contains(&r) {
            return None;
        }
        let (left, right) = self.cols.unwrap_or((0, len));
        let from = if r == self.start.0 { self.start.1 } else { left };
        let to = if r == self.end.0 { self.end.1 + 1 } else { right };
        Some((from, to.min(right).max(from)))
    }

    /// The selected chars of row `r`, end exclusive.
    pub fn chars<S: AsRef<str>>(&self, rows: &[S], r: usize) -> Option<(usize, usize)> {
        let row = rows.get(r).map_or("", |s| s.as_ref());
        let (from, to) = self.span(r, row_width(row))?;
        Some((index_at(row, from), index_at(row, to)))
    }

    /// The selected text. In a table: the cells of a row with a tab between them, without the
    /// lines, and no row whose cells are all empty (PANES-40).
    pub fn text<S: AsRef<str>>(&self, rows: &[S]) -> String {
        let out: Vec<String> = (self.start.0..=self.end.0)
            .filter_map(|r| {
                let row = rows.get(r)?.as_ref();
                let (from, to) = self.span(r, row_width(row))?;
                let part = cut(row, from, to);
                if self.cols.is_none() {
                    return Some(part.trim_end().to_string());
                }
                let cells: Vec<&str> = part.split(is_vertical).map(str::trim).collect();
                cells.iter().any(|c| !c.is_empty()).then(|| cells.join("\t"))
            })
            .collect();
        out.join("\n")
    }
}

/// The selection from `anchor` to `head`, in display columns (row, col). When the anchor is in a
/// cell of a table, the head stays in the rows and columns of the table; a head above or below the
/// table takes the column of the anchor to its end (PANES-41).
pub fn resolve<S: AsRef<str>>(rows: &[S], anchor: (usize, usize), head: (usize, usize)) -> Range {
    match table_at(rows, anchor.0, anchor.1) {
        Some(t) => within(rows, &t, anchor, head),
        None => {
            let (start, end) = if (anchor.0, anchor.1) <= (head.0, head.1) { (anchor, head) } else { (head, anchor) };
            Range { start, end, cols: None, rules: Vec::new() }
        }
    }
}

/// The selection from `anchor` to `head` in chars (row, char index), when the anchor is in a table.
pub fn in_table<S: AsRef<str>>(rows: &[S], anchor: (usize, usize), head: (usize, usize)) -> Option<Range> {
    let col = |(r, i): (usize, usize)| (r, rows.get(r).map_or(i, |row| col_of(row.as_ref(), i)));
    let (anchor, head) = (col(anchor), col(head));
    table_at(rows, anchor.0, anchor.1).map(|t| within(rows, &t, anchor, head))
}

fn within<S: AsRef<str>>(rows: &[S], t: &Table, anchor: (usize, usize), head: (usize, usize)) -> Range {
    let (first, last) = (t.lines[0], t.lines[t.lines.len() - 1]);
    let cell = |c: usize| {
        let left = t.lines.iter().rev().find(|l| **l < c).copied().unwrap_or(first);
        (left, t.lines.iter().find(|l| **l >= c).copied().unwrap_or(last))
    };
    let (al, ar) = cell(anchor.1);
    // Above or below the table, the selection goes to the end of the column of the anchor.
    let head = match head.0 {
        r if r < t.top => (t.top, al + 1),
        r if r > t.bottom => (t.bottom, ar - 1),
        r => (r, head.1.clamp(first + 1, last - 1)),
    };
    let (hl, hr) = cell(head.1);
    let (from, to) = (al.min(hl) + 1, ar.max(hr));
    let (start, end) = if anchor <= head { (anchor, head) } else { (head, anchor) };
    let fit = |(r, c): (usize, usize)| (r, c.clamp(from, to - 1));
    let rules = (start.0..=end.0).filter(|r| is_rule(rows, *r, t)).collect();
    Range { start: fit(start), end: fit(end), cols: Some((from, to)), rules }
}

#[cfg(test)]
#[path = "boxtable_tests.rs"]
mod tests;
