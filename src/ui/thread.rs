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

/// The shell blocks of a thread: their first row, the row after them, their command.
pub(crate) type Blocks = Vec<(usize, usize, String)>;

pub(crate) fn panel_lines(app: &App, t: &Theme, w: usize) -> (Vec<Line<'static>>, Blocks) {
    let key = app.key();
    let dim = Style::new().fg(t.dim);
    let Some(th) = app.threads.get(&key).filter(|th| !th.items.is_empty() || th.busy.is_some()) else {
        let mut out = empty_thread(app, &app.scope(), dim, w);
        out.extend(cli_line(app, t));
        return (out, Vec::new());
    };
    let mut out: Vec<Line<'static>> = Vec::new();
    let mut blocks = Blocks::new();
    for it in &th.items {
        match it {
            Item::User(s) => out.extend(user_lines(question_of(s), t, w)),
            Item::Agent(s) => {
                let (lines, b) = agent_blocks(s, t, w);
                let base = out.len();
                blocks.extend(b.into_iter().map(|(a, z, c)| (a + base, z + base, c)));
                out.extend(lines);
            }
            Item::Tools { uses, started, took } => out.extend(tool_lines(app, uses, took.unwrap_or_else(|| started.elapsed()), dim, w)),
            Item::Blocked(s) => out.extend(markdown::wrap(vec![Span::styled(format!("blocked a shell command: {s}"), dim)], w, "", "")),
            Item::Info(s) => out.push(Line::styled(s.clone(), dim.add_modifier(Modifier::ITALIC))),
            Item::Error(s) => out.extend(markdown::wrap(vec![Span::styled(s.clone(), Style::new().fg(t.del))], w, "", "")),
        }
        out.push(Line::raw(""));
    }
    if let Some(start) = th.busy {
        let e = start.elapsed();
        let word = if th.thinking { "thinking" } else { "working" };
        out.push(Line::styled(format!("{word} {} {} s", SPIN[(e.as_millis() / 300) as usize % 4], e.as_secs()), dim));
    }
    for q in &th.queue {
        out.extend(markdown::wrap(vec![Span::styled(format!("queued › {}", question_of(&q.question)), dim)], w, "", "         "));
    }
    out.extend(cli_line(app, t));
    (out, blocks)
}

/// The play button of a shell block: green, at the right end of its first row (TERM-8).
pub(crate) const PLAY: &str = "▶";

/// The play button at the right end of `l`: the padding of a code block gives way to it.
fn put_play(l: &mut Line<'static>, t: &Theme, w: usize) {
    let used = |l: &Line| l.spans.iter().map(|s| s.content.width()).sum::<usize>();
    while used(l) + 2 > w {
        let Some(last) = l.spans.last_mut() else { break };
        let text = last.content.to_string();
        let Some(cut) = text.strip_suffix(' ') else { break };
        last.content = cut.to_string().into();
    }
    let pad = w.saturating_sub(used(l) + 2);
    l.spans.push(Span::styled(" ".repeat(pad), l.spans.last().map(|s| s.style).unwrap_or_default()));
    l.spans.push(Span::styled(PLAY, Style::new().fg(t.add).add_modifier(Modifier::BOLD)));
}

/// A message of the agent with a play button on each shell block, and the rows of the blocks.
fn agent_blocks(s: &str, t: &Theme, w: usize) -> (Vec<Line<'static>>, Blocks) {
    let on_panel = Theme { panel: t.bg, ..*t };
    let src = s.trim_end();
    let r = markdown::render(&markdown::plain(src), w, &on_panel, false);
    let mut lines = r.lines;
    let n = lines.len();
    let row = |i: usize| r.rows.get(i).copied().unwrap_or(n);
    let mut blocks = Blocks::new();
    for b in crate::text::shell::blocks(src) {
        let (first, end) = (row(b.fence), row(b.close).max(row(b.fence) + 1));
        if let Some(l) = lines.get_mut(first) {
            put_play(l, t, w);
        }
        blocks.push((first, end.min(n), b.command));
    }
    (lines, blocks)
}

/// The state of the Claude Code session of the agent session in the chat (SESSIONS-20).
fn cli_line(app: &App, t: &Theme) -> Option<Line<'static>> {
    use crate::session::State;
    let state = app.cli.state(app.state.session.min(crate::app::SESSIONS - 1))?;
    let key = app.keys.first(crate::keys::Pane::Chat, "handoff");
    let spin = SPIN[(app.cli.since().as_millis() / 300) as usize % 4];
    let (text, color) = match state {
        State::Works => (format!("Claude Code works {spin}"), t.dim),
        State::Waits => ("Claude Code waits for you".to_string(), t.acc),
        State::Idle => ("Claude Code is idle".to_string(), t.dim),
    };
    Some(Line::from(vec![Span::styled(text, Style::new().fg(color)), Span::styled(format!(" · {key} shows it"), Style::new().fg(t.dim))]))
}

/// The question in a message that zendo sent: a replayed message has the context before it.
pub fn question_of(s: &str) -> &str {
    s.rsplit_once("\nQuestion: ").map(|(_, q)| q).or_else(|| s.strip_prefix("Question: ")).unwrap_or(s)
}

/// A message of the user: a block of the selection color, with `›` in the accent color.
pub fn user_lines(s: &str, t: &Theme, w: usize) -> Vec<Line<'static>> {
    let spans = chip_spans(s, Style::new().fg(t.fg), chip_styles(t));
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
        format!("{who} reads {reads}, and edits files without a question."),
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

/// Text with its chips shown as chips: `⟦label⟧` shows as ` label ` on the chip color.
pub fn chip_spans(text: &str, base: Style, chip: [Style; 2]) -> Vec<Span<'static>> {
    chip_spans_at(text, 0, base, chip, None)
}

/// The styles of a chip: a selection or a file, and text from the chat.
pub fn chip_styles(t: &Theme) -> [Style; 2] {
    [Style::new().fg(t.bg).bg(t.acc2), Style::new().fg(t.bg).bg(t.add)]
}

/// The same for a piece of a line that starts at char `offset`; the chip at `hot` gets the accent.
pub fn chip_spans_at(text: &str, offset: usize, base: Style, chip: [Style; 2], hot: Option<(usize, usize, Style)>) -> Vec<Span<'static>> {
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
            _ if label.starts_with(crate::app::CHAT_MARK) => chip[1],
            _ => chip[0],
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
