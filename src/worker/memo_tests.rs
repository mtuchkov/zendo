use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

fn eventually<T: PartialEq + std::fmt::Debug>(want: T, f: impl Fn() -> T) {
    let end = Instant::now() + Duration::from_secs(5);
    loop {
        let got = f();
        if got == want {
            return;
        }
        assert!(Instant::now() < end, "still {got:?}, want {want:?}");
        std::thread::sleep(Duration::from_millis(5));
    }
}

// IO-1, IO-6: a get returns at once; the value comes from the worker, and until then the caller has
// nothing.
#[test]
fn a_get_returns_at_once_and_the_value_comes_later() {
    fn slow(n: &u32) -> u32 {
        std::thread::sleep(Duration::from_millis(100));
        n * 2
    }
    let m = Memo::new(slow, Duration::from_millis(20));
    let t = Instant::now();
    assert_eq!(m.get(&21), None, "not known yet");
    assert!(t.elapsed() < Duration::from_millis(10), "the get waited: {:?}", t.elapsed());
    eventually(Some(42), || m.get(&21));
}

// IO-6: the worker computes a key again while it is asked, so a change on the disk shows.
#[test]
fn an_asked_key_is_computed_again() {
    static CALLS: AtomicUsize = AtomicUsize::new(0);
    fn count(_: &u8) -> usize {
        CALLS.fetch_add(1, Ordering::SeqCst)
    }
    let m = Memo::new(count, Duration::from_millis(10));
    eventually(true, || m.get(&1).is_some());
    let first = m.get(&1).unwrap();
    eventually(true, || m.get(&1).is_some_and(|v| v > first));
}
