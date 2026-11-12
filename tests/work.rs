//! Visits, delta and recaps through the app, with the mock agent and no terminal.

use zendo::app::{App, Config, ENGAGED, Mode};
use zendo::docs::delta;
use std::time::{Duration, Instant};

fn repo(name: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("zendo-work-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(d.join("docs/notes")).unwrap();
    std::fs::write(d.join("docs/notes/2026-11-11-log.md"), "# Event log\n\n- a\n").unwrap();
    d
}

fn app(root: &std::path::Path) -> App {
    App::new(Config {
        root: root.to_path_buf(),
        notes_dir: root.join("docs/notes"),
        agent: env!("CARGO_BIN_EXE_zendo-mock-agent").to_string(),
        agent_env: Default::default(),
        offline: false,
        notes_repo: None,
    })
    .unwrap()
}

/// Feed agent events and ticks until `done` holds.
async fn until(app: &mut App, done: impl Fn(&App) -> bool) {
    let end = Instant::now() + Duration::from_secs(20);
    while !done(app) {
        assert!(Instant::now() < end, "timeout");
        app.tick();
        if let Some(rx) = app.agent_events.as_mut() {
            if let Ok(Some(e)) = tokio::time::timeout(Duration::from_millis(50), rx.recv()).await {
                app.on_agent(e);
            }
        } else {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }
}

const REL: &str = "docs/notes/2026-11-11-log.md";

#[tokio::test]
async fn brief_open_does_not_count_but_work_does() {
    let root = repo("visit");
    let mut a = app(&root);
    a.sync_visit();
    a.interact(3);
    a.mode = Mode::Notes;
    a.sync_visit();
    assert!(!a.state.files.contains_key(REL), "a brief open must not count");

    a.mode = Mode::Read;
    a.interact(ENGAGED);
    a.mode = Mode::Notes;
    a.sync_visit();
    let f = &a.state.files[REL];
    assert_eq!(f.visits, 1);
    assert!(f.seen.as_ref().unwrap().text.contains("- a"));
    // The database is on disk.
    assert!(std::fs::read_to_string(root.join(".zendo/db.json")).unwrap().contains(REL));

    // A change after the visit shows in the delta since the last visit.
    std::fs::write(root.join(REL), "# Event log\n\n- a\n- b\n").unwrap();
    a.reload();
    a.mode = Mode::Read;
    let (label, old) = a.delta_base();
    assert!(label.starts_with("since your last visit"), "{label}");
    let d = delta::compute(&old, &a.note_text().unwrap(), 2);
    assert_eq!((d.added, d.removed), (1, 0));
}

#[tokio::test]
async fn recaps_on_request_and_when_idle() {
    let root = repo("recap");
    let mut a = app(&root);
    a.ensure_agent();
    until(&mut a, |a| a.agent_name.is_some()).await;

    // On request.
    a.recap_now();
    until(&mut a, |a| a.state.files.get(REL).and_then(|f| f.recap.as_ref()).is_some()).await;
    let r = a.state.files[REL].recap.clone().unwrap();
    assert!(!r.auto);
    assert!(r.text.contains("Decides: Event log"), "{}", r.text);
    // Recaps never show up as chat threads.
    assert!(a.threads.values().all(|t| t.items.is_empty()));

    // Work on the note, change it, then go idle: Claude writes a new recap by itself.
    a.interact(ENGAGED);
    std::fs::write(root.join(REL), "# Event log\n\n- a\n- WHEN x, the log shall y.\n").unwrap();
    a.reload();
    a.last_input = Instant::now() - Duration::from_secs(120);
    until(&mut a, |a| a.state.files[REL].recap.as_ref().is_some_and(|r| r.auto)).await;
    let r = a.state.files[REL].recap.clone().unwrap();
    assert!(r.text.contains("Changed since the last recap"), "{}", r.text);

    // Nothing changed: no new automatic recap.
    let at = r.at;
    for _ in 0..10 {
        a.tick();
        tokio::time::sleep(Duration::from_millis(30)).await;
    }
    assert!(a.recap_job.is_none());
    assert_eq!(a.state.files[REL].recap.as_ref().unwrap().at, at);
}

#[tokio::test]
async fn no_auto_recap_while_the_user_is_active() {
    let root = repo("active");
    let mut a = app(&root);
    a.ensure_agent();
    until(&mut a, |a| a.agent_name.is_some()).await;
    a.interact(ENGAGED);
    a.last_input = Instant::now();
    for _ in 0..10 {
        a.tick();
        tokio::time::sleep(Duration::from_millis(30)).await;
    }
    assert!(a.recap_job.is_none());
}
