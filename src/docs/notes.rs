//! Notes are plain markdown files in one folder of the repo.
//! The first `# ` line is the title. The file name starts with the date.

use anyhow::{Context, Result};
use chrono::{DateTime, Local, NaiveDate};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Clone, Debug)]
pub struct Note {
    pub path: PathBuf,
    pub title: String,
    pub created: Option<NaiveDate>,
    pub modified: DateTime<Local>,
    pub body: String,
}

impl Note {
    pub fn stem(&self) -> String {
        self.path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default()
    }

    /// The link name: the file stem without its date prefix.
    pub fn link_name(&self) -> String {
        strip_date(&self.stem()).to_string()
    }

    pub fn tags(&self) -> Vec<String> {
        tags_in(&self.body)
    }

    pub fn words(&self) -> usize {
        self.body.split_whitespace().count()
    }

    /// True when this note has a `[[name]]` link to `other`.
    pub fn links_to(&self, other: &Note) -> bool {
        let a = format!("[[{}]]", other.link_name());
        let b = format!("[[{}]]", other.stem());
        self.path != other.path && (self.body.contains(&a) || self.body.contains(&b))
    }
}

#[derive(Default)]
pub struct Store {
    pub root: PathBuf,
    pub dir: PathBuf,
}

impl Store {
    pub fn new(root: PathBuf, dir: PathBuf) -> Self {
        Self { root, dir }
    }

    pub fn rel<'a>(&self, path: &'a Path) -> &'a Path {
        path.strip_prefix(&self.root).unwrap_or(path)
    }

    /// The notes in the folder. A missing folder is no error: zendo creates it with the first note.
    pub fn load_all(&self) -> Result<Vec<Note>> {
        let mut notes = Vec::new();
        let dir = match fs::read_dir(&self.dir) {
            Ok(d) => d,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(notes),
            Err(e) => return Err(e).with_context(|| format!("cannot read {}", self.dir.display())),
        };
        for entry in dir {
            let path = entry?.path();
            if path.extension().is_some_and(|e| e == "md")
                && let Ok(note) = load(&path) {
                    notes.push(note);
                }
        }
        notes.sort_by_key(|b| std::cmp::Reverse(b.modified));
        Ok(notes)
    }

    /// A free path for a new note: `<dir>/<date>-<slug>.md`.
    pub fn path_for(&self, title: &str, date: NaiveDate) -> PathBuf {
        let slug = slug(title);
        let slug = if slug.is_empty() { "note".to_string() } else { slug };
        let base = format!("{}-{}", date.format("%Y-%m-%d"), slug);
        let mut path = self.dir.join(format!("{base}.md"));
        let mut n = 2;
        while path.exists() {
            path = self.dir.join(format!("{base}-{n}.md"));
            n += 1;
        }
        path
    }
}

pub fn load(path: &Path) -> Result<Note> {
    let body = fs::read_to_string(path)?;
    let modified: DateTime<Local> = fs::metadata(path)
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::now())
        .into();
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let created = stem
        .get(..10)
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    // Only markdown has a `# ` title; other files use their name.
    let is_md = path.extension().is_some_and(|e| e == "md");
    let title = if is_md { title_of(&body) } else { None }
        .unwrap_or_else(|| if is_md { strip_date(&stem).replace('-', " ") } else { path.file_name().unwrap_or_default().to_string_lossy().into_owned() });
    Ok(Note { path: path.to_path_buf(), title, created, modified, body })
}

pub fn save(path: &Path, body: &str) -> Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("md.tmp");
    fs::write(&tmp, body)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

pub fn title_of(body: &str) -> Option<String> {
    body.lines()
        .find(|l| !l.trim().is_empty())
        .and_then(|l| l.trim().strip_prefix("# "))
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
}

pub fn slug(title: &str) -> String {
    let mut out = String::new();
    for c in title.chars() {
        if c.is_alphanumeric() {
            out.extend(c.to_lowercase());
        } else if !out.ends_with('-') && !out.is_empty() {
            out.push('-');
        }
    }
    let out = out.trim_end_matches('-');
    out.chars().take(60).collect::<String>().trim_end_matches('-').to_string()
}

pub fn strip_date(stem: &str) -> &str {
    let b = stem.as_bytes();
    let is_date = b.len() > 11
        && b[..10].iter().enumerate().all(|(i, c)| if i == 4 || i == 7 { *c == b'-' } else { c.is_ascii_digit() })
        && b[10] == b'-';
    if is_date { &stem[11..] } else { stem }
}

/// Tags are `#word` tokens outside headings and code fences.
pub fn tags_in(body: &str) -> Vec<String> {
    let mut tags: Vec<String> = Vec::new();
    let mut fence = false;
    for line in body.lines() {
        let t = line.trim_start();
        if t.starts_with("```") {
            fence = !fence;
            continue;
        }
        if fence || t.starts_with('#') && t.trim_start_matches('#').starts_with(' ') {
            continue;
        }
        for word in line.split(|c: char| c.is_whitespace() || c == ',' || c == '(' || c == ')') {
            if let Some(tag) = word.strip_prefix('#') {
                let tag: String = tag
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
                    .collect();
                if !tag.is_empty() && !tag.chars().all(|c| c.is_ascii_digit()) && !tags.contains(&tag) {
                    tags.push(tag);
                }
            }
        }
    }
    tags
}

pub fn git_branch(root: &Path) -> Option<String> {
    let head = fs::read_to_string(root.join(".git/HEAD")).ok()?;
    let head = head.trim();
    Some(match head.strip_prefix("ref: refs/heads/") {
        Some(b) => b.to_string(),
        None => head.chars().take(7).collect(),
    })
}

#[cfg(test)]
#[path = "notes_tests.rs"]
mod tests;
