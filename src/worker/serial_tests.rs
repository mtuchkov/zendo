use super::*;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

fn answers(s: &mut Serial, n: usize) -> Vec<(u64, Result<(), String>)> {
    let end = Instant::now() + Duration::from_secs(5);
    let mut got = Vec::new();
    while got.len() < n {
        assert!(Instant::now() < end, "no answer");
        got.extend(s.answers());
        std::thread::sleep(Duration::from_millis(2));
    }
    got
}

// IO-1: a job returns at once; the jobs run in order, one at a time, and each answers once.
#[test]
fn jobs_run_in_order_and_answer_once() {
    let mut s = Serial::default();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let t = Instant::now();
    let ids: Vec<u64> = (0..3)
        .map(|i| {
            let seen = seen.clone();
            s.run(move || {
                std::thread::sleep(Duration::from_millis(20));
                seen.lock().unwrap().push(i);
                if i == 1 { Err("no".into()) } else { Ok(()) }
            })
        })
        .collect();
    assert!(t.elapsed() < Duration::from_millis(10), "the jobs did not wait");
    let got = answers(&mut s, 3);
    assert_eq!(got, vec![(ids[0], Ok(())), (ids[1], Err("no".to_string())), (ids[2], Ok(()))]);
    assert_eq!(*seen.lock().unwrap(), [0, 1, 2]);
    assert!(s.answers().is_empty(), "once");
}
