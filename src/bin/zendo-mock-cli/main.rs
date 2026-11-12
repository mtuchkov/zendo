//! A mock of the `claude` command line for the tests of the sessions (docs/zendo/sessions.md).
//!
//! It prints its arguments, then `tick N` each 100 ms. In raw mode it reads keys: `w` reports the
//! state "waits", `i` reports "idle", `q` exits. A state goes to the file in `$ZENDO_STATE`, as the
//! hooks of Claude Code do.

use ratatui::crossterm::terminal;
use std::io::{Read, Write};
use std::time::Duration;

fn report(state: &str) {
    if let Ok(path) = std::env::var("ZENDO_STATE") {
        let _ = std::fs::write(path, state);
    }
}

fn main() {
    let _ = terminal::enable_raw_mode();
    let args: Vec<String> = std::env::args().skip(1).collect();
    print!("mock cli: {}\r\n", args.join(" "));
    let _ = std::io::stdout().flush();
    std::thread::spawn(|| {
        let mut n = 0u64;
        loop {
            n += 1;
            print!("tick {n}\r\n");
            let _ = std::io::stdout().flush();
            std::thread::sleep(Duration::from_millis(100));
        }
    });
    let mut stdin = std::io::stdin();
    let mut b = [0u8; 1];
    while stdin.read(&mut b).is_ok_and(|n| n == 1) {
        match b[0] {
            b'w' => report("waits"),
            b'i' => report("idle"),
            b'q' => break,
            _ => {}
        }
    }
    let _ = terminal::disable_raw_mode();
}
