use super::*;
use std::time::{Duration, Instant};

fn answer<T: Send + 'static>(t: &mut Task<T>) -> Option<T> {
    let end = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(a) = t.poll() {
            return Some(a);
        }
        if Instant::now() > end {
            return None;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
}

// IO-1, IO-6: a task runs on a worker; the answer comes on a later poll, and the start returns at
// once.
#[test]
fn an_answer_comes_on_a_later_poll() {
    let mut t = Task::new(Duration::from_secs(5));
    let started = Instant::now();
    t.start(|| {
        std::thread::sleep(Duration::from_millis(50));
        7
    });
    assert!(started.elapsed() < Duration::from_millis(10));
    assert!(t.running());
    assert_eq!(t.poll(), None, "not yet");
    assert_eq!(answer(&mut t), Some(7));
    assert!(!t.running());
}

// IO-6: a new start drops the answer of the older one.
#[test]
fn a_new_start_drops_the_older_answer() {
    let mut t = Task::new(Duration::from_secs(5));
    t.start(|| {
        std::thread::sleep(Duration::from_millis(100));
        "old"
    });
    t.start(|| "new");
    assert_eq!(answer(&mut t), Some("new"));
    std::thread::sleep(Duration::from_millis(150));
    assert_eq!(t.poll(), None, "the older answer is dropped");
}

// IO-5: a task over its limit is overdue once; zendo stops waiting and drops its late answer.
#[test]
fn a_task_over_its_limit_is_overdue_once() {
    let mut t = Task::new(Duration::from_millis(30));
    t.start(|| {
        std::thread::sleep(Duration::from_millis(200));
        1
    });
    assert!(!t.overdue());
    std::thread::sleep(Duration::from_millis(50));
    assert!(t.overdue(), "over the limit");
    assert!(!t.running(), "zendo stopped waiting");
    assert!(!t.overdue(), "once");
    std::thread::sleep(Duration::from_millis(200));
    assert_eq!(t.poll(), None, "the late answer is dropped");
}
