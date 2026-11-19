//! The title of a new note in the float (docs/zendo/float-title.md): the name model writes it when
//! the body has three lines of text, and again when the float closes if the text changed since
//! (FLOAT-18 to FLOAT-23). A title of the user stays (FLOAT-21).
use super::*;
use crate::agent::api::Question;
use crate::docs::notes;
use chrono::DateTime;
use std::path::{Path, PathBuf};

/// The lines with text, or the chars of text, that a body needs for a title (FLOAT-18).
const TITLE_LINES: usize = 3;
const TITLE_CHARS: usize = 200;
/// The name of the throwaway session that writes titles.
const TITLE_KEY: &str = "title";

/// What the float knows of the title of its note.
#[derive(Clone, Debug, Default)]
pub struct Titling {
    /// The name model got the question while the float is open (FLOAT-23).
    pub asked: bool,
    /// The title of the agent, and the body of the note when the title went in.
    pub agent: Option<(String, String)>,
}

/// A question for the title of the note of the float that opened at `opened`. `old`: the title of
/// the agent that the answer replaces.
#[derive(Clone, Debug)]
pub struct TitleJob {
    pub opened: DateTime<Local>,
    pub old: Option<String>,
}

/// The title line of a note and its index: the first line with text, when it is `# …` or an
/// empty title, `#` (FLOAT-17).
fn title_line(text: &str) -> Option<(usize, &str)> {
    text.split('\n').enumerate().find(|(_, l)| !l.trim().is_empty()).filter(|(_, l)| l.trim() == "#" || l.starts_with("# "))
}

/// The title of a note: the text of its title line, when it has some.
pub fn title_of(text: &str) -> Option<&str> {
    title_line(text).map(|(_, l)| l.trim().trim_start_matches('#').trim()).filter(|t| !t.is_empty())
}

/// The body of a note: the text after the title line, without the blank lines around it.
pub fn body_of(text: &str) -> &str {
    let body = match title_line(text) {
        Some((i, _)) => text.splitn(i + 2, '\n').nth(i + 1).unwrap_or(""),
        None => text,
    };
    body.trim()
}

/// The body has enough text for a title: three lines with text, or 200 chars (FLOAT-18).
pub fn ripe(text: &str) -> bool {
    let lines: Vec<&str> = body_of(text).lines().filter(|l| !l.trim().is_empty()).collect();
    lines.len() >= TITLE_LINES || lines.iter().map(|l| l.chars().count()).sum::<usize>() >= TITLE_CHARS
}

/// The text with the title of the agent: in the empty title line, as the first line when the
/// text has no title line, or in place of `old` while the title is still `old`. None: the title
/// stays (FLOAT-18, FLOAT-21).
pub fn with_title(text: &str, title: &str, old: Option<&str>) -> Option<String> {
    let title = title.trim();
    let at = match (title_line(text), title_of(text)) {
        (None, _) => return Some(format!("# {title}\n\n{text}")),
        (Some((i, _)), None) => i,
        (Some((i, _)), Some(t)) if Some(t) == old && t != title => i,
        _ => return None,
    };
    let line = format!("# {title}");
    let lines: Vec<&str> = text.split('\n').enumerate().map(|(k, l)| if k == at { line.as_str() } else { l }).collect();
    Some(lines.join("\n"))
}

impl App {
    /// The float waits for a title, or a closed float does: the agent has to run.
    pub(super) fn wants_title(&self) -> bool {
        self.writer.titles_wait() || self.live_title().is_some()
    }

    /// The float with a body of three lines and no title, not asked yet (FLOAT-18, FLOAT-23).
    fn live_title(&self) -> Option<DateTime<Local>> {
        let fl = self.writer.float().filter(|f| !f.title.asked)?;
        let text = self.writer.editor().text();
        (title_of(&text).is_none() && ripe(&text)).then_some(fl.opened)
    }

    /// Ask the name model for the title of the float, or of a closed float. True when it asked.
    pub(super) fn title_tick(&mut self) -> bool {
        if let Some(opened) = self.live_title() {
            if let Some(fl) = self.writer.float_mut() {
                fl.title.asked = true;
            }
            let rel = self.writer.path().map_or_else(|| "new note".to_string(), |p| self.rel(p));
            let text = self.writer.editor().text();
            return self.send_title(TitleJob { opened, old: None }, rel, text);
        }
        let Some(job) = self.writer.next_title() else { return false };
        let Some(path) = self.writer.closed_note(job.opened).cloned() else { return true };
        let Ok(text) = std::fs::read_to_string(&path) else { return true };
        let rel = self.rel(&path);
        self.send_title(job, rel, text)
    }

    /// The question for a title: through the API of the agent, else in a throwaway session of the
    /// name model (FLOAT-18, NAMES-15, NAMES-16).
    fn send_title(&mut self, job: TitleJob, rel: String, text: String) -> bool {
        let prompt = META_PROMPT
            .replace("{{PARTS}}", "TITLE: <the title>\n")
            .replace("{{TODAY}}", &Local::now().format("%Y-%m-%d").to_string())
            .replace("{{FILE}}", &rel)
            .replace("{{TEXT}}", &text);
        if self.link.api().is_some() {
            self.link.ask_api(Question::Title { opened: job.opened, old: job.old }, prompt, None);
            return true;
        }
        let key = format!("{RECAP_PREFIX}{TITLE_KEY}");
        self.work.start(RecapJob { rel, hash: state::hash(&text), text, title: Some(job), ..RecapJob::default() });
        if let Some(model) = self.link.name_model() {
            self.link.send(Cmd::Model { key: key.clone(), model });
        }
        self.link.send(Cmd::Prompt { key, known: None, text: prompt, attach: vec![] });
        true
    }

    /// The float closes with its note in `path`: ask for a title when the note has none, or has the
    /// title of the agent and new text (FLOAT-19, FLOAT-21).
    pub(crate) fn title_on_close(&mut self, path: PathBuf) {
        let Some(fl) = self.writer.float() else { return };
        let opened = fl.opened;
        let text = self.writer.editor().text();
        let old = match (title_of(&text), &fl.title.agent) {
            (None, _) => None,
            (Some(t), Some((a, body))) if t == a && body_of(&text) != body.as_str() => Some(a.clone()),
            _ => return,
        };
        self.writer.float_closed(opened, path);
        // A question that runs for this float puts its title into the file.
        let in_api = self.link.api_asks().iter().any(|q| matches!(q, Question::Title { opened: o, .. } if *o == opened));
        let running = in_api || self.work.job().and_then(|j| j.title.as_ref()).is_some_and(|j| j.opened == opened);
        if self.cfg.offline || running {
            return;
        }
        self.writer.ask_title(TitleJob { opened, old });
        if self.link.api().is_none() {
            self.ensure_agent();
        }
    }

    /// The title turn ended in its throwaway session.
    pub(super) fn title_done(&mut self, job: RecapJob) {
        if let Some(tj) = job.title {
            self.titled(tj, &job.buf, job.failed);
        }
    }

    /// The answer for a title: the title goes into the float, or into the file of the closed float.
    pub(super) fn titled(&mut self, tj: TitleJob, answer: &str, failed: Option<String>) {
        let Some(title) = parse_meta(answer).0 else {
            if let Some(e) = failed {
                self.flash(format!("no title from the agent: {}", e.lines().next().unwrap_or("")));
            }
            return;
        };
        if self.writer.float().is_some_and(|f| f.opened == tj.opened) {
            self.title_into_float(&title, tj.old.as_deref());
        } else {
            self.title_into_file(&tj, &title);
        }
    }

    /// The title as the first line of the note in the float; the cursor stays on its char
    /// (FLOAT-18, FLOAT-20).
    fn title_into_float(&mut self, title: &str, old: Option<&str>) {
        let text = self.writer.editor().text();
        let Some(new) = with_title(&text, title, old) else { return };
        let shift = new.split('\n').count().saturating_sub(text.split('\n').count());
        let e = self.writer.editor_mut();
        let (row, col, anchor) = (e.row + shift, e.col, e.anchor.map(|(r, c)| (r + shift, c)));
        // The loglet stays: the title goes into the file with the next save (UNDO-16).
        e.set_text(&new);
        e.row = row.min(e.lines.len() - 1);
        e.col = col.min(e.lines[e.row].chars().count());
        e.anchor = anchor;
        e.dirty = true;
        if let Some(fl) = self.writer.float_mut() {
            fl.title.agent = Some((title.trim().to_string(), body_of(&new).to_string()));
        }
        self.save();
        // The file takes the name of the title after the save (IO-6).
        let title = title.to_string();
        self.when_saved(move |a| {
            if let Some(target) = a.writer.path().cloned().and_then(|p| a.name_by_title(&p, &title)) {
                a.writer.saved(target);
            }
        });
    }

    /// The title into the file of a closed float (FLOAT-19, FLOAT-20).
    fn title_into_file(&mut self, tj: &TitleJob, title: &str) {
        let Some(path) = self.writer.closed_note(tj.opened).cloned() else { return };
        let Ok(now) = std::fs::read_to_string(&path) else { return };
        let Some(new) = with_title(&now, title, tj.old.as_deref()) else { return };
        if let Err(e) = crate::atomic::write(&path, new) {
            self.flash(format!("cannot write the title into {}: {e}", path.display()));
            return;
        }
        let target = self.name_by_title(&path, title);
        if let Some(t) = &target {
            self.writer.closed_moved(tj.opened, t.clone());
        }
        self.reload();
        self.flash(format!("added a title to {}", self.rel(target.as_ref().unwrap_or(&path))));
    }

    /// Rename a note to its date and a short form of its title (FLOAT-20, NOTE-7). The new path,
    /// when the name changed.
    fn name_by_title(&mut self, path: &Path, title: &str) -> Option<PathBuf> {
        let dir = path.parent()?.to_path_buf();
        let stem = path.file_stem()?.to_string_lossy().into_owned();
        let date = stem.get(..10).and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok()).unwrap_or_else(|| Local::now().date_naive());
        let slug = Some(notes::slug(title)).filter(|s| !s.is_empty()).unwrap_or_else(|| "note".to_string());
        let want = dir.join(format!("{}-{slug}.md", date.format("%Y-%m-%d")));
        if want == path {
            return None;
        }
        let target = if want.exists() { notes::Store::new(self.cfg.root.clone(), dir).path_for(title, date) } else { want };
        self.move_file(path, &target).ok().map(|()| target)
    }
}

#[cfg(test)]
#[path = "title_tests.rs"]
mod tests;
