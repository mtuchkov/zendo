//! The activity of the agent in the chat (docs/zendo/activity.md): the status line with the step
//! that runs, the time since the last news, and the failed steps.

mod common;
use common::*;
use std::time::{Duration, Instant};
use zendo::agent::{Event, ToolState};
use zendo::app::{App, Focus, session_key};

fn busy_chat(name: &str) -> (std::path::PathBuf, App) {
    let root = repo(name, &[("docs/a.md", "# A\n")]);
    let mut a = app(&root);
    a.open_panel();
    a.focus = Focus::Claude;
    let now = Instant::now();
    let t = a.thread(&session_key(0));
    t.busy = Some(now);
    t.news = Some(now);
    (root, a)
}

fn tool(id: &str, title: &str, kind: &str, state: ToolState, place: Option<&str>, error: Option<&str>) -> Event {
    Event::Tool {
        key: session_key(0),
        id: id.into(),
        title: title.into(),
        kind: kind.into(),
        state,
        place: place.map(Into::into),
        error: error.map(Into::into),
    }
}

fn screen(a: &mut App) -> String {
    draw(a, 400, 40).join("\n")
}

// ACT-1, ACT-7: the status line names the step that runs now from the tool call of the agent, and
// says `thinking` when no step runs.
#[tokio::test]
async fn the_status_line_names_the_step_that_runs() {
    let (root, mut a) = busy_chat("act-step");
    a.on_agent(tool("r1", "Read docs/a.md", "read", ToolState::Running, Some("docs/a.md"), None));
    let s = screen(&mut a);
    assert!(s.contains("reading docs/a.md · 0 s"), "{s}");

    a.on_agent(tool("r1", "", "", ToolState::Done, None, None));
    let s = screen(&mut a);
    assert!(!s.contains("reading docs/a.md ·"), "{s}");
    assert!(s.contains("thinking · 0 s"), "{s}");
    let _ = std::fs::remove_dir_all(&root);
}

// ACT-1: a step without a file names its tool title.
#[tokio::test]
async fn a_step_without_a_file_names_its_title() {
    let (root, mut a) = busy_chat("act-title");
    a.on_agent(tool("s1", "grep parse", "search", ToolState::Pending, None, None));
    let s = screen(&mut a);
    assert!(s.contains("searching grep parse · 0 s"), "{s}");
    let _ = std::fs::remove_dir_all(&root);
}

// ACT-2: after 1 min without news the status line says so and counts up; the next news removes it.
#[tokio::test]
async fn a_quiet_turn_shows_the_time_since_the_last_update() {
    let (root, mut a) = busy_chat("act-quiet");
    let then = Instant::now().checked_sub(Duration::from_secs(61)).unwrap();
    let t = a.thread(&session_key(0));
    t.busy = Some(then);
    t.news = Some(then);
    let s = screen(&mut a);
    assert!(s.contains("thinking · 61 s · no news for 1 min"), "{s}");

    let t = a.thread(&session_key(0));
    t.news = Some(Instant::now().checked_sub(Duration::from_secs(150)).unwrap());
    assert!(screen(&mut a).contains("no news for 2 min"));

    a.on_agent(Event::Text { key: session_key(0), text: "Still here.".into() });
    let s = screen(&mut a);
    assert!(!s.contains("no news"), "{s}");
    let _ = std::fs::remove_dir_all(&root);
}

// ACT-8: a failed step is marked, with the first line of its error.
#[tokio::test]
async fn a_failed_step_shows_its_error() {
    let (root, mut a) = busy_chat("act-failed");
    a.on_agent(tool("e1", "Edit docs/a.md", "edit", ToolState::Running, Some("docs/a.md"), None));
    a.on_agent(tool("e1", "", "", ToolState::Failed, None, Some("old_string not found\nmore details")));
    let s = screen(&mut a);
    assert!(s.contains("✗ Edit docs/a.md: old_string not found"), "{s}");
    assert!(!s.contains("more details"), "{s}");
    let _ = std::fs::remove_dir_all(&root);
}
