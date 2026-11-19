//! The journal file of the board: each writer appends one line in one write under a file lock
//! (ADR 0018).

use super::{Event, Line, State, now_ms};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct Journal {
    dir: PathBuf,
}

impl Journal {
    pub fn at(dir: impl Into<PathBuf>) -> Journal {
        Journal { dir: dir.into() }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn path(&self) -> PathBuf {
        self.dir.join("journal.jsonl")
    }

    fn open(&self) -> std::io::Result<File> {
        std::fs::create_dir_all(&self.dir)?;
        OpenOptions::new().create(true).read(true).append(true).open(self.path())
    }

    /// Append `events` with the time now.
    pub fn append(&self, events: &[Event]) -> std::io::Result<Vec<Line>> {
        self.transact(|_| Ok::<_, std::convert::Infallible>(events.to_vec())).map(|r| r.unwrap_or_default())
    }

    /// Under the lock: fold the whole journal, let `decide` choose the events to append, and append
    /// them. A check and its write are one step, so two writers cannot both win (BOARD-28).
    pub fn transact<E>(&self, decide: impl FnOnce(&State) -> Result<Vec<Event>, E>) -> std::io::Result<Result<Vec<Line>, E>> {
        let mut f = self.open()?;
        f.lock()?;
        let result = self.transact_locked(&mut f, decide);
        let _ = f.unlock();
        result
    }

    fn transact_locked<E>(&self, f: &mut File, decide: impl FnOnce(&State) -> Result<Vec<Event>, E>) -> std::io::Result<Result<Vec<Line>, E>> {
        let mut text = String::new();
        f.seek(SeekFrom::Start(0))?;
        f.read_to_string(&mut text)?;
        let state = State::fold(parse(&text).0);
        let events = match decide(&state) {
            Ok(e) => e,
            Err(e) => return Ok(Err(e)),
        };
        let at = now_ms();
        let lines: Vec<Line> = events.into_iter().map(|ev| Line { at, ev }).collect();
        let body: String = lines.iter().map(|l| format!("{}\n", serde_json::to_string(l).unwrap_or_default())).collect();
        f.write_all(body.as_bytes())?;
        f.flush()?;
        Ok(Ok(lines))
    }

    /// All lines.
    pub fn read(&self) -> Vec<Line> {
        self.read_from(0).0
    }

    /// The whole lines after byte `offset`, and the offset after the last whole line.
    pub fn read_from(&self, offset: u64) -> (Vec<Line>, u64) {
        let Ok(mut f) = File::open(self.path()) else { return (Vec::new(), offset) };
        let mut text = String::new();
        if f.seek(SeekFrom::Start(offset)).is_err() || f.read_to_string(&mut text).is_err() {
            return (Vec::new(), offset);
        }
        let (lines, used) = parse(&text);
        (lines, offset + used as u64)
    }

    /// The state of the whole journal.
    pub fn state(&self) -> State {
        State::fold(self.read())
    }
}

/// The lines of `text` up to its last new line, and the number of bytes that they take. A line
/// that does not parse is skipped.
fn parse(text: &str) -> (Vec<Line>, usize) {
    let used = text.rfind('\n').map_or(0, |i| i + 1);
    let lines = text[..used].lines().filter_map(|l| serde_json::from_str(l).ok()).collect();
    (lines, used)
}
