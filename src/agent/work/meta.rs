//! The title and front matter that the agent adds when a file has none.
use super::*;

pub const META_PROMPT: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/resources/add-metadata.md"));

impl App {
    /// What a file needs from the agent: a title, front matter. Notes get no front matter.
    pub fn meta_needs(&self, path: &std::path::Path) -> Option<Meta> {
        if path.extension().is_none_or(|e| e != "md") {
            return None;
        }
        let text = std::fs::read_to_string(path).ok()?;
        if text.trim().is_empty() {
            return None;
        }
        let (front, body) = split_front(&text);
        let title = !body.lines().find(|l| !l.trim().is_empty()).is_some_and(|l| l.starts_with("# "));
        let note = path.starts_with(&self.store.dir);
        let meta = Meta { title, front: front.is_none() && !note };
        (meta.title || meta.front).then_some(meta)
    }

    /// After write mode: ask the agent in the background for what the file lacks.
    pub fn ask_for_meta(&mut self) {
        let Some(path) = self.current.clone().or_else(|| self.editor_path.clone()) else { return };
        if self.cfg.offline || self.meta_needs(&path).is_none() {
            return;
        }
        let rel = self.rel(&path);
        if !self.meta_queue.contains(&rel) {
            self.ensure_agent();
            self.meta_queue.push_back(rel);
        }
    }

    pub(super) fn meta_tick(&mut self) -> bool {
        let Some(rel) = self.meta_queue.pop_front() else { return false };
        let path = self.cfg.root.join(&rel);
        let Some(meta) = self.meta_needs(&path) else { return true };
        let Ok(text) = std::fs::read_to_string(&path) else { return true };
        let mut parts = String::new();
        if meta.title {
            parts.push_str("TITLE: <the title>\n");
        }
        if meta.front {
            parts.push_str("FRONT MATTER:\n<the YAML lines>\n");
        }
        let prompt = META_PROMPT
            .replace("{{PARTS}}", &parts)
            .replace("{{TODAY}}", &Local::now().format("%Y-%m-%d").to_string())
            .replace("{{FILE}}", &rel)
            .replace("{{TEXT}}", &text);
        let key = format!("{RECAP_PREFIX}meta {rel}");
        self.recap_job = Some(RecapJob { rel, hash: state::hash(&text), text, buf: String::new(), auto: true, failed: None, meta: Some(meta), review: false });
        if let Some(a) = &self.agent {
            a.send(Cmd::Prompt { key, known: None, text: prompt, attach: vec![] });
        }
        true
    }

    /// The metadata turn ended: put the title and the front matter into the file.
    pub(super) fn meta_done(&mut self, job: RecapJob) {
        let Some(meta) = job.meta else { return };
        let path = self.cfg.root.join(&job.rel);
        let Ok(now) = std::fs::read_to_string(&path) else { return };
        let (title, front) = parse_meta(&job.buf);
        let Some(new) = with_meta(&now, meta, title.as_deref(), front.as_deref()) else {
            if let Some(e) = job.failed {
                self.flash(format!("no title from the agent: {}", e.lines().next().unwrap_or("")));
            }
            return;
        };
        if std::fs::write(&path, &new).is_ok() {
            self.reload();
            let what = match (title.is_some() && meta.title, front.is_some() && meta.front) {
                (true, true) => "a title and front matter",
                (true, false) => "a title",
                _ => "front matter",
            };
            self.flash(format!("added {what} to {}", job.rel));
        }
    }
}

/// What a file gets from the agent after write mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Meta {
    pub title: bool,
    pub front: bool,
}

/// The front matter (without its `---` lines), and the rest.
pub fn split_front(text: &str) -> (Option<&str>, &str) {
    if let Some(rest) = text.strip_prefix("---\n")
        && let Some(end) = rest.find("\n---\n").map(|i| (i, i + 5)).or_else(|| rest.strip_suffix("\n---").map(|r| (r.len(), rest.len())))
    {
        return (Some(&rest[..end.0]), &rest[end.1..]);
    }
    (None, text)
}

/// The title and the front matter in the answer of the agent.
pub fn parse_meta(answer: &str) -> (Option<String>, Option<String>) {
    let mut title = None;
    let mut front: Option<Vec<String>> = None;
    for line in answer.lines() {
        let t = line.trim_end();
        if let Some(v) = t.trim_start().strip_prefix("TITLE:") {
            let v = v.trim().trim_matches(|c| c == '"' || c == '\'' || c == '#').trim().to_string();
            if !v.is_empty() {
                title = Some(v);
            }
            continue;
        }
        if t.trim_start().starts_with("FRONT MATTER:") {
            front = Some(Vec::new());
            continue;
        }
        if t.trim() == "END" {
            break;
        }
        if let Some(f) = front.as_mut()
            && !t.trim().is_empty()
            && t.trim() != "---"
            && !t.trim_start().starts_with("```")
        {
            f.push(t.to_string());
        }
    }
    (title, front.filter(|f| !f.is_empty()).map(|f| f.join("\n")))
}

/// The text with the title and the front matter in, when the file still lacks them. None: nothing to add.
pub fn with_meta(text: &str, meta: Meta, title: Option<&str>, front: Option<&str>) -> Option<String> {
    let (old_front, body) = split_front(text);
    let add_front = meta.front && old_front.is_none() && front.is_some();
    let has_title = body.lines().find(|l| !l.trim().is_empty()).is_some_and(|l| l.starts_with("# "));
    let add_title = meta.title && !has_title && title.is_some();
    if !add_front && !add_title {
        return None;
    }
    let mut out = String::new();
    match (add_front, old_front) {
        (true, _) => out.push_str(&format!("---\n{}\n---\n\n", front.expect("checked").trim())),
        (false, Some(f)) => out.push_str(&format!("---\n{f}\n---\n")),
        _ => {}
    }
    let mut body = if add_front || add_title { body.trim_start_matches('\n') } else { body };
    if add_title {
        let t = title.expect("checked");
        // A first line that is the title without `#` becomes the title.
        if let Some((first, rest)) = body.split_once('\n')
            && first.trim().eq_ignore_ascii_case(t.trim())
        {
            body = rest.trim_start_matches('\n');
        }
        out.push_str(&format!("# {t}\n\n"));
    }
    out.push_str(body);
    Some(out)
}
