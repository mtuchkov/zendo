//! Posts and leases (BOARD-13 to BOARD-19, BOARD-29 to BOARD-35).

use super::super::tests::{beat, fresh, other};
use super::super::tools::call;
use super::super::{Event, End, now_ms};
use serde_json::json;

fn posts(c: &super::super::tools::Ctx) -> serde_json::Value {
    call(c, "posts", &json!({})).unwrap().unwrap()
}

// BOARD-13, BOARD-29: a post shows to a session of another repository, also after a restart: the
// journal holds it.
#[test]
fn a_post_shows_in_another_repository() {
    let a = fresh("post-repos", 2);
    beat(&a);
    call(&a, "post", &json!({ "resource": "port:5220", "text": "dev server" })).unwrap().unwrap();
    let mut b = other(&a, 1);
    b.repo = "/other".into();
    let v = posts(&b);
    assert_eq!(v["posts"][0]["resource"], "port:5220");
    assert_eq!(v["posts"][0]["repo"], "/r");
    assert_eq!(v["posts"][0]["lease_live"], true);
    let again = super::super::Journal::at(a.journal.dir()).state();
    assert_eq!(again.live_posts().count(), 1, "a restart reads the same journal");
    let _ = std::fs::remove_dir_all(a.journal.dir());
}

// BOARD-15, BOARD-16: release moves a post to the history; two sessions can post one resource.
#[test]
fn release_moves_a_post_to_the_history_and_two_posts_both_show() {
    let a = fresh("post-release", 2);
    let b = other(&a, 4);
    call(&a, "post", &json!({ "resource": "docker:foo", "text": "kind" })).unwrap().unwrap();
    call(&b, "post", &json!({ "resource": "docker:foo", "text": "mine too" })).unwrap().unwrap();
    assert_eq!(posts(&a)["posts"].as_array().unwrap().len(), 2, "zendo does not decide a conflict");
    call(&a, "release", &json!({ "resource": "docker:foo" })).unwrap().unwrap();
    let v = posts(&a);
    assert_eq!(v["posts"].as_array().unwrap().len(), 1);
    assert_eq!(v["history"][0]["ended"], "Released");
    assert!(call(&a, "release", &json!({ "resource": "docker:foo" })).unwrap().is_err());
    let _ = std::fs::remove_dir_all(a.journal.dir());
}

// BOARD-15: the end of the thread of the author moves its posts to the history.
#[test]
fn the_end_of_the_thread_ends_its_posts() {
    let a = fresh("post-thread", 2);
    call(&a, "post", &json!({ "resource": "port:1", "text": "x" })).unwrap().unwrap();
    a.journal.append(&[Event::ThreadEnd { repo: "/r".into(), session: 2 }]).unwrap();
    assert_eq!(a.journal.state().history().next().unwrap().end, Some(End::ThreadEnded));
    let _ = std::fs::remove_dir_all(a.journal.dir());
}

// BOARD-31, BOARD-32: while no zendo of the repository beats, the lease is expired, and `posts`
// returns for how long.
#[test]
fn a_lease_without_a_heartbeat_is_expired_with_its_age() {
    let a = fresh("post-age", 2);
    call(&a, "post", &json!({ "resource": "port:2", "text": "x" })).unwrap().unwrap();
    let v = posts(&a);
    assert_eq!(v["posts"][0]["lease_live"], false, "no zendo beats");
    assert!(v["posts"][0]["expired_for_seconds"].is_u64());
    let _ = std::fs::remove_dir_all(a.journal.dir());
}

// BOARD-33, BOARD-34, BOARD-35: a live lease refuses a tombstone; an expired one takes it, and a
// later renewal does not bring the post back.
#[test]
fn a_dead_post_can_be_tombstoned() {
    let a = fresh("post-tomb", 2);
    beat(&a);
    let p = call(&a, "post", &json!({ "resource": "port:3", "text": "x" })).unwrap().unwrap()["id"].as_str().unwrap().to_string();
    let b = other(&a, 1);
    assert!(call(&b, "tombstone", &json!({ "post": p })).unwrap().unwrap_err().contains("live"));
    a.journal.append(&[Event::Expired { post: p.clone() }]).unwrap();
    call(&b, "tombstone", &json!({ "post": p })).unwrap().unwrap();
    a.journal.append(&[Event::Renewed { post: p.clone() }]).unwrap();
    let st = a.journal.state();
    assert_eq!(st.post(&p).unwrap().end, Some(End::Tombstone { repo: "/r".into(), session: 1 }));
    assert_eq!(st.live_posts().count(), 0);
    let _ = std::fs::remove_dir_all(a.journal.dir());
}

// BOARD-30: an expiry, then a renewal after a restart: the lease is live again.
#[test]
fn a_renewal_after_an_expiry_makes_the_lease_live() {
    let a = fresh("post-renew", 2);
    beat(&a);
    let p = call(&a, "post", &json!({ "resource": "port:4", "text": "x" })).unwrap().unwrap()["id"].as_str().unwrap().to_string();
    a.journal.append(&[Event::Expired { post: p.clone() }]).unwrap();
    assert!(!a.journal.state().post(&p).unwrap().lease(now_ms(), Some(now_ms())).live);
    a.journal.append(&[Event::Renewed { post: p.clone() }]).unwrap();
    assert!(a.journal.state().post(&p).unwrap().lease(now_ms(), Some(now_ms())).live);
    let _ = std::fs::remove_dir_all(a.journal.dir());
}
