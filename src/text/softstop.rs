//! Soft stops of the selection keys in a table with box borders (PANES-42): while a selection that
//! starts in a table grows, the cursor stops at the edge of its cell. Two quick presses in one
//! direction, the stop and one more, take the cursor on to the next cell in that direction.

use super::boxtable::{col_of, index_at, is_rule, table_at, Table};
use std::time::{Duration, Instant};

/// The second press of the two comes at most this long after the stop.
pub const QUICK: Duration = Duration::from_millis(400);
/// A press sooner than this after the stop comes from a held key: it stops again.
pub const REPEAT: Duration = Duration::from_millis(60);

/// The direction of a cursor key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dir {
    Left,
    Right,
    Up,
    Down,
}

impl Dir {
    /// The direction of a cursor action of the keymap; None for other actions.
    pub fn of(action: &str) -> Option<Dir> {
        match action {
            "left" | "select_left" | "select_word_left" => Some(Dir::Left),
            "right" | "select_right" | "select_word_right" => Some(Dir::Right),
            "up" | "select_up" => Some(Dir::Up),
            "down" | "select_down" => Some(Dir::Down),
            _ => None,
        }
    }

    pub fn horizontal(self) -> bool {
        matches!(self, Dir::Left | Dir::Right)
    }
}

/// The cell of a cursor in display columns: the lines left and right of it, and its rows between
/// the rules of the table.
struct Cell {
    left: usize,
    right: usize,
    top: usize,
    bottom: usize,
}

impl Cell {
    fn of<S: AsRef<str>>(rows: &[S], t: &Table, (r, c): (usize, usize)) -> Option<Cell> {
        if r < t.top || r > t.bottom || t.lines.contains(&c) {
            return None;
        }
        let left = *t.lines.iter().rev().find(|l| **l < c)?;
        let right = *t.lines.iter().find(|l| **l > c)?;
        let top = (t.top..=r).rev().take_while(|i| !is_rule(rows, *i, t)).last().unwrap_or(r);
        let bottom = (r..=t.bottom).take_while(|i| !is_rule(rows, *i, t)).last().unwrap_or(r);
        Some(Cell { left, right, top, bottom })
    }

    fn contains(&self, (r, c): (usize, usize)) -> bool {
        (self.top..=self.bottom).contains(&r) && c > self.left && c < self.right
    }

    /// The last position in the cell toward `dir`, on the row or in the column of `at`.
    fn edge(&self, (r, c): (usize, usize), dir: Dir) -> (usize, usize) {
        match dir {
            Dir::Left => (r, self.left + 1),
            Dir::Right => (r, self.right - 1),
            Dir::Up => (self.top, c),
            Dir::Down => (self.bottom, c),
        }
    }

    /// The first position of the next cell toward `dir`; None at the border of the table.
    fn next(&self, t: &Table, (r, c): (usize, usize), dir: Dir) -> Option<(usize, usize)> {
        let (first, last) = (t.lines[0], t.lines[t.lines.len() - 1]);
        match dir {
            Dir::Left => (self.left > first).then(|| (r, self.left - 1)),
            Dir::Right => (self.right < last).then(|| (r, self.right + 1)),
            Dir::Up => (self.top >= t.top + 2).then(|| (self.top - 2, c)),
            Dir::Down => (self.bottom + 2 <= t.bottom).then(|| (self.bottom + 2, c)),
        }
    }
}

/// The last stop at the edge of a cell: its direction, the position of the cursor, and its time.
#[derive(Clone, Copy, Debug, Default)]
pub struct SoftStop {
    last: Option<(Dir, (usize, usize), Instant)>,
}

impl SoftStop {
    /// Where the cursor goes when a key toward `dir` moves it from `cursor` to `target`, in a
    /// selection from `anchor`. Positions are (row, char index). Outside a table: `target`.
    pub fn step<S: AsRef<str>>(&mut self, rows: &[S], anchor: (usize, usize), cursor: (usize, usize), target: (usize, usize), dir: Dir, now: Instant) -> (usize, usize) {
        let col = |(r, i): (usize, usize)| (r, rows.get(r).map_or(i, |row| col_of(row.as_ref(), i)));
        let index = |(r, c): (usize, usize)| (r, rows.get(r).map_or(c, |row| index_at(row.as_ref(), c)));
        let (a, cur) = (col(anchor), col(cursor));
        let Some((t, cell)) = table_at(rows, a.0, a.1).and_then(|t| Cell::of(rows, &t, cur).map(|c| (t, c))) else {
            self.last = None;
            return target;
        };
        if cell.contains(col(target)) {
            self.last = None;
            return target;
        }
        let edge = cell.edge(cur, dir);
        if edge != cur {
            self.last = Some((dir, edge, now));
            return index(edge);
        }
        let quick = self.last.is_some_and(|(d, at, when)| d == dir && at == cur && (REPEAT..=QUICK).contains(&now.saturating_duration_since(when)));
        if quick {
            self.last = None;
            return cell.next(&t, cur, dir).map_or(cursor, index);
        }
        self.last = Some((dir, cur, now));
        cursor
    }
}

#[cfg(test)]
#[path = "softstop_tests.rs"]
mod tests;
