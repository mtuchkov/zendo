//! The structure view: the main materials of `project-structure.toml`, in its order. A material
//! with one folder shows the folder as a tree; a material with more locations shows the files of
//! all of them as one flat list.
use super::*;
use crate::docs::structure::{Entry, path_covers};

/// The row of a main material: its first location.
pub fn section_id(e: &Entry) -> PathBuf {
    PathBuf::from(e.path().trim_end_matches('/'))
}

impl Tree {
    pub(super) fn section_rows(&self, st: &Structure) -> Vec<Row> {
        let mut out = Vec::new();
        for e in st.main() {
            let rel = section_id(e);
            if e.is_group() {
                out.extend(self.group_rows(e, rel));
                continue;
            }
            // A main material that is a file, of any kind (e.g. a .toml).
            if self.files.contains(&rel) || self.root.join(&rel).is_file() || rel.extension().is_some_and(|x| x == "md") {
                out.push(Row { rel, name: e.label(), depth: 0, kind: Kind::File, section: true });
                continue;
            }
            let open = self.opened.contains(&rel);
            out.push(Row { rel: rel.clone(), name: e.label(), depth: 0, kind: Kind::Dir { open }, section: true });
            if open {
                let stripped: Vec<PathBuf> = self.files.iter().filter(|f| f.starts_with(&rel) && **f != rel).filter_map(|f| f.strip_prefix(&rel).ok().map(Path::to_path_buf)).collect();
                out.extend(self.tree_rows(&stripped, &[], &rel, 1));
            }
        }
        out
    }

    /// A material with more locations: its row, and when it is open, the files of every location
    /// in one flat list, each with its path.
    fn group_rows(&self, e: &Entry, rel: PathBuf) -> Vec<Row> {
        let open = self.opened.contains(&rel);
        let mut out = vec![Row { rel, name: e.label(), depth: 0, kind: Kind::Dir { open }, section: true }];
        if open {
            out.extend(self.group_files(e).into_iter().map(|f| Row { name: f.display().to_string(), rel: f, depth: 1, kind: Kind::File, section: false }));
        }
        out
    }

    /// The files of every location of a material: the listed ones, and files that are locations.
    pub fn group_files(&self, e: &Entry) -> Vec<PathBuf> {
        let named = e.paths.iter().map(|p| PathBuf::from(p.trim_end_matches('/'))).filter(|p| self.root.join(p).is_file());
        let mut files: Vec<PathBuf> = self.files.iter().filter(|f| e.covers(f)).cloned().chain(named).collect();
        files.sort();
        files.dedup();
        files
    }

    /// The main material that holds `rel`: the most specific folder, or a group with a location
    /// that covers it.
    pub(super) fn section_of(&self, rel: &Path) -> Option<PathBuf> {
        let st = self.structured()?;
        let group = st.main().find(|e| e.is_group() && e.paths.iter().any(|p| path_covers(p, rel)));
        group.map(section_id).or_else(|| st.main().map(section_id).filter(|p| rel.starts_with(p) && p != rel).max_by_key(|p| p.components().count()))
    }
}
