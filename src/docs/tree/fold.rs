//! Folding: one folder or main material, all of them, and the folders above a file. Every folder
//! and main material starts closed (FILES-56).
use super::*;

impl Tree {
    /// Open every folder on the way to `rel` and select it.
    pub fn reveal(&mut self, rel: &Path) {
        // In the structure view, open the most specific main material that holds the file.
        if let Some(p) = self.section_of(rel) {
            self.opened.insert(p);
        }
        self.opened.extend(folders_above(rel));
        if let Some(i) = self.rows().iter().position(|r| r.rel == rel) {
            self.sel = i;
        }
        self.reveal_later(rel);
    }

    /// Open or close a folder or a main material (FILES-5).
    pub fn toggle(&mut self, rel: &Path) {
        if !self.opened.remove(rel) {
            self.opened.insert(rel.to_path_buf());
        }
    }

    /// `[+]` in the head of the tree: every folder and main material opens (FILES-57).
    pub fn expand_all(&mut self) {
        let mut open: Vec<PathBuf> = self.files.iter().flat_map(|f| folders_above(f)).collect();
        open.extend(self.dirs.iter().cloned());
        open.extend(self.structure.iter().flat_map(|s| s.main().map(section_id)));
        if let Some(d) = &self.notes {
            open.push(d.clone());
            open.extend(self.notes_files.iter().flat_map(|f| folders_above(&d.join(f)).filter(|p| p.starts_with(d)).collect::<Vec<_>>()));
        }
        self.opened.extend(open);
    }

    /// `[-]` in the head of the tree: every folder and main material closes, and the selection goes
    /// to the top (FILES-57).
    pub fn collapse_all(&mut self) {
        self.opened.clear();
        (self.sel, self.scroll) = (0, 0);
    }
}

/// The folders that hold `rel`, from its parent up; without the root.
fn folders_above(rel: &Path) -> impl Iterator<Item = PathBuf> + '_ {
    rel.ancestors().skip(1).filter(|d| !d.as_os_str().is_empty()).map(Path::to_path_buf)
}
