//! The lines of a chat thread: messages, tool uses, proposals.
use super::*;

pub(crate) fn kind_verb(kind: &str) -> &str {
    match kind {
        "read" => "read",
        "edit" => "edit",
        "search" => "search",
        "fetch" => "fetch",
        "execute" => "shell",
        "think" => "think",
        "delete" => "delete",
        "move" => "move",
        _ => "tool",
    }
}

pub(crate) fn panel_lines(app: &App, t: &Theme, w: usize) -> Vec<Line<'static>> {
    let key = app.key();
    let dim = Style::new().fg(t.dim);
    let Some(th) = app.threads.get(&key).filter(|th| !th.items.is_empty() || th.busy.is_some()) else {
        return empty_thread(app, &key, dim, w);
    };
    let mut out: Vec<Line<'static>> = Vec::new();
    for it in &th.items {
        match it {
            Item::User(s) => out.extend(user_lines(s, t, w)),
            Item::Agent(s) => out.extend(agent_lines(s, t, w)),
            Item::Tools { uses, started, took } => out.extend(tool_lines(app, uses, took.unwrap_or_else(|| started.elapsed()), dim, w)),
            Item::Proposal { title, diffs, choices, state, .. } => out.extend(proposal_lines(title, diffs, choices, state, t, w)),
            Item::Blocked(s) => out.extend(markdown::wrap(vec![Span::styled(format!("blocked a shell command: {s}"), dim)], w, "", "")),
            Item::Info(s) => out.push(Line::styled(s.clone(), dim.add_modifier(Modifier::ITALIC))),
            Item::Error(s) => out.extend(markdown::wrap(vec![Span::styled(s.clone(), Style::new().fg(t.del))], w, "", "")),
        }
        out.push(Line::raw(""));
    }
    if let Some(start) = th.busy.filter(|_| th.pending().is_none()) {
        let e = start.elapsed();
        let word = if th.thinking { "thinking" } else { "working" };
        out.push(Line::styled(format!("{word} {} {} s", SPIN[(e.as_millis() / 300) as usize % 4], e.as_secs()), dim));
    }
    out
}

/// A message of the user: a block of the selection color, with `›` in the accent color.
pub fn user_lines(s: &str, t: &Theme, w: usize) -> Vec<Line<'static>> {
    let spans = chip_spans(s, Style::new().fg(t.fg), Style::new().fg(t.bg).bg(t.acc2));
    let mut lines = markdown::wrap(spans, w.saturating_sub(1), " › ", "   ");
    if let Some(first) = lines.first_mut().and_then(|l| l.spans.first_mut()) {
        first.style = Style::new().fg(t.acc).add_modifier(Modifier::BOLD);
    }
    lines.into_iter().map(|l| tint(l, t.sel, w)).collect()
}

/// A message of the agent: markdown on the panel. Inline code and code blocks take the color of
/// the note, so they stand out from the panel.
pub fn agent_lines(s: &str, t: &Theme, w: usize) -> Vec<Line<'static>> {
    let on_panel = Theme { panel: t.bg, ..*t };
    markdown::render(&markdown::plain(s.trim_end()), w, &on_panel, false).lines
}

/// What the chat says before the first question.
fn empty_thread(app: &App, key: &str, dim: Style, w: usize) -> Vec<Line<'static>> {
    let word = app.file_word();
    let what = if key == ALL_KEY { "all notes".to_string() } else { format!("this {word}") };
    let who = app.profile().map_or("The agent", |p| p.name);
    let reads = if key == ALL_KEY { "the notes".to_string() } else { format!("the {word} and the notes it links to") };
    let mut out = Vec::new();
    for s in [
        format!("Ask about {what}."),
        String::new(),
        format!("{who} reads {reads}. An edit comes back as a proposal: nothing is written until you press a."),
        String::new(),
        "Shell commands are blocked.".to_string(),
    ] {
        out.extend(markdown::wrap(vec![Span::styled(s, dim)], w, "", ""));
    }
    out
}

/// One line for a run of tool uses: how many of each kind, and how long; with details, each title.
fn tool_lines(app: &App, uses: &[crate::app::ToolUse], took: std::time::Duration, dim: Style, w: usize) -> Vec<Line<'static>> {
    let mut counts: Vec<(String, usize)> = Vec::new();
    for u in uses {
        let v = kind_verb(&u.kind).to_string();
        match counts.iter_mut().find(|(k, _)| *k == v) {
            Some((_, n)) => *n += 1,
            None => counts.push((v, 1)),
        }
    }
    let mut s: Vec<String> = counts.iter().map(|(k, n)| if *n == 1 { k.clone() } else { format!("{k} {n}") }).collect();
    s.push(format!("{:.0} s", took.as_secs_f32()));
    let mut out = vec![Line::styled(s.join(" · "), dim)];
    if app.details {
        for u in uses {
            out.extend(markdown::wrap(vec![Span::styled(u.title.clone(), dim)], w, "  · ", "    "));
        }
    }
    out
}

/// A proposal: its title, the diffs, and the keys or the answer.
fn proposal_lines(title: &str, diffs: &[crate::agent::DiffView], choices: &[crate::agent::Choice], state: &Answered, t: &Theme, w: usize) -> Vec<Line<'static>> {
    let dim = Style::new().fg(t.dim);
    let mut out = markdown::wrap(vec![Span::styled(title.to_string(), dim)], w, "", "");
    for d in diffs {
        out.extend(diff_box(d.old.as_deref().unwrap_or(""), &d.new, w, t));
    }
    if diffs.is_empty() {
        out.push(Line::styled("Claude asks to run this tool.", Style::new().fg(t.fg)));
    }
    let key = |k: &str, label: &str| vec![Span::styled(k.to_string(), Style::new().fg(t.fg).add_modifier(Modifier::BOLD)), Span::styled(format!(" {label}   "), dim)];
    match state {
        Answered::Pending => {
            let mut spans = key("a", if diffs.is_empty() { "allow" } else { "apply" });
            if choices.iter().any(|c| c.allow && c.always) {
                spans.extend(key("A", "always"));
            }
            if !diffs.is_empty() {
                spans.extend(key("e", "edit first"));
            }
            spans.extend(key("x", "discard"));
            out.push(Line::from(spans));
        }
        Answered::Applied => out.push(Line::styled("applied", Style::new().fg(t.add))),
        Answered::Discarded => out.push(Line::styled("discarded", dim)),
        Answered::EditFirst => out.push(Line::styled("you are editing it", Style::new().fg(t.acc2))),
    }
    out
}

/// Changed lines with one line of context, in a box.
pub(crate) fn diff_box(old: &str, new: &str, w: usize, t: &Theme) -> Vec<Line<'static>> {
    let d = markdown::diff_lines(old, new);
    let changed: Vec<bool> = d.iter().map(|(_, m)| *m != Mark::None).collect();
    let near = |i: usize| (i.saturating_sub(1)..=(i + 1).min(d.len().saturating_sub(1))).any(|j| changed.get(j) == Some(&true));
    let fill = Style::new().bg(t.bg);
    let mut out = Vec::new();
    let mut gap = false;
    for (i, (text, mark)) in d.iter().enumerate() {
        if !near(i) {
            gap = true;
            continue;
        }
        if gap && !out.is_empty() {
            out.push(Line::styled(" …", Style::new().fg(t.dim).bg(t.bg)));
        }
        gap = false;
        // Show list dashes as bullets, so they do not read as removed lines.
        let text = match text.trim_start().strip_prefix("- ") {
            Some(rest) => format!("{}• {rest}", &text[..text.len() - text.trim_start().len()]),
            None => text.clone(),
        };
        let (p, style) = match mark {
            Mark::Added => ("+ ", Style::new().fg(t.add)),
            Mark::Removed => ("- ", Style::new().fg(t.del)),
            Mark::None => ("  ", Style::new().fg(t.dim)),
        };
        for l in markdown::wrap(vec![Span::styled(text, style)], w.saturating_sub(2), &format!(" {p}"), "   ") {
            let mut l = l.style(fill);
            if let Some(first) = l.spans.first_mut() {
                first.style = style;
            }
            let used: usize = l.spans.iter().map(|s| s.content.width()).sum();
            l.spans.push(Span::raw(" ".repeat(w.saturating_sub(used))));
            out.push(l);
        }
    }
    out
}

/// Text with its chips shown as chips: `⟦label⟧` shows as ` label ` on the chip color.
pub fn chip_spans(text: &str, base: Style, chip: Style) -> Vec<Span<'static>> {
    chip_spans_at(text, 0, base, chip, None)
}

/// The same for a piece of a line that starts at char `offset`; the chip at `hot` gets the accent.
pub fn chip_spans_at(text: &str, offset: usize, base: Style, chip: Style, hot: Option<(usize, usize, Style)>) -> Vec<Span<'static>> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut at = 0;
    for (a, b) in crate::app::chips::chip_ranges(text) {
        if a > at {
            out.push(Span::styled(chars[at..a].iter().collect::<String>(), base));
        }
        let label: String = chars[a + 1..b - 1].iter().collect();
        let style = match hot {
            Some((ha, hb, s)) if ha == a + offset && hb == b + offset => s,
            _ => chip,
        };
        out.push(Span::styled(format!(" {label} "), style));
        at = b;
    }
    // A chip cut by the wrap: its marks show as spaces.
    let rest: String = chars[at..].iter().map(|c| if *c == crate::app::chips::OPEN || *c == crate::app::chips::CLOSE { ' ' } else { *c }).collect();
    if !rest.is_empty() {
        out.push(Span::styled(rest, base));
    }
    out
}
