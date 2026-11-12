//! Templates of project-structure.toml: a new file starts from the template of its folder, and
//! `ctrl-t` in the chat adds a template for the agent.

mod common;
use common::*;
use ratatui::crossterm::event::KeyCode;
use zendo::app::Focus;

const STRUCTURE: &str = "[project]\nname = \"t\"\n[[entry]]\npath = \"docs/specs\"\nmain = true\ntemplate = \"docs/templates/spec.md\"\n";
const TEMPLATE: &str = "---\nstatus: draft\n---\n\n# Title of the specification\n\n## Requirements\n";

fn files() -> Vec<(&'static str, &'static str)> {
    vec![
        ("project-structure.toml", STRUCTURE),
        ("docs/templates/spec.md", TEMPLATE),
        ("docs/templates/adr.md", "# Decision\n"),
        ("docs/specs/old.md", "# Old\n"),
        ("docs/notes/2026-11-12-idea.md", "# Idea\n"),
    ]
}

// FILES-19: f3 in a folder with a template: the new file starts as the template, with its title.
#[test]
fn a_new_file_starts_from_the_template_of_its_folder() {
    let root = repo("tmpl-new", &files());
    let mut a = app(&root);
    a.toggle_tree();
    a.focus = Focus::Tree;
    draw(&mut a, 120, 30);
    let row = a.tree.rows().iter().position(|r| r.rel.ends_with("docs/specs")).expect("the specs row");
    a.tree.sel = row;
    key(&mut a, KeyCode::F(3));
    typ(&mut a, "audit-log");
    key(&mut a, KeyCode::Enter);
    let text = std::fs::read_to_string(root.join("docs/specs/audit-log.md")).unwrap();
    assert!(text.starts_with("---\nstatus: draft\n---"), "{text}");
    assert!(text.contains("# Audit log\n") && text.contains("## Requirements"), "{text}");
    let _ = std::fs::remove_dir_all(&root);
}

// CLAUDE-25: ctrl-t in the chat suggests only templates; tab adds one as a chip with its use.
#[tokio::test]
async fn ctrl_t_suggests_only_templates() {
    let root = repo("tmpl-pick", &files());
    let mut a = app(&root);
    a.open_panel();
    a.focus = Focus::Claude;
    ctrl(&mut a, 't');
    let hits: Vec<String> = a.at.as_ref().expect("the suggestions").hits.iter().map(|p| p.display().to_string()).collect();
    assert_eq!(hits.len(), 2, "{hits:?}");
    assert!(hits.iter().all(|h| h.starts_with("docs/templates/")), "{hits:?}");
    typ(&mut a, "spec");
    key(&mut a, KeyCode::Tab);
    assert!(a.at.is_none());
    let chip = a.chips.last().expect("a chip");
    let ctx = chip.context.as_deref().unwrap_or_default();
    assert!(ctx.contains("docs/templates/spec.md") && ctx.contains("docs/specs"), "{ctx}");
    assert!(chip.attach.is_some());
    let _ = std::fs::remove_dir_all(&root);
}
