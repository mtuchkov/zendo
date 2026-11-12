//! AGENTS.md, Project structure: `project-structure.toml` of this repository passes the checks of
//! `zendo doctor` (STRUCT-5, STRUCT-13, STRUCT-14).

use std::path::Path;
use zendo::docs::structure::check;

#[test]
fn the_project_structure_of_this_repository_passes_the_doctor_checks() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let text = std::fs::read_to_string(root.join("project-structure.toml")).unwrap();
    let report = check(&text, root);
    let issues: Vec<String> = report.issues.iter().map(|i| format!("{}: {}", i.at, i.what)).collect();
    assert!(report.parsed.is_ok(), "project-structure.toml does not parse: {:?}", report.parsed.err());
    assert!(issues.is_empty(), "project-structure.toml has problems:\n{}", issues.join("\n"));
}
