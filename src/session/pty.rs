//! One Claude Code process in a pseudo terminal, with its screen in a terminal emulator.

use super::output::Recorder;
use super::messages::{Inbox, Message};
use super::{Command, Resume, State, hooks, marks};
use portable_pty::{Child, CommandBuilder, MasterPty, PtySize};
use std::io::{Read, Write};
use std::num::NonZeroU16;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Lines of the screen kept after they scroll off it.
const SCROLLBACK: usize = 2000;
/// A session that reported *works* and printed nothing for this long is idle (SESSIONS-17).
const QUIET: Duration = Duration::from_secs(3);

/// Answers to the queries of the program: the cursor position and the device attributes; and the
/// result of the last command of a shell (TERM-23).
#[derive(Default)]
pub struct Replies(Vec<u8>, Recorder);

impl vt100::Callbacks for Replies {
    fn unhandled_csi(&mut self, screen: &mut vt100::Screen, i1: Option<u8>, _i2: Option<u8>, params: &[&[u16]], c: char) {
        match (i1, c, params.first().and_then(|p| p.first())) {
            (None, 'n', Some(6)) => {
                let (row, col) = screen.cursor_position();
                self.0.extend(format!("\x1b[{};{}R", row + 1, col + 1).bytes());
            }
            (None, 'n', Some(5)) => self.0.extend(b"\x1b[0n"),
            (None, 'c', None | Some(0)) => self.0.extend(b"\x1b[?62;22c"),
            _ => {}
        }
    }
}

type Screen = Arc<Mutex<vt100::Parser<Replies>>>;
type Writer = Arc<Mutex<Box<dyn Write + Send>>>;

pub struct Cli {
    child: Box<dyn Child + Send + Sync>,
    master: Box<dyn MasterPty + Send>,
    writer: Writer,
    screen: Screen,
    last_output: Arc<Mutex<Instant>>,
    state_file: PathBuf,
    reported: Option<State>,
    /// A shell of a terminal, not Claude Code: no hooks, busy while a command runs.
    shell: bool,
    started: Instant,
    /// A hook reported *works*, or the shell ran a command, since the last end (ATTN-1, ATTN-6).
    worked: bool,
    /// The messages from the hooks of Claude Code (CLIMSG-2); none for a shell.
    inbox: Option<Inbox>,
}

fn pty_size((rows, cols): (u16, u16)) -> PtySize {
    PtySize { rows, cols, pixel_width: 0, pixel_height: 0 }
}

fn nz(n: u16) -> NonZeroU16 {
    NonZeroU16::new(n.max(1)).unwrap_or(NonZeroU16::MIN)
}

impl Cli {
    /// Start `cmd` with the hooks, then `id`, in `cwd`, on a screen of `size`.
    pub fn spawn(cmd: &Command, resume: Resume, cwd: &Path, dir: &Path, n: usize, size: (u16, u16), notify: Arc<tokio::sync::Notify>) -> std::io::Result<Cli> {
        std::fs::create_dir_all(dir)?;
        let state_file = dir.join(format!("{}.state", n + 1));
        let _ = std::fs::remove_file(&state_file);
        let settings = dir.join("hooks.json");
        std::fs::write(&settings, hooks::settings())?;
        let mut b = CommandBuilder::new(&cmd.program);
        b.arg("--settings");
        b.arg(&settings);
        b.args(&cmd.args);
        b.arg(resume.id);
        b.args(resume.prompt);
        b.cwd(cwd);
        b.env("ZENDO_STATE", &state_file);
        let messages = dir.join(format!("{}.messages", n + 1));
        let _ = std::fs::remove_file(&messages);
        b.env("ZENDO_MESSAGES", &messages);
        b.env("TERM", "xterm-256color");
        let mut cli = Cli::start(b, size, notify, state_file, false)?;
        cli.inbox = Some(Inbox::new(messages));
        Ok(cli)
    }

    /// The prompts and answers that the hooks reported since the last call (CLIMSG-1).
    pub fn messages(&mut self) -> Vec<Message> {
        self.inbox.as_mut().map(Inbox::take).unwrap_or_default()
    }

    /// Start the shell `program` of a terminal in `cwd`, with the script of the marks in `dir`, and
    /// `home` as the home directory if it is given (TERM-12, TERM-34).
    pub fn spawn_shell(program: &str, cwd: &Path, dir: &Path, home: Option<&Path>, size: (u16, u16), notify: Arc<tokio::sync::Notify>) -> std::io::Result<Cli> {
        let mut b = CommandBuilder::new(program);
        if let Some(home) = home {
            b.env("HOME", home);
        }
        marks::prepare(&mut b, program, dir)?;
        b.cwd(cwd);
        b.env("TERM", "xterm-256color");
        Cli::start(b, size, notify, PathBuf::new(), true)
    }

    fn start(b: CommandBuilder, size: (u16, u16), notify: Arc<tokio::sync::Notify>, state_file: PathBuf, shell: bool) -> std::io::Result<Cli> {
        let err = |e: anyhow::Error| std::io::Error::other(e.to_string());
        let pair = portable_pty::native_pty_system().openpty(pty_size(size)).map_err(err)?;
        let child = pair.slave.spawn_command(b).map_err(err)?;
        drop(pair.slave);
        let reader = pair.master.try_clone_reader().map_err(err)?;
        let writer: Writer = Arc::new(Mutex::new(pair.master.take_writer().map_err(err)?));
        let screen: Screen = Arc::new(Mutex::new(vt100::Parser::new_with_callbacks(nz(size.0), nz(size.1), SCROLLBACK, Replies::default())));
        let started = Instant::now();
        let last_output = Arc::new(Mutex::new(started));
        read_loop(reader, screen.clone(), writer.clone(), last_output.clone(), notify);
        Ok(Cli { child, master: pair.master, writer, screen, last_output, state_file, reported: None, shell, started, worked: false, inbox: None })
    }

    /// A shell runs a command: the foreground process group of the terminal is not the shell.
    pub fn busy(&self) -> bool {
        #[cfg(unix)]
        if let (Some(group), Some(pid)) = (self.master.process_group_leader(), self.pid()) {
            return i64::from(group) != i64::from(pid);
        }
        false
    }

    /// The program printed and then was quiet for a moment: its prompt is ready for input.
    pub fn ready(&self) -> bool {
        let last = self.last_output.lock().map(|t| *t).unwrap_or(self.started);
        (last > self.started && last.elapsed() > Duration::from_millis(150)) || self.started.elapsed() > Duration::from_secs(3)
    }

    pub fn pid(&self) -> Option<u32> {
        self.child.process_id()
    }

    /// Run `f` with the recorder of the results of the commands.
    pub fn with_recorder<T>(&self, f: impl FnOnce(&mut Recorder) -> T) -> T {
        f(&mut self.screen.lock().unwrap_or_else(|e| e.into_inner()).callbacks_mut().1)
    }

    /// The time of the last output of the program.
    pub fn last_output(&self) -> Instant {
        self.last_output.lock().map(|t| *t).unwrap_or(self.started)
    }

    pub fn with_screen<T>(&self, f: impl FnOnce(&vt100::Screen) -> T) -> T {
        f(self.screen.lock().unwrap_or_else(|e| e.into_inner()).screen())
    }

    pub fn write(&self, bytes: &[u8]) {
        self.screen.lock().unwrap_or_else(|e| e.into_inner()).screen_mut().set_scrollback(0);
        let mut w = self.writer.lock().unwrap_or_else(|e| e.into_inner());
        let _ = w.write_all(bytes).and_then(|_| w.flush());
    }

    pub fn scroll(&self, up: bool) {
        let mut p = self.screen.lock().unwrap_or_else(|e| e.into_inner());
        let now = p.screen().scrollback();
        p.screen_mut().set_scrollback(if up { now + 3 } else { now.saturating_sub(3) });
    }

    pub fn resize(&self, size: (u16, u16)) {
        let _ = self.master.resize(pty_size(size));
        self.screen.lock().unwrap_or_else(|e| e.into_inner()).set_size(nz(size.0), nz(size.1));
    }

    pub fn ended(&mut self) -> bool {
        !matches!(self.child.try_wait(), Ok(None))
    }

    pub fn read_state(&mut self) {
        if self.shell {
            return;
        }
        if let Some(s) = hooks::read(&self.state_file) {
            self.reported = Some(s);
        }
    }

    /// The state that a hook reported; without one, *works* while the session prints (SESSIONS-17).
    pub fn state(&self) -> State {
        if self.shell {
            return if self.busy() { State::Works } else { State::Idle };
        }
        let quiet = self.last_output.lock().map(|t| t.elapsed()).unwrap_or_default();
        match self.reported {
            Some(State::Works) | None if quiet > QUIET => State::Idle,
            None if quiet > Duration::from_secs(1) => State::Idle,
            None => State::Works,
            Some(s) => s,
        }
    }

    /// The turn or the command ended since the last call: a hook reported *works* and then *idle* or
    /// *waits*, or the shell ran a command and is at its prompt again. A state that no hook
    /// reported does not count (ATTN-1, ATTN-6).
    pub fn finished(&mut self) -> bool {
        let (works, stopped) = if self.shell {
            let busy = self.busy();
            (busy, !busy)
        } else {
            (self.reported == Some(State::Works), matches!(self.reported, Some(State::Idle | State::Waits)))
        };
        self.worked |= works;
        let end = self.worked && stopped;
        self.worked &= !end;
        end
    }

    pub fn stop(mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        if !self.shell {
            let _ = std::fs::remove_file(&self.state_file);
        }
    }
}

/// Feed the output of the program to its screen, answer its queries, and wake the terminal loop.
fn read_loop(mut reader: Box<dyn Read + Send>, screen: Screen, writer: Writer, last: Arc<Mutex<Instant>>, notify: Arc<tokio::sync::Notify>) {
    std::thread::spawn(move || {
        let mut buf = [0u8; 8192];
        while let Ok(n) = reader.read(&mut buf) {
            if n == 0 {
                break;
            }
            let replies = {
                let mut p = screen.lock().unwrap_or_else(|e| e.into_inner());
                p.process(&buf[..n]);
                p.callbacks_mut().1.feed(&buf[..n]);
                std::mem::take(&mut p.callbacks_mut().0)
            };
            if !replies.is_empty() {
                let mut w = writer.lock().unwrap_or_else(|e| e.into_inner());
                let _ = w.write_all(&replies).and_then(|_| w.flush());
            }
            if let Ok(mut t) = last.lock() {
                *t = Instant::now();
            }
            notify.notify_one();
        }
        notify.notify_one();
    });
}
