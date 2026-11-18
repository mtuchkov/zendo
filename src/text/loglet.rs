//! A loglet: the operations of one input in order, and a head (UNDO-2, ADR 0014). The operations
//! before the head are done, the operations after it are undone. A new operation trims the tail
//! (UNDO-3); a change of another writer transforms every operation (UNDO-6).
use super::ot::Op;
use serde::{Deserialize, Serialize};

/// The most operations that a loglet keeps; the oldest go first.
pub const MAX_OPS: usize = 500;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Loglet {
    ops: Vec<Op>,
    head: usize,
}

impl Loglet {
    /// A new operation at the head: the undone operations go (UNDO-3).
    pub fn record(&mut self, op: Op) {
        if op.is_noop() {
            return;
        }
        self.ops.truncate(self.head);
        self.ops.push(op);
        let over = self.ops.len().saturating_sub(MAX_OPS);
        self.ops.drain(..over);
        self.head = self.ops.len();
    }

    /// The operation before the head, the last done one.
    pub fn last(&self) -> Option<&Op> {
        self.head.checked_sub(1).and_then(|i| self.ops.get(i))
    }

    /// A larger operation takes the place of the last done one: a word that the user types goes
    /// on (UNDO-7). The undone operations go.
    pub fn replace_last(&mut self, op: Op) {
        if self.head == 0 {
            return self.record(op);
        }
        self.ops.truncate(self.head);
        self.ops[self.head - 1] = op;
    }

    /// The operation that undoes the last done one; the head moves back (UNDO-9: none at the start).
    pub fn undo(&mut self) -> Option<Op> {
        self.head = self.head.checked_sub(1)?;
        Some(self.ops[self.head].invert())
    }

    /// The operation after the head; the head moves forward (UNDO-9: none at the end).
    pub fn redo(&mut self) -> Option<Op> {
        let op = self.ops.get(self.head)?.clone();
        self.head += 1;
        Some(op)
    }

    /// Another writer changed the text at the head with `x`: every operation moves over it, so
    /// that undo and redo keep the change (UNDO-6).
    pub fn rebase(&mut self, x: Op) {
        if x.is_noop() {
            return;
        }
        let mut down = x.clone();
        for op in self.ops[..self.head].iter_mut().rev() {
            let (inv, next) = Op::transform(&op.invert(), &down);
            *op = inv.invert();
            down = next;
        }
        let mut up = x;
        for op in &mut self.ops[self.head..] {
            let (moved, next) = Op::transform(op, &up);
            *op = moved;
            up = next;
        }
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

#[cfg(test)]
#[path = "loglet_tests.rs"]
mod tests;
