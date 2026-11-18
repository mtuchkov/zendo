//! A question of the agent in the chat: its text, its choices with the cursor, and the hint; after
//! the answer, what the user picked (docs/zendo/activity.md: ACT-4).
use super::*;
use crate::app::Ask;

/// The answer to question `i` as a line says it.
fn answer(a: &Ask, i: usize) -> String {
    let (q, p) = (&a.questions[i], &a.picks[i]);
    let titles = p.values.iter().filter_map(|v| q.choices.iter().find(|(c, _)| c == v).map(|(_, t)| t.clone()));
    let all: Vec<String> = titles.chain((!p.own.is_empty()).then(|| p.own.clone())).collect();
    if all.is_empty() { "no answer".to_string() } else { all.join(", ") }
}

pub(super) fn lines(a: &Ask, t: &Theme, w: usize) -> Vec<Line<'static>> {
    let (dim, ask) = (Style::new().fg(t.dim), Style::new().fg(t.acc).add_modifier(Modifier::BOLD));
    let mut out = Vec::new();
    for (i, q) in a.questions.iter().enumerate() {
        out.extend(markdown::wrap(vec![Span::styled(q.text.clone(), ask)], w, "? ", "  "));
        if i < a.at || a.done {
            out.extend(markdown::wrap(vec![Span::styled(format!("→ {}", answer(a, i)), dim)], w, "  ", "    "));
            continue;
        }
        if i > a.at {
            break;
        }
        for (j, (v, title)) in q.choices.iter().enumerate() {
            let on = a.picks[i].values.contains(v);
            let mark = match (q.many, on) {
                (true, true) => "[x]",
                (true, false) => "[ ]",
                (false, _) if j == a.cursor => "●",
                _ => "○",
            };
            let lead = if j == a.cursor { "  › " } else { "    " };
            let style = if j == a.cursor { Style::new().fg(t.fg) } else { dim };
            out.extend(markdown::wrap(vec![Span::styled(format!("{mark} {title}"), style)], w, lead, "      "));
        }
        let keys = if q.many { "↑ ↓ and space pick, enter sends" } else { "↑ ↓ pick, enter sends" };
        out.push(Line::styled(format!("  {keys}; or type your own answer"), dim.add_modifier(Modifier::ITALIC)));
    }
    out
}
