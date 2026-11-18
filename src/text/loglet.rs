//! A loglet: the operations of one input in order, and a head (UNDO-2, ADR 0014). The operations
//! before the head are done, the operations after it are undone. A new operation trims the tail
//! (UNDO-3). A change of another writer is an operation of its own, marked as external (UNDO-6).
use super::ot::Op;
use serde::{Deserialize, Serialize};

/// The most operations that a loglet keeps; the oldest go first.
pub const MAX_OPS: usize = 500;

/// One operation of the loglet.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Entry {
    op: Op,
    /// Another writer made the change: the agent, git, another editor (UNDO-6).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    external: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Loglet {
    ops: Vec<Entry>,
    head: usize,
}

impl Loglet {
    /// A new operation of the user at the head: the undone operations go (UNDO-3).
    pub fn record(&mut self, op: Op) {
        self.push(Entry { op, external: false });
    }

    /// A change of another writer at the head, as one operation (UNDO-6). The undone operations go.
    pub fn record_external(&mut self, op: Op) {
        self.push(Entry { op, external: true });
    }

    fn push(&mut self, e: Entry) {
        if e.op.is_noop() {
            return;
        }
        self.ops.truncate(self.head);
        self.ops.push(e);
        let over = self.ops.len().saturating_sub(MAX_OPS);
        self.ops.drain(..over);
        self.head = self.ops.len();
    }

    /// The operation before the head, the last done one.
    pub fn last(&self) -> Option<&Op> {
        self.head.checked_sub(1).and_then(|i| self.ops.get(i)).map(|e| &e.op)
    }

    /// The next undo takes back a change of another writer: zendo asks first (UNDO-12).
    pub fn external_next(&self) -> bool {
        self.head.checked_sub(1).and_then(|i| self.ops.get(i)).is_some_and(|e| e.external)
    }

    /// A larger operation takes the place of the last done one: a word that the user types goes
    /// on (UNDO-7). The undone operations go.
    pub fn replace_last(&mut self, op: Op) {
        if self.head == 0 {
            return self.record(op);
        }
        self.ops.truncate(self.head);
        self.ops[self.head - 1].op = op;
    }

    /// The operation that undoes the last done one; the head moves back (UNDO-9: none at the start).
    pub fn undo(&mut self) -> Option<Op> {
        self.head = self.head.checked_sub(1)?;
        Some(self.ops[self.head].op.invert())
    }

    /// The operation after the head; the head moves forward (UNDO-9: none at the end).
    pub fn redo(&mut self) -> Option<Op> {
        let op = self.ops.get(self.head)?.op.clone();
        self.head += 1;
        Some(op)
    }

    /// The user keeps the external change before the head: it leaves the loglet, and the
    /// operations before it move over it, so that their undo keeps the change (UNDO-14).
    pub fn keep_external(&mut self) {
        if !self.external_next() {
            return;
        }
        self.head -= 1;
        let x = self.ops.remove(self.head).op;
        move_down(&mut self.ops[..self.head], x);
    }

    pub fn head(&self) -> usize {
        self.head
    }

    pub fn len(&self) -> usize {
        self.ops.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }
}

/// The done operations `ops`, last at the end, move over the change `x` after them.
fn move_down(ops: &mut [Entry], x: Op) {
    let mut down = x;
    for e in ops.iter_mut().rev() {
        let (inv, next) = Op::transform(&e.op.invert(), &down);
        e.op = inv.invert();
        down = next;
    }
}

#[cfg(test)]
#[path = "loglet_tests.rs"]
mod tests;
