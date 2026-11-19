//! Responsiveness of actions (docs/zendo/responsiveness.md): the time that the user waits for the
//! result of each action of the keymap, from the read of its input event to the end of the draw of
//! the screen that shows the result. An action that starts work in the background, a search or a
//! question to the agent, waits for the end of that work (RESP-1). The app times an action
//! where it counts it (`App::did`, USAGE-1), so the code of an action holds no measurement code
//! (RESP-2, ADR 0024). Each action becomes an event of the telemetry log with its time (TELEM-1).

use crate::keys::{ACTIONS, Pane};
use crate::telemetry::{Event, Trigger};
use chrono::Utc;
use std::collections::BTreeSet;
use std::time::{Duration, Instant};

/// The longest wait for a result: after it, the event of the action has no time (RESP-9).
const LONGEST: Duration = Duration::from_secs(512);

/// One piece of work in the background that the user waits for (RESP-1). Each piece has its own
/// identity: a new search or the next question is other work.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Work {
    /// The walk of the files of the open find panel.
    Walk,
    /// The search of query `n` of the find panel.
    Search(u64),
    /// The answer of the agent to question `n` (`Outgoing::n`), in its turn or in the queue. The
    /// number is unique in the process, so a rename of the thread does not change the work.
    Answer(usize),
}

/// One run of an action, from the read of its input event.
struct Run {
    pane: Pane,
    name: &'static str,
    trigger: Trigger,
    from: Instant,
}

impl Run {
    /// The event of the run, with the time of its input event on the clock of the wall.
    fn event(&self, wait: Option<Duration>) -> Event {
        let ago = chrono::Duration::from_std(Instant::now().saturating_duration_since(self.from)).unwrap_or_default();
        Event { at: Utc::now() - ago, pane: self.pane, action: self.name, trigger: self.trigger, wait }
    }
}

/// A run that waits for the draw and for the end of the work that it started.
struct Held {
    run: Run,
    works: BTreeSet<Work>,
}

/// The runs of the actions of this zendo process that wait for their results, and the events of
/// the runs that got them.
#[derive(Default)]
pub struct Responsiveness {
    /// Off: no run and no event (TELEM-12).
    on: bool,
    /// The read of the input event that the next actions answer.
    input: Option<Instant>,
    /// The runs of that input.
    waiting: Vec<Run>,
    /// The runs of earlier inputs, which wait for the draw and for the end of their work.
    held: Vec<Held>,
    /// The work that ran when that input came or at the last draw.
    seen: BTreeSet<Work>,
    /// The events that the app takes for the log.
    ready: Vec<Event>,
}

impl Responsiveness {
    /// The measurements on; `Default` is off, as in an app of a test.
    pub fn on() -> Responsiveness {
        Responsiveness { on: true, ..Responsiveness::default() }
    }

    /// On, unless `stats = false` (TELEM-12).
    pub fn for_process(on: bool) -> Responsiveness {
        if on { Responsiveness::on() } else { Responsiveness::default() }
    }

    /// The input thread read an input event at `at`; the work `now` runs before the loop handles
    /// it. The actions of the input before wait for the work that started since it.
    pub fn input(&mut self, at: Instant, now: &BTreeSet<Work>) {
        if self.on {
            self.close(now);
            self.input = Some(at);
        }
    }

    /// The actions of the last input wait for the work that started since that input (RESP-1).
    fn close(&mut self, now: &BTreeSet<Work>) {
        let started: BTreeSet<Work> = now.difference(&self.seen).cloned().collect();
        let mine = std::mem::take(&mut self.waiting).into_iter().map(|run| Held { run, works: started.clone() });
        self.held.extend(mine);
        self.seen = now.clone();
    }

    /// The action `name` of `pane` ran for the last input. Only an action of the keymap counts
    /// (USAGE-8). An action without an input gets its event at once, with no time.
    pub fn ran(&mut self, pane: Pane, name: &str, trigger: Trigger) {
        let Some(act) = ACTIONS.iter().find(|a| self.on && a.pane == pane && a.name == name) else { return };
        let run = Run { pane, name: act.name, trigger, from: self.input.unwrap_or_else(Instant::now) };
        match self.input {
            Some(_) => self.waiting.push(run),
            None => self.ready.push(run.event(None)),
        }
    }

    /// The draw of the screen ended at `at`, while the work `now` runs in the background. A run
    /// gets its event at the first draw after the end of its work, or with no time when it waited
    /// longer than 512 seconds (RESP-1, RESP-9).
    pub fn drawn(&mut self, at: Instant, now: &BTreeSet<Work>) {
        if !self.on {
            return;
        }
        self.close(now);
        self.input = None;
        let waited = |h: &Held| at.saturating_duration_since(h.run.from);
        let (done, held): (Vec<Held>, Vec<Held>) = std::mem::take(&mut self.held).into_iter().partition(|h| h.works.is_disjoint(now) || waited(h) > LONGEST);
        let events = done.iter().map(|h| h.run.event(Some(waited(h)).filter(|w| *w <= LONGEST)));
        self.ready.extend(events);
        self.held = held;
    }

    /// At the exit: each run that still waits gets its event with no time (RESP-9).
    pub fn end(&mut self) {
        let seen = self.seen.clone();
        self.close(&seen);
        let events = std::mem::take(&mut self.held).into_iter().map(|h| h.run.event(None));
        self.ready.extend(events);
    }

    /// The events of the runs that ended, for the log.
    pub fn take(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.ready)
    }
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
