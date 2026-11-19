//! The questions of the agent in the chat: the user picks a choice with the arrows, `space` and
//! `enter`, or types an own answer; `esc` stops the turn and cancels the question
//! (docs/zendo/activity.md: ACT-4, ACT-5, ACT-11).
use super::*;
use crate::agent::{Pick, Question};

/// A question of the agent in a thread, and what the user picked so far.
#[derive(Clone, Debug, Default)]
pub struct Ask {
    pub request: u64,
    pub questions: Vec<Question>,
    pub picks: Vec<Pick>,
    /// The question that the user answers now.
    pub at: usize,
    /// The choice under the cursor.
    pub cursor: usize,
    /// The answer went to the agent, or the user cancelled the question.
    pub done: bool,
}

impl Ask {
    pub fn new(request: u64, questions: Vec<Question>) -> Self {
        let picks = vec![Pick::default(); questions.len()];
        Ask { request, questions, picks, ..Default::default() }
    }

    /// The question that waits for the user.
    pub fn current(&self) -> Option<&Question> {
        self.questions.get(self.at).filter(|_| !self.done)
    }
}

impl Thread {
    /// The question of the agent that waits for the user in this thread.
    pub fn asking(&self) -> Option<&Ask> {
        self.items.iter().rev().find_map(|it| if let Item::Question(a) = it { Some(a.as_ref()) } else { None }).filter(|a| !a.done)
    }

    fn asking_mut(&mut self) -> Option<&mut Ask> {
        self.items.iter_mut().rev().find_map(|it| if let Item::Question(a) = it { Some(a.as_mut()) } else { None }).filter(|a| !a.done)
    }
}

impl App {
    /// The agent asks: the question goes into its thread, and the user who does not see the
    /// session hears the bell and sees the blink (ACT-4, ACT-11).
    pub(crate) fn on_question(&mut self, key: String, request: u64, questions: Vec<Question>) {
        self.thread(&key).push(Item::Question(Box::new(Ask::new(request, questions))));
        if let Some(n) = (0..SESSIONS).find(|n| session_key(*n) == key) {
            self.turn_ended(n);
        }
    }

    /// A key of the chat input while a question waits. True when the key went to the question.
    pub(crate) fn question_key(&mut self, k: &KeyEvent, act: Option<&str>) -> bool {
        let key = self.key();
        let empty = self.composer().input().is_empty();
        let Some(a) = self.thread(&key).asking_mut() else { return false };
        let n = a.current().map_or(0, |q| q.choices.len()).max(1);
        match (k.code, act) {
            (KeyCode::Up, _) if empty => a.cursor = (a.cursor + n - 1) % n,
            (KeyCode::Down, _) if empty => a.cursor = (a.cursor + 1) % n,
            (KeyCode::Char(' '), _) if empty && a.current().is_some_and(|q| q.many) => toggle(a),
            (_, Some("send" | "send_now")) => return self.answer_question(key),
            _ => return false,
        }
        true
    }

    /// `enter`: the own text of the input, or the choice under the cursor, answers the question;
    /// after the last one, the picks go to the agent.
    fn answer_question(&mut self, key: String) -> bool {
        let own = self.composer_mut().input_mut().take().trim().to_string();
        let Some(a) = self.thread(&key).asking_mut() else { return false };
        let Some(q) = a.current().cloned() else { return false };
        let pick = &mut a.picks[a.at];
        if !own.is_empty() {
            pick.own = own;
        } else if pick.values.is_empty() {
            pick.values.extend(q.choices.get(a.cursor).map(|(v, _)| v.clone()));
        }
        a.at += 1;
        a.cursor = 0;
        if a.at < a.questions.len() {
            return true;
        }
        a.done = true;
        let (request, picks) = (a.request, a.picks.clone());
        self.link.send(Cmd::Reply { request, picks: Some(picks) });
        true
    }

    /// The turn stops: an open question of it is cancelled.
    pub(crate) fn cancel_question(&mut self, key: &str) {
        if let Some(a) = self.threads.get_mut(key).and_then(Thread::asking_mut) {
            a.done = true;
            let request = a.request;
            self.link.send(Cmd::Reply { request, picks: None });
        }
    }
}

/// `space` on a question with many choices: the choice under the cursor goes in or out.
fn toggle(a: &mut Ask) {
    let Some(v) = a.current().and_then(|q| q.choices.get(a.cursor)).map(|(v, _)| v.clone()) else { return };
    let values = &mut a.picks[a.at].values;
    match values.iter().position(|x| *x == v) {
        Some(i) => drop(values.remove(i)),
        None => values.push(v),
    }
}
