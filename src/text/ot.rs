//! Operations on a text, for undo and redo with operational transformation (UNDO-2, UNDO-6,
//! ADR 0014). An operation is a list of parts over the chars of a text: retain chars, insert a
//! text, delete a text. A delete keeps its text, so each operation has an inverse.
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// One part of an operation. In JSON: `{"r": 3}`, `{"i": "text"}`, `{"d": "text"}`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Part {
    #[serde(rename = "r")]
    Retain(usize),
    #[serde(rename = "i")]
    Insert(String),
    #[serde(rename = "d")]
    Delete(String),
}

impl Part {
    /// The chars that the part goes over: in the old text for retain and delete, in the new text
    /// for insert.
    fn len(&self) -> usize {
        match self {
            Part::Retain(n) => *n,
            Part::Insert(s) | Part::Delete(s) => s.chars().count(),
        }
    }

    /// The first `n` chars of the part, and the rest.
    fn split(self, n: usize) -> (Part, Option<Part>) {
        let cut = |s: String| {
            let (a, b) = split_chars(&s, n).unwrap_or((&s, ""));
            (a.to_string(), (!b.is_empty()).then(|| b.to_string()))
        };
        match self {
            Part::Retain(m) if m > n => (Part::Retain(n), Some(Part::Retain(m - n))),
            Part::Insert(s) => {
                let (a, b) = cut(s);
                (Part::Insert(a), b.map(Part::Insert))
            }
            Part::Delete(s) => {
                let (a, b) = cut(s);
                (Part::Delete(a), b.map(Part::Delete))
            }
            p => (p, None),
        }
    }
}

/// The text before the char `n`, and the rest; `None` when the text has fewer chars.
fn split_chars(s: &str, n: usize) -> Option<(&str, &str)> {
    let at = match s.char_indices().nth(n) {
        Some((b, _)) => b,
        None if s.chars().count() == n => s.len(),
        None => return None,
    };
    Some(s.split_at(at))
}

/// A change of a text: the parts from its start to its end.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Op(Vec<Part>);

impl Op {
    fn retain(&mut self, n: usize) {
        match self.0.last_mut() {
            _ if n == 0 => {}
            Some(Part::Retain(m)) => *m += n,
            _ => self.0.push(Part::Retain(n)),
        }
    }

    fn insert(&mut self, s: &str) {
        match self.0.last_mut() {
            _ if s.is_empty() => {}
            Some(Part::Insert(t)) => t.push_str(s),
            _ => self.0.push(Part::Insert(s.to_string())),
        }
    }

    fn delete(&mut self, s: &str) {
        match self.0.last_mut() {
            _ if s.is_empty() => {}
            Some(Part::Delete(t)) => t.push_str(s),
            _ => self.0.push(Part::Delete(s.to_string())),
        }
    }

    fn push(&mut self, p: &Part) {
        match p {
            Part::Retain(n) => self.retain(*n),
            Part::Insert(s) => self.insert(s),
            Part::Delete(s) => self.delete(s),
        }
    }

    /// The operation that makes `b` from `a`: the chars between the same start and the same end
    /// go, the new chars come in their place.
    pub fn diff(a: &str, b: &str) -> Op {
        let (ac, bc): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
        let pre = ac.iter().zip(&bc).take_while(|(x, y)| x == y).count();
        let most = ac.len().min(bc.len()) - pre;
        let suf = ac.iter().rev().zip(bc.iter().rev()).take(most).take_while(|(x, y)| x == y).count();
        let text = |c: &[char]| c.iter().collect::<String>();
        let mut op = Op::default();
        op.retain(pre);
        op.insert(&text(&bc[pre..bc.len() - suf]));
        op.delete(&text(&ac[pre..ac.len() - suf]));
        op.retain(suf);
        op
    }

    /// The text after the operation; `None` when the operation does not fit the text (UNDO-11).
    pub fn apply(&self, s: &str) -> Option<String> {
        let mut out = String::with_capacity(s.len());
        let mut rest = s;
        for p in &self.0 {
            match p {
                Part::Retain(n) => {
                    let (a, b) = split_chars(rest, *n)?;
                    out.push_str(a);
                    rest = b;
                }
                Part::Insert(t) => out.push_str(t),
                Part::Delete(t) => rest = rest.strip_prefix(t.as_str())?,
            }
        }
        rest.is_empty().then_some(out)
    }

    /// The operation that takes this one back.
    pub fn invert(&self) -> Op {
        let swap = |p: &Part| match p {
            Part::Retain(n) => Part::Retain(*n),
            Part::Insert(s) => Part::Delete(s.clone()),
            Part::Delete(s) => Part::Insert(s.clone()),
        };
        Op(self.0.iter().map(swap).collect())
    }

    /// The operation changes nothing.
    pub fn is_noop(&self) -> bool {
        self.0.iter().all(|p| matches!(p, Part::Retain(_)))
    }

    /// The char in the new text after the last change: the cursor goes there (UNDO-8).
    pub fn end(&self) -> usize {
        let (mut at, mut end) = (0, 0);
        for p in &self.0 {
            match p {
                Part::Retain(n) => at += n,
                Part::Insert(s) => {
                    at += s.chars().count();
                    end = at;
                }
                Part::Delete(_) => end = at,
            }
        }
        end
    }

    /// The one change of a simple operation: where it starts, and the inserted or the deleted
    /// text. `None` when the operation changes more than one place, or inserts and deletes.
    pub fn single(&self) -> Option<(usize, &str, &str)> {
        let (mut at, mut change) = (0, None);
        for p in &self.0 {
            match p {
                Part::Retain(n) if change.is_none() => at += n,
                Part::Retain(_) => {}
                Part::Insert(s) if change.is_none() => change = Some((s.as_str(), "")),
                Part::Delete(s) if change.is_none() => change = Some(("", s.as_str())),
                _ => return None,
            }
        }
        change.map(|(i, d)| (at, i, d))
    }

    /// Two operations on the same text: `a2` does `a` after `b`, `b2` does `b` after `a`, and both
    /// orders give the same text. At the same place, the insert of `a` goes first (UNDO-6).
    pub fn transform(a: &Op, b: &Op) -> (Op, Op) {
        let (mut x, mut y) = (Parts(a.0.iter().cloned().collect()), Parts(b.0.iter().cloned().collect()));
        let (mut a2, mut b2) = (Op::default(), Op::default());
        loop {
            match (x.0.front(), y.0.front()) {
                (Some(Part::Insert(s)), _) => {
                    b2.retain(s.chars().count());
                    a2.push(&x.take(usize::MAX));
                }
                (_, Some(Part::Insert(s))) => {
                    a2.retain(s.chars().count());
                    b2.push(&y.take(usize::MAX));
                }
                (Some(p), Some(q)) => {
                    let n = p.len().min(q.len());
                    let (p, q) = (x.take(n), y.take(n));
                    over(&p, &q, &mut a2, &mut b2);
                }
                _ => break,
            }
        }
        (a2, b2)
    }
}

/// A retain or a delete of `a` and of `b` over the same chars.
fn over(p: &Part, q: &Part, a2: &mut Op, b2: &mut Op) {
    match (p, q) {
        (Part::Retain(n), Part::Retain(_)) => {
            a2.retain(*n);
            b2.retain(*n);
        }
        (Part::Delete(s), Part::Retain(_)) => a2.delete(s),
        (Part::Retain(_), Part::Delete(s)) => b2.delete(s),
        _ => {}
    }
}

/// The parts of an operation that the transform did not take yet.
struct Parts(VecDeque<Part>);

impl Parts {
    /// The next `n` chars of the first part; the rest of it stays first.
    fn take(&mut self, n: usize) -> Part {
        let first = self.0.pop_front().expect("a part");
        let (head, rest) = first.split(n);
        if let Some(r) = rest {
            self.0.push_front(r);
        }
        head
    }
}

#[cfg(test)]
#[path = "ot_tests.rs"]
mod tests;
