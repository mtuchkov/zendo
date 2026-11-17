//! Read a config that a typo does not break (docs/zendo/cli/cli.md, CMD-17, CMD-18). Each table
//! header starts a block; a block with an error is read setting by setting, so a line with an
//! error costs only its own setting.

use super::UserConfig;
use std::collections::HashMap;

/// A line of the config with an error (CMD-18).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Problem {
    /// The line, from 1; None when the file cannot be read at all.
    pub line: Option<usize>,
    /// The error, on one line.
    pub error: String,
}

/// The lines of one block: from a table header to the next, or the settings before the first.
pub(super) struct Block {
    pub start: usize,
    pub end: usize,
    /// The name of the header: `notes` for `[notes]`, `keys.read` for `[keys.read]`; None for the
    /// settings before the first table.
    pub name: Option<String>,
}

impl Block {
    /// The lines of the settings: without the header.
    pub fn body(&self) -> std::ops::Range<usize> {
        self.start + usize::from(self.name.is_some())..self.end
    }

    /// The setting of the block: `keys` for `[keys.read]`.
    pub fn key(&self) -> Option<&str> {
        self.name.as_deref().map(|n| n.split('.').next().unwrap_or(n))
    }
}

/// `[` and `]` outside strings and comments: the depth that `line` adds to an array.
fn brackets(line: &str) -> i32 {
    let (mut quote, mut depth) = (None, 0);
    for c in line.chars() {
        match (quote, c) {
            (Some(q), c) if c == q => quote = None,
            (Some(_), _) => {}
            (None, '"' | '\'') => quote = Some(c),
            (None, '#') => break,
            (None, '[') => depth += 1,
            (None, ']') => depth -= 1,
            _ => {}
        }
    }
    depth
}

/// A whole header on its own line, as `[notes]`: no `=` and no `,` before its end.
fn whole_header(t: &str) -> bool {
    let t = t.split(" #").next().unwrap_or(t).trim_end();
    t.starts_with('[') && t.ends_with(']') && !t.contains(['=', ','])
}

/// Which lines are table headers: a `[` first on its line, outside an array over several lines and
/// a string over several lines. A whole header ends an array that a typo left open.
pub(super) fn headers(lines: &[&str]) -> Vec<bool> {
    let (mut depth, mut long) = (0, false);
    let mut header = |line: &&str| {
        let t = line.trim_start();
        long ^= line.matches("\"\"\"").count() % 2 == 1 || line.matches("'''").count() % 2 == 1;
        let is = !long && t.starts_with('[') && (depth == 0 || whole_header(t));
        depth = if is { 0 } else if long { depth } else { (depth + brackets(line)).max(0) };
        is
    };
    lines.iter().map(&mut header).collect()
}

/// The name of a header line: `keys.read` for `[keys.read]`, `name_models` for `[name_models`.
fn header_name(line: &str) -> String {
    let name = line.trim().trim_start_matches('[');
    let name = name.split(']').next().unwrap_or(name);
    name.split('.').map(|s| s.trim().trim_matches(['"', '\''])).collect::<Vec<_>>().join(".")
}

pub(super) fn blocks(lines: &[&str]) -> Vec<Block> {
    let is = headers(lines);
    let starts: Vec<usize> = std::iter::once(0).chain((1..lines.len()).filter(|&i| is[i])).collect();
    let ends = starts.iter().skip(1).copied().chain(std::iter::once(lines.len()));
    let name = |s: usize| is.get(s).copied().unwrap_or(false).then(|| header_name(lines[s]));
    starts.iter().zip(ends).map(|(&start, end)| Block { start, end, name: name(start) }).collect()
}

/// The setting of a line: `sound` for `sound = true`, `notes` for `notes.path = "/n"`.
pub(super) fn line_key(line: &str) -> Option<String> {
    line_path(line).map(|p| p.split('.').next().unwrap_or(&p).to_string())
}

/// The whole name of the setting of a line: `notes.path` for `notes.path = "/n"`.
pub(super) fn line_path(line: &str) -> Option<String> {
    let name = line.split_once('=')?.0.trim();
    let path: Vec<&str> = name.split('.').map(|s| s.trim().trim_matches(['"', '\''])).collect();
    (!name.is_empty() && !name.starts_with('#') && path.iter().all(|s| !s.is_empty())).then(|| path.join("."))
}

/// What the reader found so far.
#[derive(Default)]
struct Reader {
    all: toml::Table,
    problems: Vec<Problem>,
    /// The first line of each setting.
    at: HashMap<String, usize>,
}

impl Reader {
    fn problem(&mut self, line: usize, error: impl std::fmt::Display) {
        let error = error.to_string();
        self.problems.push(Problem { line: Some(line), error: error.lines().next().unwrap_or("").trim().to_string() });
    }

    /// Take the settings of `t`, which start at `line`; a setting that was there already is a
    /// problem, and the last one counts.
    fn take(&mut self, t: toml::Table, line: usize) {
        let mut twice = Vec::new();
        merge(&mut self.all, t, "", &mut twice);
        if !twice.is_empty() {
            self.problem(line, format!("{} set twice; zendo uses the last", twice.join(", ")));
        }
    }

    fn block(&mut self, lines: &[&str], b: &Block) {
        match lines[b.start..b.end].join("\n").parse::<toml::Table>() {
            Ok(t) => self.take(t, b.start + 1),
            Err(_) => self.settings(lines, b),
        }
    }

    /// A block with an error, setting by setting: each takes the fewest lines that parse, so an
    /// array over lines stays one setting; a line that parses with none is a problem.
    fn settings(&mut self, lines: &[&str], b: &Block) {
        let head = b.name.as_ref().map(|_| format!("{}\n", lines[b.start]));
        if let Some(Err(e)) = head.as_ref().map(|h| h.parse::<toml::Table>()) {
            return self.problem(b.start + 1, e.message());
        }
        let head = head.unwrap_or_default();
        let parse = |i: usize, j: usize| format!("{head}{}", lines[i..j].join("\n")).parse::<toml::Table>();
        let mut i = b.body().start;
        while i < b.end {
            match (i + 1..=b.end).find_map(|j| parse(i, j).ok().map(|t| (j, t))) {
                Some((j, t)) => {
                    self.take(t, i + 1);
                    i = j;
                }
                None => {
                    let error = parse(i, i + 1).err().map(|e| e.message().to_string()).unwrap_or_default();
                    self.problem(i + 1, error);
                    i += 1;
                }
            }
        }
    }

    fn lines_of_settings(&mut self, lines: &[&str], blocks: &[Block]) {
        for b in blocks {
            let found: Vec<(String, usize)> = match b.key() {
                Some(k) => vec![(k.to_string(), b.start + 1)],
                None => b.body().filter_map(|i| line_key(lines[i]).map(|k| (k, i + 1))).collect(),
            };
            for (k, n) in found {
                self.at.entry(k).or_insert(n);
            }
        }
    }

    /// Each setting on its own: a value of the wrong type costs only its setting.
    fn finish(mut self) -> (UserConfig, Vec<Problem>) {
        let mut good = toml::Table::new();
        for (k, v) in std::mem::take(&mut self.all) {
            let one = toml::Table::from_iter([(k.clone(), v.clone())]);
            match toml::Value::Table(one).try_into::<UserConfig>() {
                Ok(_) => drop(good.insert(k, v)),
                Err(e) => {
                    let line = self.at.get(&k).copied().unwrap_or(0);
                    self.problem(line, format!("{k}: {}", e.message()));
                }
            }
        }
        self.problems.sort_by_key(|p| p.line);
        (toml::Value::Table(good).try_into().unwrap_or_default(), self.problems)
    }
}

/// Put the settings of `from` into `into`: the tables of one setting, as `[keys.read]` and
/// `[keys.write]`, go together; a value that was there already goes into `twice`.
fn merge(into: &mut toml::Table, from: toml::Table, path: &str, twice: &mut Vec<String>) {
    for (k, v) in from {
        let name = if path.is_empty() { k.clone() } else { format!("{path}.{k}") };
        if let (Some(toml::Value::Table(a)), toml::Value::Table(b)) = (into.get_mut(&k), &v) {
            merge(a, b.clone(), &name, twice);
            continue;
        }
        if into.insert(k, v).is_some() {
            twice.push(name);
        }
    }
}

/// The settings of `text` without an error, and the lines with one (CMD-17, CMD-18).
pub fn read_text(text: &str) -> (UserConfig, Vec<Problem>) {
    // CRLF line endings read as LF: a bare `\r` at the end of a line is no error.
    let text = text.replace("\r\n", "\n");
    let lines: Vec<&str> = text.split('\n').collect();
    let blocks = blocks(&lines);
    let mut r = Reader::default();
    for b in &blocks {
        r.block(&lines, b);
    }
    r.lines_of_settings(&lines, &blocks);
    r.finish()
}

#[cfg(test)]
#[path = "read_tests.rs"]
mod tests;
