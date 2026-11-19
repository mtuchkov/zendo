//! `f3` a new file and `f4` a new folder, from the file tree. The field starts with the folder of the
//! selection; `tab` completes a folder name.
use super::{Effect, FilePrompt, Files};
use crate::app::IoJob;
use crate::docs::tree::{Kind, View};
use std::path::{Path, PathBuf};

impl Files {
    /// The folder of the selection in the tree, with a `/` at the end; empty at the root.
    fn suggested_folder(&self) -> String {
        let rows = self.tree.rows();
        let Some(r) = rows.get(self.tree.sel) else { return String::new() };
        let dir = match r.kind {
            Kind::Dir { .. } => r.rel.clone(),
            Kind::File | Kind::Head => r.rel.parent().map(Path::to_path_buf).unwrap_or_default(),
        };
        let s = dir.display().to_string();
        if s.is_empty() { s } else { format!("{s}/") }
    }

    /// `f3` or `f4` in the tree.
    pub fn ask_new(&mut self, folder: bool) {
        let at = self.suggested_folder();
        self.prompt = Some(if folder { FilePrompt::NewFolder(at.into()) } else { FilePrompt::NewFile(at.into()) });
    }

    /// A new file; a name without an extension is a markdown note with its title.
    pub(super) fn make_file(&mut self, root: &Path, text: &str) -> Option<Effect> {
        let path = match inside(root, text) {
            Ok(p) if text.ends_with('/') => return Effect::flash(format!("give a file name after {}", p.display())),
            Ok(p) => p,
            Err(e) => return Effect::flash(e),
        };
        let path = if path.extension().is_none() { path.with_extension("md") } else { path };
        let (title, template) = self.new_text(root, &path);
        // The file is made on a thread; its answer opens it (IO-1).
        Some(Effect::Io(IoJob::new(move || match crate::docs::disk::make_file(&path, template.as_deref(), |t| first_text(title, t)) {
            Ok(true) => Some(Effect::MadeFile(path)),
            Ok(false) => Effect::flash("a file with that name exists"),
            Err(e) => Effect::flash(format!("could not make the file: {e}")),
        })))
    }

    /// A new folder. The text view shows it, also while it is empty (FILES-17).
    pub(super) fn make_folder(&mut self, root: &Path, text: &str) -> Option<Effect> {
        let path = match inside(root, text) {
            Ok(p) => p,
            Err(e) => return Effect::flash(e),
        };
        // A worktree view stays: its files show the new folder when a file is in it.
        if !self.tree.view.is_worktree() {
            self.tree.view = View::Text;
        }
        Some(Effect::Io(IoJob::new(move || match crate::docs::disk::make_folder(&path) {
            Ok(()) => Some(Effect::MadeFolder(path)),
            Err(e) => Effect::flash(format!("could not make the folder: {e}")),
        })))
    }

    /// The title of a new markdown file, and the template of its folder; None for a file that is
    /// not markdown.
    fn new_text(&self, root: &Path, path: &Path) -> (Option<String>, Option<PathBuf>) {
        if path.extension().is_none_or(|e| e != "md") {
            return (None, None);
        }
        let rel = path.strip_prefix(root).unwrap_or(path);
        (Some(title_of_name(path)), self.tree.structure.as_ref().and_then(|st| st.template_for(rel)).map(|t| root.join(t)))
    }
}

/// The first text of a new file: the template with the title in it, a title, or nothing for a file
/// that is not markdown.
fn first_text(title: Option<String>, template: Option<String>) -> String {
    let Some(title) = title else { return String::new() };
    let Some(t) = template else { return format!("# {title}\n\n") };
    // The first heading of the template becomes the title.
    match t.lines().position(|l| l.starts_with("# ")) {
        Some(h) => t.lines().enumerate().map(|(i, l)| if i == h { format!("# {title}") } else { l.to_string() }).collect::<Vec<_>>().join("\n") + "\n",
        None => format!("# {title}\n\n{t}"),
    }
}

/// A path of the repository from a typed text, or why not.
fn inside(root: &Path, text: &str) -> Result<PathBuf, String> {
    let t = text.trim().trim_start_matches('/');
    if t.is_empty() || t.split('/').any(|p| p == "..") {
        return Err("give a path inside the repository".into());
    }
    Ok(root.join(t))
}

/// A title from a file name: `audit-log.md` → `Audit log`.
fn title_of_name(path: &Path) -> String {
    let words = path.file_stem().map(|s| s.to_string_lossy().replace(['-', '_'], " ")).unwrap_or_default();
    let mut cs = words.chars();
    cs.next().map(|c| c.to_uppercase().chain(cs).collect()).unwrap_or_default()
}
