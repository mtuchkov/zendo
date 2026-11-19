//! The journal file of the board: each writer appends its lines in one write under the lock of
//! `journal.lock`, and a large journal is compacted under the same lock (ADR 0018).

use super::{Event, Line, State, now_ms};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

/// A journal larger than this is compacted after a write.
pub const COMPACT_BYTES: u64 = 1 << 20;

#[derive(Clone, Debug)]
pub struct Journal {
    dir: PathBuf,
    /// The size over which a write compacts the journal.
    limit: u64,
}

impl Journal {
    pub fn at(dir: impl Into<PathBuf>) -> Journal {
        Journal { dir: dir.into(), limit: COMPACT_BYTES }
    }

    /// The same journal, compacted over `limit` bytes.
    pub fn with_limit(self, limit: u64) -> Journal {
        Journal { limit, ..self }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn path(&self) -> PathBuf {
        self.dir.join("journal.jsonl")
    }

    /// The lock of every write. It is a file of its own: a compaction replaces the journal, and a
    /// writer that waits holds no handle of the old one.
    fn lock(&self) -> std::io::Result<File> {
        std::fs::create_dir_all(&self.dir)?;
        let f = OpenOptions::new().create(true).truncate(false).write(true).open(self.dir.join("journal.lock"))?;
        f.lock()?;
        Ok(f)
    }

    /// Append `events` with the time now.
    pub fn append(&self, events: &[Event]) -> std::io::Result<Vec<Line>> {
        let at = now_ms();
        self.append_lines(&events.iter().cloned().map(|ev| Line { at, ev }).collect::<Vec<_>>())
    }

    /// Append `lines` with their own times.
    pub fn append_lines(&self, lines: &[Line]) -> std::io::Result<Vec<Line>> {
        self.transact_lines(|_| Ok::<_, std::convert::Infallible>(lines.to_vec())).map(|r| r.unwrap_or_default())
    }

    /// Under the lock: fold the whole journal, let `decide` choose the events to append, and append
    /// them. A check and its write are one step, so two writers cannot both win (BOARD-28).
    pub fn transact<E>(&self, decide: impl FnOnce(&State) -> Result<Vec<Event>, E>) -> std::io::Result<Result<Vec<Line>, E>> {
        let at = now_ms();
        self.transact_lines(|st| decide(st).map(|ev| ev.into_iter().map(|ev| Line { at, ev }).collect()))
    }

    fn transact_lines<E>(&self, decide: impl FnOnce(&State) -> Result<Vec<Line>, E>) -> std::io::Result<Result<Vec<Line>, E>> {
        let lock = self.lock()?;
        let result = self.write_locked(decide);
        let _ = lock.unlock();
        result
    }

    fn write_locked<E>(&self, decide: impl FnOnce(&State) -> Result<Vec<Line>, E>) -> std::io::Result<Result<Vec<Line>, E>> {
        let mut f = OpenOptions::new().create(true).read(true).append(true).open(self.path())?;
        let mut text = String::new();
        f.read_to_string(&mut text)?;
        let lines = parse(&text).0;
        let lines = match decide(&State::fold(lines.clone())) {
            Ok(new) => new,
            Err(e) => return Ok(Err(e)),
        };
        let body: String = lines.iter().map(|l| format!("{}\n", serde_json::to_string(l).unwrap_or_default())).collect();
        f.write_all(body.as_bytes())?;
        f.flush()?;
        if text.len() as u64 + body.len() as u64 > self.limit {
            self.compact_locked(now_ms())?;
        }
        Ok(Ok(lines))
    }

    /// Compact the journal now, under the lock (ADR 0018).
    pub fn compact(&self, now: u64) -> std::io::Result<()> {
        let lock = self.lock()?;
        let r = self.compact_locked(now);
        let _ = lock.unlock();
        r
    }

    fn compact_locked(&self, now: u64) -> std::io::Result<()> {
        let mut text = String::new();
        File::open(self.path())?.read_to_string(&mut text)?;
        let kept = super::compact::keep(parse(&text).0, now);
        crate::atomic::write_with(&self.path(), |f| kept.iter().try_for_each(|l| writeln!(f, "{}", serde_json::to_string(l).unwrap_or_default())))
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

    /// The file of the journal now: another one after a compaction. Its id and its size.
    pub fn stamp(&self) -> (u64, u64) {
        let Ok(m) = std::fs::metadata(self.path()) else { return (0, 0) };
        #[cfg(unix)]
        let id = std::os::unix::fs::MetadataExt::ino(&m);
        #[cfg(not(unix))]
        let id = m.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map_or(0, |d| d.as_nanos() as u64);
        (id, m.len())
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
