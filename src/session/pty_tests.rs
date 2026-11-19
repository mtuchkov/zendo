use super::*;

// TERM-76: the terminal of zendo is not a VTE terminal: the shell does not get `VTE_VERSION`, so
// `/etc/profile.d/vte.sh` adds no marks of its own to the shell.
#[test]
fn a_shell_gets_no_vte_version() {
    let mut b = CommandBuilder::new("sh");
    b.env("VTE_VERSION", "7806");
    for_terminal(&mut b, Path::new("/"), None);
    assert_eq!(b.get_env("VTE_VERSION"), None);
    assert_eq!(b.get_env("TERM").and_then(|t| t.to_str()), Some("xterm-256color"));
}

// IO-1, IO-2: a write to a program that does not read its input returns at once; the input waits
// on the writer thread of the session.
#[test]
fn a_write_returns_while_the_program_does_not_read() {
    let notify = Arc::new(tokio::sync::Notify::new());
    let cli = Cli::spawn_shell("yes", Path::new("/"), None, (24, 80), notify).expect("yes starts");
    // A guard ends `yes` after 2 s, so that a write that waits ends too.
    let pid = cli.pid().unwrap_or(0).to_string();
    let (done, wait) = std::sync::mpsc::channel::<()>();
    let guard = std::thread::spawn(move || {
        if wait.recv_timeout(Duration::from_secs(2)).is_err() {
            let _ = std::process::Command::new("kill").arg(&pid).status();
        }
    });
    let t = Instant::now();
    cli.write(&vec![b'a'; 4 << 20]);
    let took = t.elapsed();
    let _ = done.send(());
    let _ = guard.join();
    assert!(took < Duration::from_millis(100), "the write waited for the program: {took:?}");
    cli.stop();
}
