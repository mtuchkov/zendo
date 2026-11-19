//! The claim of a delivery and the text of a message for the agent (BOARD-1, BOARD-8, BOARD-11).

use super::{Event, How, Journal, Msg, owner_of};

impl Journal {
    /// Claim the delivery of `id` to session `r`. True when this claim is the first of the pair;
    /// a named channel of the sender in `reply_to` takes `r` in (BOARD-8).
    pub fn claim(&self, id: &str, r: usize, how: How) -> bool {
        let r = self.transact(|st| {
            if st.claimed(id, r).is_some() {
                return Err(());
            }
            let mut ev = vec![Event::Claim { msg: id.to_string(), to: r, how }];
            if let Some(m) = st.msg(id)
                && let Some(ch) = m.reply_to.as_ref().filter(|c| owner_of(c) == Some(m.from))
            {
                ev.push(Event::Invite { repo: m.repo.clone(), channel: ch.clone(), session: r });
            }
            Ok(ev)
        });
        matches!(r, Ok(Ok(_)))
    }

    /// The turn of session `r` that took the initiations `ids` ended (BOARD-9).
    pub fn land(&self, ids: &[String], r: usize) {
        let ev: Vec<Event> = ids.iter().map(|id| Event::Landed { msg: id.clone(), to: r }).collect();
        if !ev.is_empty() {
            let _ = self.append(&ev);
        }
    }
}

/// The message as the agent gets it: in a tag with the sender, the id, the kind and where its
/// responses go (BOARD-11).
pub fn tagged(m: &Msg) -> String {
    let mut attrs = format!("from=\"{}\" id=\"{}\" kind=\"{}\"", m.from, m.id, m.kind.name());
    if let Some(s) = m.status {
        attrs.push_str(&format!(" status=\"{}\"", s.name()));
    }
    if let Some(r) = &m.in_reply_to {
        attrs.push_str(&format!(" in_reply_to=\"{r}\""));
    }
    if m.kind.initiation() {
        attrs.push_str(&format!(" reply_to=\"{}\"", m.responses_to()));
    }
    if let Some(to) = &m.to {
        attrs.push_str(&format!(" to=\"{to}\""));
    }
    format!("<zendo_message {attrs}>\n{}\n</zendo_message>", m.text)
}

/// The message as the chat shows it: `from 3 · <name>` and the text (BOARD-11).
pub fn label(m: &Msg, name: &str) -> String {
    let who = if name.is_empty() { format!("from {}", m.from) } else { format!("from {} · {name}", m.from) };
    let kind = match m.status {
        Some(s) => format!("{} {}", m.kind.name(), s.name()),
        None => m.kind.name().to_string(),
    };
    format!("{who} · {kind}: {}", m.text)
}
