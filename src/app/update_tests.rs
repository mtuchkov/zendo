//! The update check without an app (ADR 0008).
use super::*;
use tokio::sync::oneshot;

// ADR 0008: the answer of the check comes once; a newer commit stays for the top line.
#[test]
fn a_newer_commit_comes_once_and_stays() {
    let mut u = Update::default();
    assert_eq!(u.poll(), None, "no check runs");
    let (tx, rx) = oneshot::channel();
    u.watch(rx);
    assert_eq!(u.poll(), None, "no answer yet");
    tx.send(Some("abc1234".to_string())).unwrap();
    assert_eq!(u.poll().as_deref(), Some("abc1234"));
    assert_eq!((u.poll(), u.latest().map(String::as_str)), (None, Some("abc1234")));
}

// ADR 0008: a check that finds nothing, or that ends without an answer, leaves nothing.
#[test]
fn a_check_without_a_newer_commit_leaves_nothing() {
    let mut u = Update::default();
    let (tx, rx) = oneshot::channel();
    u.watch(rx);
    tx.send(None).unwrap();
    assert_eq!((u.poll(), u.latest()), (None, None));
    let (tx, rx) = oneshot::channel::<Option<String>>();
    u.watch(rx);
    drop(tx);
    assert_eq!((u.poll(), u.latest()), (None, None));
    assert!(!u.upgrade_now());
    u.upgrade();
    assert!(u.upgrade_now());
}
