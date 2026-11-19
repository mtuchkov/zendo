//! The file suggestions of the chat input, joined with the files and the chat of the app.
use super::*;

impl App {
    /// After a key in the chat input: open, update or close the file suggestions.
    pub(crate) fn update_at(&mut self) {
        let named: Vec<PathBuf> = self.files.structure().map(|st| st.templates().map(|(t, _)| PathBuf::from(t)).collect()).unwrap_or_default();
        let all = self.facts.files.get(&self.cfg.root);
        self.chat.composer_mut(self.state.session).update_at(all, &named);
    }

    /// A key of the open file suggestions. A file that the user takes goes into the chat as an
    /// attachment; a template brings how to use it. True when the key was used.
    pub(crate) fn key_at(&mut self, k: KeyEvent) -> bool {
        let (rel, templates) = match self.composer_mut().key_at(k) {
            AtKey::Unused => return false,
            AtKey::Used => return true,
            AtKey::Taken(rel, templates) => (rel, templates),
        };
        if !templates {
            self.attach(Attachment::of(self.cfg.root.join(rel)));
            return true;
        }
        let ctx = self.template_context(&rel);
        let file = Attachment::of(self.cfg.root.join(&rel));
        let label = self.composer_mut().add_chip(chips::file_label(&rel), Some(ctx), Some(file), false);
        self.flash(format!("template {}", label.trim_matches([chips::OPEN, chips::CLOSE])));
        true
    }

    /// `ctrl-t` in the chat: the suggestions show only templates.
    pub(crate) fn pick_template(&mut self) {
        self.composer_mut().pick_template();
        self.update_at();
    }

    /// What the agent gets with a template: where it is used, and how to use it.
    fn template_context(&self, rel: &Path) -> String {
        let t = rel.display().to_string();
        let used: Vec<&str> = self.files.structure().map(|st| st.templates().filter(|(x, _)| *x == t).map(|(_, at)| at).collect()).unwrap_or_default();
        let place = if used.is_empty() { String::new() } else { format!(" project-structure.toml uses it for new documents in: {}.", used.join(", ")) };
        format!("Template: {t}.{place} Write the new document from this template: keep its front matter, its headings and their order, and fill them from what the user gives (notes, files, the chat). Put the new file where project-structure.toml says, and propose it as an edit.")
    }
}
