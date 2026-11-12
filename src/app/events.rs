//! Events from the agent.
use super::*;

impl App {
    pub fn on_agent(&mut self, e: Event) {
        // Recaps run in throwaway sessions and never reach the chat.
        if event_key(&e).as_deref().is_some_and(|k| k.starts_with(RECAP_PREFIX)) {
            return self.on_recap_event(e);
        }
        match e {
            Event::Ready { agent, can_image, .. } => {
                self.agent_name = Some(agent);
                self.agent_images = can_image;
            }
            Event::Session { key, id, resumed } => self.on_session(key, id, resumed),
            Event::Replay { key, user, text } => self.on_replay(key, user, text),
            Event::UserText { key, text } => {
                let t = self.thread(&key);
                if t.busy.is_none() || matches!(t.items.last(), Some(Item::Info(_))) {
                    t.items.push(Item::User(text));
                }
            }
            Event::Text { key, text } => {
                let t = self.thread(&key);
                t.thinking = false;
                match t.items.last_mut() {
                    Some(Item::Agent(s)) => s.push_str(&text),
                    _ => t.items.push(Item::Agent(text.trim_start().to_string())),
                }
            }
            Event::Thought { key } => self.thread(&key).thinking = true,
            Event::Tool { key, id, title, kind, done } => self.on_tool(key, ToolUse { id, title, kind }, done),
            Event::Permission { key, request, title, kind: _, diffs, choices } => {
                let t = self.thread(&key);
                t.items.push(Item::Proposal { request, title, diffs, choices, state: Answered::Pending });
                self.panel_scroll = 0;
            }
            Event::Blocked { key, title } => self.thread(&key).items.push(Item::Blocked(title)),
            Event::TurnEnd { key, stop } => self.on_turn_end(key, stop),
            Event::Error { key, text } => {
                let key = key.unwrap_or_else(|| self.key());
                let text = if text.contains("Authentication required") {
                    "Claude is not logged in. Run `claude` once in a terminal and log in, then ask again.".to_string()
                } else {
                    text
                };
                self.thread(&key).items.push(Item::Error(text));
            }
            Event::Exited { text } => self.on_exited(text),
            Event::Commands { list } => {
                self.slash.commands = list;
                self.update_slash();
            }
        }
    }

    fn on_recap_event(&mut self, e: Event) {
        match e {
            Event::Text { text, .. } => {
                if let Some(j) = self.recap_job.as_mut() {
                    j.buf.push_str(&text);
                }
            }
            Event::Permission { request, .. } => {
                if let Some(a) = &self.agent {
                    a.send(Cmd::Answer { request, option: None });
                }
            }
            Event::Error { text, .. } => {
                if let Some(j) = self.recap_job.as_mut() {
                    j.failed = Some(text);
                }
            }
            Event::TurnEnd { key, .. } => self.recap_done(key),
            _ => {}
        }
    }

    fn on_session(&mut self, key: String, id: String, resumed: bool) {
        self.state.set_session(&self.cfg.agent, &key, id.clone());
        self.state.save(&self.cfg.root);
        let t = self.thread(&key);
        t.session = Some(id);
        if resumed {
            t.items.insert(0, Item::Info("resumed the earlier thread".into()));
        }
    }

    fn on_replay(&mut self, key: String, user: bool, text: String) {
        let t = self.thread(&key);
        // Earlier messages go before the question that is waiting now, if there is one.
        let waiting = t.busy.and_then(|_| t.items.iter().rposition(|i| matches!(i, Item::User(_))));
        let at = waiting.unwrap_or(t.items.len());
        let prev = at.checked_sub(1).and_then(|i| t.items.get_mut(i));
        match (prev, user) {
            (Some(Item::Agent(s)), false) => s.push_str(&text),
            (Some(Item::User(s)), true) if at > 0 => s.push_str(&text),
            _ => t.items.insert(at, if user { Item::User(text) } else { Item::Agent(text) }),
        }
    }

    fn on_tool(&mut self, key: String, new: ToolUse, done: bool) {
        let t = self.thread(&key);
        let known = t.items.iter_mut().rev().find_map(|it| match it {
            Item::Tools { uses, .. } => uses.iter_mut().find(|u| u.id == new.id),
            _ => None,
        });
        match known {
            Some(u) => {
                if !new.title.is_empty() {
                    u.title = new.title;
                }
                if !new.kind.is_empty() {
                    u.kind = new.kind;
                }
            }
            // An update for a call we never saw (e.g. the edit behind a proposal).
            None if new.title.is_empty() => {}
            None => match t.items.last_mut() {
                Some(Item::Tools { uses, .. }) => uses.push(new),
                _ => t.items.push(Item::Tools { uses: vec![new], started: Instant::now(), took: None }),
            },
        }
        if done {
            self.reload_after_agent();
        }
    }

    fn on_turn_end(&mut self, key: String, stop: String) {
        if self.say.as_ref().is_some_and(|s| s.key == key) {
            self.say = None;
        }
        let t = self.thread(&key);
        t.busy = None;
        t.thinking = false;
        for it in t.items.iter_mut().rev() {
            if let Item::Tools { took: took @ None, started: s, .. } = it {
                *took = Some(s.elapsed());
            }
        }
        if stop == "cancelled" {
            t.items.push(Item::Info("stopped".into()));
        }
        self.reload_after_agent();
    }

    fn on_exited(&mut self, text: String) {
        self.recap_job = None;
        self.agent = None;
        self.agent_events = None;
        for t in self.threads.values_mut() {
            t.busy = None;
            t.primed = false;
            t.opened = false;
        }
        self.agent_down = Some(text);
    }
}

/// The thread an event belongs to.
fn event_key(e: &Event) -> Option<String> {
    match e {
        Event::Session { key, .. }
        | Event::UserText { key, .. }
        | Event::Replay { key, .. }
        | Event::Text { key, .. }
        | Event::Thought { key }
        | Event::Tool { key, .. }
        | Event::Permission { key, .. }
        | Event::Blocked { key, .. }
        | Event::TurnEnd { key, .. } => Some(key.clone()),
        Event::Error { key, .. } => key.clone(),
        _ => None,
    }
}
