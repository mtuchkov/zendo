//! The ACP client against the mock agent: prompt, stream, permission, resume.

use zendo::agent::{self, Cmd, Event};
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedReceiver;

fn tmpdir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("zendo-test-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

async fn next(rx: &mut UnboundedReceiver<Event>) -> Event {
    tokio::time::timeout(Duration::from_secs(20), rx.recv()).await.expect("timeout").expect("closed")
}

fn mock() -> String {
    env!("CARGO_BIN_EXE_zendo-mock-agent").to_string()
}

#[tokio::test]
async fn edit_is_proposed_and_applied_only_after_allow() {
    let dir = tmpdir("edit");
    let note = dir.join("2026-11-11-x.md");
    std::fs::write(&note, "# X\n\n- a\n").unwrap();
    let (h, mut rx) = agent::spawn(mock(), Default::default(), dir.clone(), None);
    assert!(matches!(next(&mut rx).await, Event::Ready { can_load: true, .. }));

    let text = format!("Note file: {}\nQuestion: what is missing?", note.display());
    h.send(Cmd::Prompt { key: "x".into(), known: None, text, attach: vec![] });

    let mut streamed = String::new();
    let mut session = None;
    loop {
        match next(&mut rx).await {
            Event::Session { id, resumed, .. } => {
                assert!(!resumed);
                session = Some(id);
            }
            Event::Text { text, .. } => streamed.push_str(&text),
            Event::Tool { .. } => {}
            Event::Permission { request, diffs, choices, key, .. } => {
                assert_eq!(key, "x");
                assert_eq!(diffs.len(), 1);
                assert!(diffs[0].new.contains("- WHEN the mock agent"));
                // Nothing is written before the answer.
                assert_eq!(std::fs::read_to_string(&note).unwrap(), "# X\n\n- a\n");
                let allow = choices.iter().find(|c| c.allow).unwrap().id.clone();
                h.send(Cmd::Answer { request, option: Some(allow) });
            }
            Event::TurnEnd { stop, .. } => {
                assert_eq!(stop, "endturn");
                break;
            }
            Event::Commands { list } => assert!(list.iter().any(|c| c.name == "review"), "the slash commands of the agent: {list:?}"),
            e => panic!("unexpected {e:?}"),
        }
    }
    assert!(session.is_some());
    assert!(streamed.contains("You asked: \"what is missing?\""), "{streamed}");
    assert!(std::fs::read_to_string(&note).unwrap().contains("- WHEN the mock agent"));
}

#[tokio::test]
async fn shell_is_blocked_and_resume_replays() {
    let dir = tmpdir("shell");
    let (h, mut rx) = agent::spawn(mock(), Default::default(), dir.clone(), None);
    assert!(matches!(next(&mut rx).await, Event::Ready { .. }));

    h.send(Cmd::Prompt { key: "n".into(), known: Some("mock-old".into()), text: "Question: run a shell command".into(), attach: vec![] });
    let mut blocked = false;
    let mut replayed = false;
    loop {
        match next(&mut rx).await {
            Event::Replay { key, user: true, .. } => {
                assert_eq!(key, "n");
                replayed = true;
            }
            Event::Session { resumed, id, .. } => {
                assert!(resumed);
                assert_eq!(id, "mock-old");
            }
            Event::Blocked { title, .. } => {
                assert!(title.contains("git log"));
                blocked = true;
            }
            Event::TurnEnd { .. } => break,
            Event::Permission { .. } => panic!("shell must not reach the user"),
            _ => {}
        }
    }
    assert!(blocked && replayed);
}

#[tokio::test]
async fn images_and_files_go_with_the_prompt() {
    let dir = tmpdir("attach");
    let (png, md) = (dir.join("shot.png"), dir.join("spec.md"));
    std::fs::write(&png, [0x89, b'P', b'N', b'G']).unwrap();
    std::fs::write(&md, "# Spec\n").unwrap();
    let (h, mut rx) = agent::spawn(mock(), Default::default(), dir.clone(), None);
    assert!(matches!(next(&mut rx).await, Event::Ready { can_image: true, .. }), "the mock agent takes images");
    let attach = vec![agent::Attachment::of(png.clone()), agent::Attachment::of(md.clone())];
    assert!(matches!(attach[0], agent::Attachment::Image(_)) && matches!(attach[1], agent::Attachment::File(_)));
    h.send(Cmd::Prompt { key: "a".into(), known: None, text: "Question: [image shot.png] [file spec.md] what is this?".into(), attach });
    let mut streamed = String::new();
    loop {
        match next(&mut rx).await {
            Event::Text { text, .. } => streamed.push_str(&text),
            Event::Permission { request, .. } => h.send(Cmd::Answer { request, option: None }),
            Event::TurnEnd { .. } => break,
            _ => {}
        }
    }
    assert!(streamed.contains("an image (image/png"), "{streamed}");
    assert!(streamed.contains(&format!("a file link (file://{})", md.display())), "{streamed}");
}
