//! The steps of a turn in the chat: the status line with the step that runs, the line of a run of
//! tool calls, the failed steps, and the open steps with their input and output
//! (docs/zendo/activity.md: ACT-1, ACT-2, ACT-6, ACT-8; docs/zendo/interactivity.md: INTER-2).
use super::*;
use crate::agent::ToolState;

/// For each row of a step in the transcript: the row, the item of the thread, and the step in the
/// item; no step for the line of the whole run.
pub(crate) type Steps = Vec<(usize, usize, Option<usize>)>;

/// After this long without an event of the agent, the status line says so (ACT-2).
const NO_NEWS: std::time::Duration = std::time::Duration::from_secs(60);

/// The status line of a turn: the step that runs now, or `thinking`, the time of the turn, and the
/// time since the last news when it is long (ACT-1, ACT-2).
pub(super) fn status_line(th: &crate::app::Thread, start: std::time::Instant) -> String {
    let e = start.elapsed();
    let step = if th.asking().is_some() { "waits for your answer".to_string() } else { running_step(th).map_or_else(|| "thinking".to_string(), step_label) };
    let mut s = format!("{} {step} · {} s", SPIN[(e.as_millis() / 300) as usize % 4], e.as_secs());
    let quiet = th.news.unwrap_or(start).elapsed();
    if quiet >= NO_NEWS {
        s.push_str(&format!(" · no news for {} min", quiet.as_secs() / 60));
    }
    s
}

/// The last step of the turn that has not ended.
fn running_step(th: &crate::app::Thread) -> Option<&crate::app::ToolUse> {
    th.items
        .iter()
        .rev()
        .take_while(|it| !matches!(it, Item::User(_)))
        .filter_map(|it| match it {
            Item::Tools { uses, .. } => uses.iter().rev().find(|u| !u.state.ended()),
            _ => None,
        })
        .next()
}

/// What a step does, as the status line says it: `reading docs/a.md`.
fn step_label(u: &crate::app::ToolUse) -> String {
    let verb = match u.kind.as_str() {
        "read" => "reading",
        "edit" => "editing",
        "search" => "searching",
        "fetch" => "fetching",
        "execute" => "running",
        "think" => "thinking",
        "delete" => "deleting",
        "move" => "moving",
        _ => "using",
    };
    let what = u.info.place.clone().filter(|p| !p.is_empty()).unwrap_or_else(|| u.title.clone());
    format!("{verb} {what}")
}

/// The line of a run of tool calls: how many of each kind, and how long.
fn summary(uses: &[crate::app::ToolUse], took: std::time::Duration) -> String {
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
    s.join(" · ")
}

/// The line of one step: its mark, its title, its exit code, and the error of a failed step.
fn step_text(u: &crate::app::ToolUse) -> (&'static str, String) {
    let failed = u.state == ToolState::Failed;
    let mark = if u.open { "  ▾ " } else if failed { "  ✗ " } else { "  · " };
    let mut text = if u.title.is_empty() { u.info.place.clone().unwrap_or_default() } else { u.title.clone() };
    if let Some(code) = u.info.exit {
        text.push_str(&format!(" · exit {code}"));
    }
    if let Some(e) = u.info.error.as_ref().filter(|_| failed) {
        text.push_str(&format!(": {e}"));
    }
    (mark, text)
}

/// Under an open step: its input, then the last three lines of the output of a command, or the
/// first lines of another output, or its error (ACT-6, INTER-2).
fn step_body(u: &crate::app::ToolUse, dim: Style, w: usize) -> Vec<Line<'static>> {
    const FIRST: usize = 8;
    let shell = u.kind == "execute";
    let out = &u.info.output;
    let mut lines: Vec<String> = u.info.input.iter().map(|i| if shell { format!("$ {i}") } else { format!("in: {i}") }).collect();
    if shell {
        lines.extend(out.iter().skip(out.len().saturating_sub(3)).cloned());
    } else {
        lines.extend(out.iter().take(FIRST).cloned());
        lines.extend((out.len() > FIRST).then(|| format!("… {} more lines", out.len() - FIRST)));
    }
    if out.is_empty() {
        lines.extend(u.info.error.clone());
    }
    if lines.is_empty() {
        lines.push(if u.state.ended() { "(no output)" } else { "(no output yet)" }.to_string());
    }
    lines.into_iter().flat_map(|l| markdown::wrap(vec![Span::styled(l, dim)], w, "    │ ", "    │ ")).collect()
}

/// The lines of a run of tool calls, and the rows of its steps. A failed step shows always, with
/// the first line of its error (ACT-8); each step shows with the details or when it is open, and an
/// open step shows its body (INTER-2).
pub(super) fn tool_lines(app: &App, uses: &[crate::app::ToolUse], took: std::time::Duration, dim: Style, w: usize) -> (Vec<Line<'static>>, Vec<(usize, Option<usize>)>) {
    let mut out = vec![Line::styled(summary(uses, took), dim)];
    let mut rows = vec![(0, None)];
    for (j, u) in uses.iter().enumerate() {
        if !(app.threads.details() || u.open || u.state == ToolState::Failed) {
            continue;
        }
        rows.push((out.len(), Some(j)));
        let (mark, text) = step_text(u);
        out.extend(markdown::wrap(vec![Span::styled(text, dim)], w, mark, "    "));
        if u.open {
            out.extend(step_body(u, dim, w));
        }
    }
    (out, rows)
}
