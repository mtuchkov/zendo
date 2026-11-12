//! `f3` a new file and `f4` a new folder, from the file tree. The field starts with the folder of the
//! selection; `tab` completes a folder name.
use super::*;

impl App {
    /// The folder of the selection in the tree, with a `/` at the end; empty at the root.
    fn suggested_folder(&self) -> String {
        let rows = self.tree.rows();
        let Some(r) = rows.get(self.tree.sel) else { return String::new() };
        let dir = match r.kind {
            crate::docs::tree::Kind::Dir { .. } => r.rel.clone(),
            crate::docs::tree::Kind::File => r.rel.parent().map(Path::to_path_buf).unwrap_or_default(),
        };
        let s = dir.display().to_string();
        if s.is_empty() { s } else { format!("{s}/") }
    }

    /// `f3` or `f4` in the tree.
    pub(crate) fn ask_new(&mut self, folder: bool) {
        let at = self.suggested_folder();
        self.file_prompt = Some(if folder { FilePrompt::NewFolder(at) } else { FilePrompt::NewFile(at) });
    }

    /// The best folder for the last part of a typed path: `docs/zen` → `docs/zendo/`.
    pub fn complete_folder(&self, text: &str) -> Option<String> {
        let (parent, part) = text.rsplit_once('/').unwrap_or(("", text));
        if part.is_empty() {
            return None;
        }
        let names: Vec<PathBuf> = std::fs::read_dir(self.cfg.root.join(parent))
            .ok()?
            .flatten()
            .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
            .map(|e| PathBuf::from(e.file_name()))
            .filter(|n| !n.to_string_lossy().starts_with('.'))
            .collect();
        let best = at::rank(&names, part).into_iter().next()?;
        let best = best.display();
        Some(if parent.is_empty() { format!("{best}/") } else { format!("{parent}/{best}/") })
    }

    /// A path of the repository from a typed text, or why not.
    fn inside(&self, text: &str) -> Result<PathBuf, String> {
        let t = text.trim().trim_start_matches('/');
        if t.is_empty() || t.split('/').any(|p| p == "..") {
            return Err("give a path inside the repository".into());
        }
        Ok(self.cfg.root.join(t))
    }

    /// A new file; a name without an extension is a markdown note with its title. It opens to write.
    pub(crate) fn make_file(&mut self, text: &str) {
        let path = match self.inside(text) {
            Ok(p) if text.ends_with('/') => return self.flash(format!("give a file name after {}", p.display())),
            Ok(p) => p,
            Err(e) => return self.flash(e),
        };
        let path = if path.extension().is_none() { path.with_extension("md") } else { path };
        if path.exists() {
            return self.flash("a file with that name exists");
        }
        let body = self.new_text(&path);
        if let Err(e) = path.parent().map_or(Ok(()), std::fs::create_dir_all).and_then(|_| std::fs::write(&path, body)) {
            return self.flash(format!("could not make the file: {e}"));
        }
        self.reload();
        self.scan_tree();
        let rel = path.strip_prefix(&self.cfg.root).unwrap_or(&path).to_path_buf();
        self.tree.reveal(&rel);
        self.open_file(path);
        self.focus = Focus::Note;
        self.start_write(false);
    }

    /// A new folder. The folders view shows it, also while it is empty.
    pub(crate) fn make_folder(&mut self, text: &str) {
        let path = match self.inside(text) {
            Ok(p) => p,
            Err(e) => return self.flash(e),
        };
        if let Err(e) = std::fs::create_dir_all(&path) {
            return self.flash(format!("could not make the folder: {e}"));
        }
        self.tree.view = crate::docs::tree::View::Folders;
        self.scan_tree();
        let rel = path.strip_prefix(&self.cfg.root).unwrap_or(&path).to_path_buf();
        self.tree.reveal(&rel);
        self.flash(format!("made {}/ · the folders view shows it", rel.display()));
    }

    /// The first text of a new file: the template of its folder with the title in it, a title, or
    /// nothing for a file that is not markdown.
    fn new_text(&self, path: &Path) -> String {
        if path.extension().is_none_or(|e| e != "md") {
            return String::new();
        }
        let title = title_of_name(path);
        let rel = path.strip_prefix(&self.cfg.root).unwrap_or(path);
        let template = self.tree.structure.as_ref().and_then(|st| st.template_for(rel)).and_then(|t| std::fs::read_to_string(self.cfg.root.join(t)).ok());
        let Some(t) = template else { return format!("# {title}\n\n") };
        // The first heading of the template becomes the title.
        match t.lines().position(|l| l.starts_with("# ")) {
            Some(h) => t.lines().enumerate().map(|(i, l)| if i == h { format!("# {title}") } else { l.to_string() }).collect::<Vec<_>>().join("\n") + "\n",
            None => format!("# {title}\n\n{t}"),
        }
    }
}

/// A title from a file name: `audit-log.md` → `Audit log`.
fn title_of_name(path: &Path) -> String {
    let words = path.file_stem().map(|s| s.to_string_lossy().replace(['-', '_'], " ")).unwrap_or_default();
    let mut cs = words.chars();
    cs.next().map(|c| c.to_uppercase().chain(cs).collect()).unwrap_or_default()
}
