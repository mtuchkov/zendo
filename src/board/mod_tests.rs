use super::*;

/// A journal and a context of session `s` in a folder of the test; repository `/r`.
pub(crate) fn ctx(name: &str, s: usize) -> tools::Ctx {
    let dir = std::env::temp_dir().join(format!("zendo-board-{name}-{}", std::process::id()));
    tools::Ctx { journal: Journal::at(dir), repo: "/r".into(), session: s, exe: "zendo".into(), choose_for: std::time::Duration::from_millis(400) }
}

/// The same journal for another session.
pub(crate) fn other(c: &tools::Ctx, s: usize) -> tools::Ctx {
    tools::Ctx { journal: c.journal.clone(), repo: c.repo.clone(), session: s, exe: c.exe.clone(), choose_for: c.choose_for }
}

/// A fresh folder for the test: an interrupted earlier run leaves nothing behind.
pub(crate) fn fresh(name: &str, s: usize) -> tools::Ctx {
    let c = ctx(name, s);
    let _ = std::fs::remove_dir_all(c.journal.dir());
    c
}

pub(crate) fn beat(c: &tools::Ctx) {
    c.journal.beat(&Beat { pid: std::process::id(), repo: c.repo.clone(), at: now_ms(), sessions: Vec::new() }).unwrap();
}

// ADR 0018: the start time tells this process from another process with the same id.
#[test]
fn this_process_is_alive_with_its_start_time() {
    let me = std::process::id();
    assert!(proc_alive(me, proc_start(me)));
    if let Some(s) = proc_start(me) {
        assert!(!proc_alive(me, Some(s + 1)), "another start time is another process");
    }
}

// ADR 0018: a folder outside git names itself.
#[test]
fn a_folder_outside_git_is_its_own_repository() {
    let d = std::env::temp_dir().join(format!("zendo-board-nogit-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    let name = repo_of(&d);
    assert!(name.ends_with(&format!("zendo-board-nogit-{}", std::process::id())), "{name}");
    let _ = std::fs::remove_dir_all(&d);
}

// BOARD-28: two writers that claim one delivery at once: one wins.
#[test]
fn one_of_two_claims_wins() {
    let c = fresh("claims", 3);
    let id = tools::call(&c, "ask", &serde_json::json!({ "to": "session:2", "text": "who owns feat/x?" })).unwrap().unwrap()["id"].as_str().unwrap().to_string();
    let wins: Vec<bool> = std::thread::scope(|s| {
        let hs: Vec<_> = (0..4).map(|i| s.spawn({ let j = c.journal.clone(); let id = id.clone(); move || j.claim(&id, 2, if i % 2 == 0 { How::Delivered } else { How::Watched }) })).collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    assert_eq!(wins.iter().filter(|w| **w).count(), 1, "{wins:?}");
    let _ = std::fs::remove_dir_all(c.journal.dir());
}

// BOARD-11: the agent gets the message in a tag; the chat shows the sender.
#[test]
fn a_message_is_tagged_for_the_agent_and_labeled_for_the_chat() {
    let m = Msg { id: "m1".into(), repo: "/r".into(), from: 3, to: Some("session:2".into()), kind: Kind::Task, status: None, in_reply_to: None, reply_to: None, text: "rebase".into() };
    let t = tagged(&m);
    assert!(t.starts_with("<zendo_message from=\"3\" id=\"m1\" kind=\"task\" reply_to=\"session:3\""), "{t}");
    assert!(t.trim_end().ends_with("</zendo_message>") && t.contains("\nrebase\n"), "{t}");
    assert_eq!(label(&m, "oci"), "from 3 · oci · task: rebase");
    assert_eq!(label(&m, ""), "from 3 · task: rebase");
}
