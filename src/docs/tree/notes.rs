//! The notes of the notes repo in the tree (FILES-25): a `notes` folder at the top of the structure
//! view, or of the docs view when the repository has no main materials. Its rows hold absolute
//! paths, so `root.join(rel)` opens the file in the notes repo.
use super::*;

impl Tree {
    /// The markdown files of the notes folder, relative to it; hidden files are skipped.
    pub(super) fn scan_notes(&mut self) {
        let Some(dir) = self.notes.clone() else { return self.notes_files.clear() };
        let walk = ignore::WalkBuilder::new(&dir).hidden(true).require_git(false).build();
        let mut files: Vec<PathBuf> = walk
            .flatten()
            .filter(|e| e.file_type().is_some_and(|t| t.is_file()) && e.path().extension().is_some_and(|x| x == "md"))
            .filter_map(|e| e.path().strip_prefix(&dir).ok().map(Path::to_path_buf))
            .collect();
        files.sort();
        self.notes_files = files;
    }

    /// The `notes` folder and, when it is open, its files; empty in the other views.
    pub(super) fn notes_rows(&self) -> Vec<Row> {
        let main = self.structure.as_ref().is_some_and(|s| s.main().next().is_some());
        let shown = self.view == View::Structure && main || self.view == View::Docs && !main;
        let Some(dir) = self.notes.as_ref().filter(|_| shown) else { return Vec::new() };
        let open = !self.closed.contains(dir);
        let mut out = vec![Row { rel: dir.clone(), name: "notes".into(), depth: 0, kind: Kind::Dir { open }, section: true }];
        if open {
            out.extend(self.tree_rows(&self.notes_files, &[], dir, 1));
        }
        out
    }
}
