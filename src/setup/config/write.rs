//! Save settings into a config of the user (docs/zendo/cli/cli.md, CMD-19): only the line of each
//! setting that changed changes, or one line comes in; every other line stays, also in a file with
//! an error, with CRLF line endings, or with bytes that are not UTF-8.

use super::UserConfig;
use super::read::{Block, blocks, line_key, line_path};
use std::ops::Range;

/// One setting that changed: its path, as `["notes", "machine"]`, and its new value; None removes
/// it.
struct Change {
    path: Vec<String>,
    value: Option<toml::Value>,
}

fn table(cfg: &UserConfig) -> toml::Table {
    toml::Table::try_from(cfg).unwrap_or_default()
}

/// A copy of `cfg`, to compare with after a change.
pub fn copy(cfg: &UserConfig) -> UserConfig {
    toml::Value::Table(table(cfg)).try_into().unwrap_or_default()
}

/// The settings that differ from `old` to `new`, down to the values in the tables.
fn changes(old: &toml::Table, new: &toml::Table, path: &[String], out: &mut Vec<Change>) {
    let empty = toml::Table::new();
    let keys: std::collections::BTreeSet<&String> = old.keys().chain(new.keys()).collect();
    for k in keys {
        let path = [path, std::slice::from_ref(k)].concat();
        match (old.get(k), new.get(k)) {
            (a, b) if a == b => {}
            (Some(toml::Value::Table(a)), Some(toml::Value::Table(b))) => changes(a, b, &path, out),
            (None, Some(toml::Value::Table(b))) => changes(&empty, b, &path, out),
            (_, b) => out.push(Change { path, value: b.cloned() }),
        }
    }
}

/// The file `bytes` with the settings that differ from `old` to `new` (CMD-19). A file without an
/// error changes through `toml_edit`, which keeps the comments and the format; a file with one
/// changes line by line.
pub fn edit_bytes(bytes: &[u8], old: &UserConfig, new: &UserConfig) -> Vec<u8> {
    let mut list = Vec::new();
    changes(&table(old), &table(new), &[], &mut list);
    if list.is_empty() {
        return bytes.to_vec();
    }
    let crlf = bytes.windows(2).any(|w| w == b"\r\n");
    let lf = if crlf { replace(bytes, b"\r\n", b"\n") } else { bytes.to_vec() };
    let doc = std::str::from_utf8(&lf).ok().and_then(|s| s.parse::<toml_edit::DocumentMut>().ok());
    let out = match doc {
        Some(doc) => edit_doc(doc, &list).into_bytes(),
        None => edit_lines(&lf, &list),
    };
    if crlf { replace(&out, b"\n", b"\r\n") } else { out }
}

fn replace(bytes: &[u8], from: &[u8], to: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len());
    let mut rest = bytes;
    while let Some(i) = rest.windows(from.len()).position(|w| w == from) {
        out.extend_from_slice(&rest[..i]);
        out.extend_from_slice(to);
        rest = &rest[i + from.len()..];
    }
    out.extend_from_slice(rest);
    out
}

/// A value as `toml_edit` writes it.
fn value(v: &toml::Value) -> Option<toml_edit::Value> {
    v.to_string().parse().ok()
}

fn edit_doc(mut doc: toml_edit::DocumentMut, list: &[Change]) -> String {
    for c in list {
        edit_table(doc.as_table_mut(), &c.path, c.value.as_ref().and_then(value));
    }
    doc.to_string()
}

/// Set the value at `path` in `t`, or remove it; a table on the way that is not there comes in.
fn edit_table(t: &mut dyn toml_edit::TableLike, path: &[String], new: Option<toml_edit::Value>) {
    let Some((first, rest)) = path.split_first() else { return };
    if rest.is_empty() {
        return set_value(t, first, new);
    }
    if new.is_none() && !t.contains_key(first) {
        return;
    }
    let mut table = toml_edit::Table::new();
    table.set_implicit(true);
    if let Some(next) = t.entry(first).or_insert(toml_edit::Item::Table(table)).as_table_like_mut() {
        edit_table(next, rest, new);
    }
}

fn set_value(t: &mut dyn toml_edit::TableLike, leaf: &str, new: Option<toml_edit::Value>) {
    let Some(new) = new else {
        t.remove(leaf);
        return;
    };
    match t.get_mut(leaf).and_then(toml_edit::Item::as_value_mut) {
        // The comment after the old value stays.
        Some(old) => {
            let decor = old.decor().clone();
            *old = new;
            *old.decor_mut() = decor;
        }
        None => drop(t.insert(leaf, toml_edit::Item::Value(new))),
    }
}

/// A key as TOML writes it: bare when it can be, else quoted with TOML escapes.
fn key(k: &str) -> String {
    toml_edit::Key::new(k).display_repr().into_owned()
}

/// A dotted name as TOML writes it: `notes.machine`.
fn dotted(path: &[String]) -> String {
    path.iter().map(|k| key(k)).collect::<Vec<_>>().join(".")
}

fn edit_lines(bytes: &[u8], list: &[Change]) -> Vec<u8> {
    let mut lines: Vec<Vec<u8>> = bytes.split(|b| *b == b'\n').map(<[u8]>::to_vec).collect();
    for c in list {
        apply(&mut lines, c);
    }
    lines.join(&b'\n')
}

/// The lines as text, and their blocks.
fn scan(lines: &[Vec<u8>]) -> (Vec<String>, Vec<Block>) {
    let text: Vec<String> = lines.iter().map(|l| String::from_utf8_lossy(l).into_owned()).collect();
    let refs: Vec<&str> = text.iter().map(String::as_str).collect();
    let blocks = blocks(&refs);
    (text, blocks)
}

fn apply(lines: &mut Vec<Vec<u8>>, c: &Change) {
    let Some((leaf, tables)) = c.path.split_last() else { return };
    let name = tables.join(".");
    let (text, blocks) = scan(lines);
    let top = blocks.iter().find(|b| b.name.is_none()).map_or(0..0, Block::body);
    // The last block of a table written twice: the one that zendo reads.
    let body = match name.is_empty() {
        true => Some(top.clone()),
        false => blocks.iter().rev().find(|b| b.name.as_deref() == Some(name.as_str())).map(Block::body),
    };
    // A dotted setting before the first table, as `notes.machine = "m"`, keeps its form.
    let full = c.path.join(".");
    let path_of = |i: &usize| line_path(&text[*i]);
    let dot = (!name.is_empty()).then(|| top.clone().find(|i| path_of(i).as_deref() == Some(full.as_str()))).flatten();
    // A new setting of a table that is dotted goes after its last dotted line.
    let sibling = (!name.is_empty()).then(|| top.clone().rev().find(|i| path_of(i).is_some_and(|p| p.starts_with(&format!("{name}."))))).flatten();
    let line = |v: &toml::Value| format!("{} = {v}", dotted(&c.path)).into_bytes();
    match (&c.value, body, dot.or(sibling)) {
        (None, body, _) => remove(lines, &text, &blocks, &c.path, body.into_iter().flatten().chain(dot)),
        (Some(v), _, Some(i)) if dot.is_some() => lines[i] = line(v),
        (Some(v), None, Some(i)) => lines.insert(i + 1, line(v)),
        (Some(v), Some(body), _) => set_in(lines, &text, body, leaf, &format!("{} = {v}", key(leaf))),
        (Some(v), None, None) => append(lines, &format!("[{}]\n{} = {v}\n", dotted(tables), key(leaf))),
    }
}

/// Set the line of `leaf` in `range` to `line`: the first line of it changes and later ones go; without
/// one, the line comes after the last line of the range that is not empty.
fn set_in(lines: &mut Vec<Vec<u8>>, text: &[String], range: Range<usize>, leaf: &str, line: &str) {
    let found: Vec<usize> = range.clone().filter(|&i| line_key(&text[i]).as_deref() == Some(leaf)).collect();
    let Some((&first, later)) = found.split_first() else {
        let after = range.clone().rev().find(|&i| !text[i].trim().is_empty()).map_or(range.start, |i| i + 1);
        return lines.insert(after, line.as_bytes().to_vec());
    };
    lines[first] = line.as_bytes().to_vec();
    later.iter().rev().for_each(|&i| drop(lines.remove(i)));
}

/// Remove the setting at `path`: its lines among `body`, the block of its table, and the blocks of the
/// tables in it. The comments before the next table stay with that table.
fn remove(lines: &mut Vec<Vec<u8>>, text: &[String], blocks: &[Block], path: &[String], body: impl Iterator<Item = usize>) {
    let name = path.join(".");
    let leaf = path.last().map(String::as_str);
    let own = |i: &usize| line_key(&text[*i]).as_deref() == leaf || line_path(&text[*i]).as_deref() == Some(name.as_str());
    let mut gone: Vec<usize> = body.filter(own).collect();
    let inside = |b: &&Block| b.name.as_deref().is_some_and(|n| n == name || n.starts_with(&format!("{name}.")));
    for b in blocks.iter().filter(inside) {
        let last = (b.start..b.end).rev().find(|&i| !text[i].trim().is_empty() && !text[i].trim_start().starts_with('#')).unwrap_or(b.start);
        let blank = (last + 1..b.end).take_while(|&i| text[i].trim().is_empty()).count();
        gone.extend(b.start..last + 1 + blank);
    }
    gone.sort_unstable();
    gone.dedup();
    gone.into_iter().rev().for_each(|i| drop(lines.remove(i)));
}

/// Add a table at the end of the file, after an empty line.
fn append(lines: &mut Vec<Vec<u8>>, table: &str) {
    while lines.last().is_some_and(|l| l.iter().all(u8::is_ascii_whitespace)) {
        lines.pop();
    }
    if !lines.is_empty() {
        lines.push(Vec::new());
    }
    lines.extend(table.lines().map(|l| l.as_bytes().to_vec()));
    lines.push(Vec::new());
}

#[cfg(test)]
#[path = "write_tests.rs"]
mod tests;
