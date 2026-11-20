//! The reader of `.envrc` (docs/zendo/envrc.md): zendo takes the variable lines of the file
//! itself, and runs nothing.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// The words that open a block of the shell, the words that close one, and the words that can
/// stand before the first word of a command.
const OPEN: [&str; 6] = ["if", "case", "for", "while", "until", "select"];
const CLOSE: [&str; 3] = ["fi", "esac", "done"];
const LEAD: [&str; 5] = ["then", "else", "do", "!", "time"];
/// The ends of a line that goes on in the next one: `\`, `&&`, `||` and `|`.
const GOES_ON: [&str; 3] = ["\\", "&&", "|"];
/// The words that declare a variable: each `NAME=value` after one of them sets a variable.
const DECLARE: [&str; 5] = ["export", "declare", "typeset", "readonly", "local"];

/// Where the reader is in the text of the shell, after the lines that are no variables.
#[derive(Default)]
struct Place {
    /// The blocks that are open: a condition, a loop, a function.
    depth: usize,
    /// The lists in parentheses that are open: `NAME=(`, `NAME=$(`, a subshell.
    lists: usize,
    /// The quote of a text that goes on in the next line.
    quote: Option<char>,
    /// The end word of a here-document that goes on.
    heredoc: Option<String>,
    /// The line before goes on in this one.
    joined: bool,
}

impl Place {
    /// The next line is a part of what a line above started (ENV-9).
    fn inside(&self) -> bool {
        self.depth > 0 || self.lists > 0 || self.quote.is_some() || self.heredoc.is_some() || self.joined
    }

    /// Go over a line that is no variable.
    fn pass(&mut self, line: &str) {
        if let Some(end) = &self.heredoc {
            if line.trim() == end {
                self.heredoc = None;
            }
            return;
        }
        // The code outside the quotes: a text in quotes opens and closes nothing.
        let (bare, quote) = scan(line, self.quote);
        self.quote = quote;
        let code = bare.trim();
        self.joined = quote.is_none() && GOES_ON.iter().any(|e| code.ends_with(e));
        let in_list = self.lists > 0;
        self.lists = (self.lists + code.matches('(').count()).saturating_sub(code.matches(')').count());
        // The lines of a list are no commands, so their words open and close no block.
        if code.is_empty() || (in_list && self.lists > 0) {
            return;
        }
        self.depth = self.depth.saturating_add_signed(code.split([';', '&', '|']).map(command).sum());
        self.heredoc = code.contains("<<").then(|| heredoc(line)).flatten();
    }
}

/// The variables of `.envrc` and the numbers of the lines that zendo skipped, from 1 (ENV-4).
/// zendo runs nothing: `env` gives a variable of the environment of zendo for `$NAME` (ENV-8).
pub fn read_envrc(text: &str, env: &dyn Fn(&str) -> Option<String>, home: &Path) -> (BTreeMap<String, String>, Vec<usize>) {
    let mut vars = BTreeMap::new();
    // The variables that a skipped line sets: zendo does not know their values (ENV-8).
    let mut unknown = BTreeSet::new();
    let mut skipped = Vec::new();
    let mut place = Place::default();
    for (n, line) in text.lines().enumerate().map(|(n, l)| (n + 1, l)) {
        let t = line.trim();
        let blank = t.is_empty() || t.starts_with('#');
        let inside = place.inside();
        if blank && !inside {
            continue;
        }
        let get = |k: &str| vars.get(k).cloned().or_else(|| (!unknown.contains(k)).then(|| env(k).unwrap_or_default()));
        let found = if inside { None } else { variable(t, &get, home) };
        match found {
            Some((k, v)) => drop(vars.insert(k, v)),
            None => {
                skipped.extend((!blank).then_some(n));
                // ENV-8: zendo does not know the variables that the line can set. A line outside a
                // block that only sets variables gives them new values for sure: the old ones go.
                let gone = if inside { Vec::new() } else { reassigned(t) };
                gone.iter().for_each(|name| drop(vars.remove(name)));
                unknown.extend(set_by(t));
                place.pass(line);
            }
        }
    }
    (vars, skipped)
}

/// The message about the lines of `.envrc` that zendo skipped, if there are any (ENV-7). A run of
/// lines is its first and its last.
pub(super) fn skipped_problem(path: &Path, skipped: &[usize]) -> Option<String> {
    let run = |r: &[usize]| if r.len() == 1 { r[0].to_string() } else { format!("{}-{}", r[0], r[r.len() - 1]) };
    let runs: Vec<String> = skipped.chunk_by(|a, b| a + 1 == *b).map(run).collect();
    let word = if skipped.len() == 1 { "line" } else { "lines" };
    (!runs.is_empty()).then(|| format!("{}: zendo skipped {word} {}: it reads only `NAME=value` lines", path.display(), runs.join(", ")))
}

/// The line without the word `export` before a variable.
fn unexported(line: &str) -> &str {
    line.strip_prefix("export").filter(|r| r.starts_with([' ', '\t'])).map_or(line, str::trim_start)
}

/// The variables that the line gives a new value for sure: a line of `NAME=value` words only, also
/// after a word of `DECLARE` and its options. `NAME=value command` sets none: the value is for
/// the command.
fn reassigned(line: &str) -> Vec<String> {
    let (bare, _) = scan(line, None);
    let mut depth = 0usize;
    let apart = |c: char| {
        depth = (depth + usize::from(c == '(')).saturating_sub(usize::from(c == ')'));
        depth == 0 && c.is_whitespace()
    };
    let words: Vec<&str> = bare.split(apart).filter(|w| !w.is_empty()).skip_while(|w| DECLARE.contains(w) || w.starts_with('-')).collect();
    let name = |w: &&str| w.split_once('=').map(|(n, _)| n.trim_end_matches('+')).filter(|n| is_name(n)).map(str::to_string);
    let names: Vec<String> = words.iter().filter_map(name).collect();
    if names.len() == words.len() { names } else { Vec::new() }
}

/// The variables that the line can set: a name before `=`, `+=` or `:=`, the names after `read`,
/// and the name after `for`.
fn set_by(line: &str) -> Vec<String> {
    let before = |(i, _): (usize, &str)| {
        let head: Vec<char> = line[..i].trim_end_matches([':', '+']).chars().rev().take_while(|c| c.is_ascii_alphanumeric() || *c == '_').collect();
        Some(head.into_iter().rev().collect::<String>()).filter(|n| is_name(n))
    };
    let words: Vec<&str> = line.split_whitespace().collect();
    let after = |word: &str| {
        let rest = words.iter().position(|w| *w == word).map_or(&[][..], |i| &words[i + 1..]);
        rest.iter().skip_while(|w| w.starts_with('-')).take_while(|w| is_name(w)).map(|w| w.to_string()).collect::<Vec<_>>()
    };
    line.match_indices('=').filter_map(before).chain(after("read")).chain(after("for").into_iter().take(1)).collect()
}

/// `export NAME=value` or `NAME=value`, with nothing after the value but a comment. `get` gives
/// the value of a variable for `$NAME`: None for a variable that zendo does not know.
fn variable(line: &str, get: &dyn Fn(&str) -> Option<String>, home: &Path) -> Option<(String, String)> {
    let (name, rest) = unexported(line).split_once('=')?;
    is_name(name).then_some(())?;
    let (value, tail) = value(rest, get, home)?;
    let comment = tail.trim_start();
    let ends = tail.is_empty() || (comment.len() < tail.len() && (comment.is_empty() || comment.starts_with('#')));
    ends.then(|| (name.to_string(), value))
}

fn is_name(name: &str) -> bool {
    let first = name.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_');
    first && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// The value at the start of `rest` and the text after it: a text in single quotes as it is, a
/// text in double quotes or a word without a char of the shell with its `$NAME` (ENV-8).
fn value<'a>(rest: &'a str, get: &dyn Fn(&str) -> Option<String>, home: &Path) -> Option<(String, &'a str)> {
    if let Some(inner) = rest.strip_prefix('\'') {
        let end = inner.find('\'')?;
        return Some((inner[..end].to_string(), &inner[end + 1..]));
    }
    if let Some(inner) = rest.strip_prefix('"') {
        let mut escaped = false;
        let closes = |&(_, c): &(usize, char)| {
            let end = c == '"' && !escaped;
            escaped = c == '\\' && !escaped;
            end
        };
        let end = inner.char_indices().find(closes)?.0;
        return Some((expand(&inner[..end], get, true)?, &inner[end + 1..]));
    }
    let (word, tail) = rest.split_at(rest.find(char::is_whitespace).unwrap_or(rest.len()));
    let plain = !word.contains(|c| "'\"`;|&()<>\\".contains(c));
    plain.then(|| tilde(word, home)).flatten().and_then(|w| expand(&w, get, false)).map(|v| (v, tail))
}

/// `~` and `~/…` at the start of a word and after a `:`, as the shell reads an assignment. None
/// for `~name`, which needs the shell.
fn tilde(word: &str, home: &Path) -> Option<String> {
    let home = home.display().to_string();
    let part = |p: &str| match p.strip_prefix('~') {
        None => Some(p.to_string()),
        Some(r) if r.is_empty() || r.starts_with('/') => Some(format!("{home}{r}")),
        Some(_) => None,
    };
    word.split(':').map(part).collect::<Option<Vec<_>>>().map(|parts| parts.join(":"))
}

/// `text` with the values of its `$NAME` and `${NAME}`; None when it needs a shell. `quoted`: the
/// text stood in double quotes, where `\` takes the special meaning from the next char.
fn expand(text: &str, get: &dyn Fn(&str) -> Option<String>, quoted: bool) -> Option<String> {
    let mut out = String::new();
    let mut rest = text;
    while let Some(c) = rest.chars().next() {
        rest = &rest[c.len_utf8()..];
        match (c, rest.chars().next()) {
            ('`', _) => return None,
            ('\\', Some(n)) if quoted && "\"\\$`".contains(n) => {
                out.push(n);
                rest = &rest[1..];
            }
            ('$', Some(_)) => {
                let (value, after) = reference(rest, get)?;
                out.push_str(&value);
                rest = after;
            }
            _ => out.push(c),
        }
    }
    Some(out)
}

/// The value of the variable whose name starts `rest`, the text after a `$`, and the text after
/// the name. None for what is no plain name: `$(…)`, `${NAME:-x}`, `$1`, and for a variable that
/// zendo does not know.
fn reference<'a>(rest: &'a str, get: &dyn Fn(&str) -> Option<String>) -> Option<(String, &'a str)> {
    let (name, after) = match rest.strip_prefix('{') {
        Some(braced) => braced.split_once('}')?,
        None => rest.split_at(rest.find(|c: char| !(c.is_ascii_alphanumeric() || c == '_')).unwrap_or(rest.len())),
    };
    is_name(name).then(|| get(name)).flatten().map(|value| (value, after))
}

/// The code of `line` outside its quotes and before its comment, with one `"` in place of each
/// text in quotes, and the quote that is open at the end of the line. `start`: the quote that was
/// open at the start of the line.
fn scan(line: &str, start: Option<char>) -> (String, Option<char>) {
    let mut quote = start;
    let mut bare = String::new();
    let mut prev = ' ';
    let mut chars = line.chars();
    while let Some(c) = chars.next() {
        match (quote, c) {
            (None, '#') if prev.is_whitespace() => break,
            (None, '\\') => bare.extend([Some(c), chars.next()].into_iter().flatten()),
            (Some('"' | '`'), '\\') => drop(chars.next()),
            (None, '\'' | '"' | '`') => {
                quote = Some(c);
                bare.push('"');
            }
            (Some(q), c) if c == q => quote = None,
            (None, c) => bare.push(c),
            _ => {}
        }
        prev = c;
    }
    (bare, quote)
}

/// The blocks that one command of a line opens, less the blocks that it closes. A brace at the
/// end opens the body of a function: `f() {`, `f(){`, `function f {`.
fn command(text: &str) -> isize {
    let words: Vec<&str> = text.split_whitespace().skip_while(|w| LEAD.contains(w)).collect();
    let (Some(first), Some(last)) = (words.first(), words.last()) else { return 0 };
    let opens = OPEN.contains(first) || last.ends_with('{');
    let closes = CLOSE.contains(first) || *first == "}";
    opens as isize - closes as isize
}

/// The end word of the here-document that `line` starts.
fn heredoc(line: &str) -> Option<String> {
    let after = &line[line.find("<<")? + 2..];
    let after = after.strip_prefix('-').unwrap_or(after);
    let word: String = after.trim_start().chars().take_while(|c| !c.is_whitespace() && !";)|&<".contains(*c)).filter(|c| !"'\"\\".contains(*c)).collect();
    (!word.is_empty()).then_some(word)
}

#[cfg(test)]
#[path = "envrc_tests.rs"]
mod tests;
