//! The search of the find panel, off the UI thread, with the search engine of ripgrep (ADR 0011):
//! one worker walks the repository when the panel opens and searches the text of the files on the
//! disk for each query. It keeps no copy of the text (FIND-34 to FIND-38).
use grep_regex::{RegexMatcher, RegexMatcherBuilder};
use grep_searcher::{BinaryDetection, SearcherBuilder, sinks::UTF8};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;
use tokio::sync::Notify;

/// The form of a requirement ID when `project-structure.toml` sets none (FIND-32).
pub const DEFAULT_ID: &str = "[A-Z][A-Z0-9]*-[0-9]+";
/// The search starts when the user stops typing for this long.
const DEBOUNCE: Duration = Duration::from_millis(100);
/// The most files that a walk lists.
const MAX_FILES: usize = 100_000;

/// A line with a match.
#[derive(Clone, Debug)]
pub struct Snippet {
    /// The line in the file, from 0.
    pub line: usize,
    pub text: String,
    /// Byte ranges of the matches in `text`.
    pub hits: Vec<(usize, usize)>,
}

/// A file with matches in its text.
#[derive(Clone, Debug)]
pub struct Hit {
    pub path: PathBuf,
    /// The number of matches: of the text in each line, or of the lines with the tag or the ID.
    pub count: usize,
    /// For an ID: the file has the whole ID, not only an ID that starts with it.
    pub exact: bool,
    /// Up to two lines, not the title line.
    pub snippets: Vec<Snippet>,
}

/// What to look for in the text.
#[derive(Clone, Debug)]
pub enum Ask {
    /// The query in markdown files: plain text, or a regular expression.
    Text { query: String, regex: bool },
    /// A tag that starts with the word, in markdown files (FIND-31).
    Tag(String),
    /// A requirement ID that starts with the word, in every text file (FIND-30).
    Id { word: String, pattern: regex::Regex },
}

impl Ask {
    /// The expression that finds the candidate lines; the line test of `Tag` and `Id` comes after
    /// it.
    fn source(&self) -> String {
        match self {
            Ask::Text { query, regex: true } => query.clone(),
            Ask::Text { query, .. } => regex::escape(query),
            Ask::Tag(w) => format!("#{}", regex::escape(w)),
            Ask::Id { word, .. } => regex::escape(word),
        }
    }

    /// The marks of a match in a line, without regard to letter case; None for a bad expression.
    pub fn mark(&self) -> Option<regex::Regex> {
        regex::RegexBuilder::new(&self.source()).case_insensitive(true).build().ok()
    }

    fn code_too(&self) -> bool {
        matches!(self, Ask::Id { .. })
    }
}

/// Every file under the roots that git does not ignore, outside hidden folders, sorted.
pub fn walk(roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = roots
        .iter()
        .flat_map(|r| ignore::WalkBuilder::new(r).hidden(true).git_ignore(true).require_git(false).build().flatten())
        .filter(|e| e.file_type().is_some_and(|t| t.is_file()))
        .map(ignore::DirEntry::into_path)
        .take(MAX_FILES)
        .collect();
    files.sort();
    files
}

fn is_md(p: &Path) -> bool {
    p.extension().is_some_and(|e| e == "md")
}

/// The files with a match, in parallel; None when `stop` says the search is old.
pub fn search(files: &[PathBuf], ask: &Ask, stop: &(dyn Fn() -> bool + Sync)) -> Option<Vec<Hit>> {
    let Some(matcher) = prefilter(ask) else { return Some(Vec::new()) };
    let mark = ask.mark();
    let todo: Vec<&PathBuf> = files.iter().filter(|p| ask.code_too() || is_md(p)).collect();
    let out = Mutex::new(Vec::new());
    let threads = std::thread::available_parallelism().map_or(2, |n| n.get()).clamp(1, 8);
    // Each thread takes the next file, so that a folder of large files does not load one thread.
    let next = std::sync::atomic::AtomicUsize::new(0);
    std::thread::scope(|s| {
        for _ in 0..threads {
            s.spawn(|| {
                let mut searcher = SearcherBuilder::new().binary_detection(BinaryDetection::quit(0)).line_number(true).build();
                while let Some(p) = todo.get(next.fetch_add(1, Ordering::Relaxed)).filter(|_| !stop()) {
                    if let Some(hit) = search_file(&mut searcher, &matcher, ask, mark.as_ref(), p) {
                        out.lock().expect("no panic holds the lock").push(hit);
                    }
                }
            });
        }
    });
    (!stop()).then(|| out.into_inner().expect("no panic holds the lock"))
}

fn prefilter(ask: &Ask) -> Option<RegexMatcher> {
    RegexMatcherBuilder::new().case_insensitive(true).build(&ask.source()).ok()
}

fn search_file(searcher: &mut grep_searcher::Searcher, matcher: &RegexMatcher, ask: &Ask, mark: Option<&regex::Regex>, p: &Path) -> Option<Hit> {
    let mut hit = Hit { path: p.to_path_buf(), count: 0, exact: false, snippets: Vec::new() };
    let sink = UTF8(|n, line| {
        let line = line.trim_end_matches(['\n', '\r']);
        let (ok, exact) = line_matches(ask, line);
        if ok {
            let text = matches!(ask, Ask::Text { .. });
            hit.count += if text { mark.map_or(1, |m| m.find_iter(line).count().max(1)) } else { 1 };
            hit.exact |= exact;
            let title = n == 1 && line.starts_with("# ") && !matches!(ask, Ask::Tag(_));
            if hit.snippets.len() < 2 && !title {
                hit.snippets.push(snippet(n as usize - 1, line, mark));
            }
        }
        Ok(true)
    });
    searcher.search_path(matcher, p, sink).ok()?;
    (hit.count > 0).then_some(hit)
}

/// The line has a match for the ask; and for an ID, the whole ID.
fn line_matches(ask: &Ask, line: &str) -> (bool, bool) {
    match ask {
        Ask::Text { .. } => (true, false),
        Ask::Tag(w) => {
            let w = w.to_lowercase();
            (crate::docs::notes::tags_in(line).iter().any(|t| t.to_lowercase().starts_with(&w)), false)
        }
        Ask::Id { word, pattern } => {
            let w = word.to_lowercase();
            let ids = pattern.find_iter(line).filter(|m| !line[..m.start()].chars().next_back().is_some_and(|c| c.is_alphanumeric() || c == '_'));
            ids.map(|m| m.as_str().to_lowercase()).filter(|id| id.starts_with(&w)).fold((false, false), |(_, e), id| (true, e || id == w))
        }
    }
}

fn snippet(line: usize, text: &str, mark: Option<&regex::Regex>) -> Snippet {
    let text = text.trim().trim_start_matches(['#', '-', '*', ' ']).to_string();
    let hits = mark.map(|m| m.find_iter(&text).map(|h| (h.start(), h.end())).collect()).unwrap_or_default();
    Snippet { line, text, hits }
}

/// What the worker sends.
#[derive(Debug)]
pub enum Out {
    /// The files of the repository, for the search by path while the panel is open (FIND-37).
    Files(Vec<PathBuf>),
    /// The matches in the text for the query with this number.
    Hits { seq: u64, hits: Vec<Hit> },
}

enum Job {
    Walk(Vec<PathBuf>),
    Search(u64, Ask),
}

/// The worker thread of the find panel. The UI thread never waits for it.
pub struct Worker {
    jobs: mpsc::Sender<Job>,
    out: mpsc::Receiver<Out>,
    seq: Arc<AtomicU64>,
}

impl Worker {
    /// Start the thread; `wake` tells the event loop that results are there.
    pub fn start(wake: Arc<Notify>) -> Self {
        let (jobs, rx) = mpsc::channel();
        let (tx, out) = mpsc::channel();
        let seq = Arc::new(AtomicU64::new(0));
        let g = seq.clone();
        std::thread::spawn(move || run(&rx, &tx, &g, &wake));
        Self { jobs, out, seq }
    }

    /// Walk the roots again: the panel opens (FIND-37).
    pub fn walk(&self, roots: Vec<PathBuf>) {
        let _ = self.jobs.send(Job::Walk(roots));
    }

    /// Search for the query with the number `seq`; the search of an older number stops (FIND-38).
    pub fn search(&self, seq: u64, ask: Ask) {
        self.seq.store(seq, Ordering::SeqCst);
        let _ = self.jobs.send(Job::Search(seq, ask));
    }

    /// No search for now: the running one stops.
    pub fn stop(&self, seq: u64) {
        self.seq.store(seq, Ordering::SeqCst);
    }

    /// What the worker sent, without a wait.
    pub fn poll(&self) -> Vec<Out> {
        self.out.try_iter().collect()
    }

    /// Wait for the next answer, for tests and benchmarks.
    pub fn wait(&self, timeout: Duration) -> Option<Out> {
        self.out.recv_timeout(timeout).ok()
    }
}

fn run(jobs: &mpsc::Receiver<Job>, out: &mpsc::Sender<Out>, seq: &AtomicU64, wake: &Notify) {
    let mut files = Vec::new();
    let mut pending: Option<(u64, Ask)> = None;
    loop {
        let job = match pending {
            Some(_) => match jobs.recv_timeout(DEBOUNCE) {
                Ok(j) => Some(j),
                Err(mpsc::RecvTimeoutError::Timeout) => None,
                Err(mpsc::RecvTimeoutError::Disconnected) => return,
            },
            None => match jobs.recv() {
                Ok(j) => Some(j),
                Err(_) => return,
            },
        };
        let sent = match job {
            Some(Job::Walk(roots)) => {
                files = walk(&roots);
                out.send(Out::Files(files.clone()))
            }
            Some(Job::Search(g, ask)) => {
                pending = Some((g, ask));
                continue;
            }
            None => {
                let Some((g, ask)) = pending.take() else { continue };
                let Some(hits) = search(&files, &ask, &|| seq.load(Ordering::SeqCst) != g) else { continue };
                out.send(Out::Hits { seq: g, hits })
            }
        };
        if sent.is_err() {
            return;
        }
        wake.notify_one();
    }
}

#[cfg(test)]
#[path = "grep_tests.rs"]
mod tests;
