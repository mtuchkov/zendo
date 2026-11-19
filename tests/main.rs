//! The integration tests: one binary, one module for each file of tests/. One link for all the
//! files makes target/ small and the build fast (Cargo.toml has `autotests = false`).
//! Run one file: `cargo test --test integration <file>::`.

mod common;

mod acp_mock;
mod activity;
mod attention;
mod background;
mod board;
mod box_keys;
mod branch;
mod chat_flows;
mod chat_history;
mod chat_queue;
mod chip_view;
mod cli_messages;
mod code_size;
mod doctor_fix;
mod find;
mod find_modes;
mod find_search;
mod find_selection;
mod float;
mod fonts_tip;
mod handoff;
mod help_keys;
mod interactivity;
mod key_change;
mod key_hints;
mod keymap_cli;
mod keymap_hints;
mod lofi;
mod mcp;
mod note_to_spec;
mod notes_repo;
mod pane_click;
mod pane_drag;
mod selection_keys;
mod session_input;
mod session_names;
mod sessions;
mod table_selection;
mod templates;
mod terminal;
mod terminal_mouse;
mod terminal_output;
mod text_input;
mod tips;
mod traceability;
mod tree_fold;
mod tree_open;
mod tree_start;
mod tree_text;
mod tree_views;
mod ui_flows;
mod undo;
mod upgrade_button;
mod usage;
mod user_config;
mod user_data_writes;
mod work;
mod worktree;
mod write_keys;
mod zz_scan;

/// With `autotests = false`, cargo builds no file of tests/ that is not a module here or a target
/// in Cargo.toml. Such a file and its tests would not run.
#[test]
fn each_test_file_is_built() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let main = std::fs::read_to_string(root.join("tests/main.rs")).unwrap();
    let cargo = std::fs::read_to_string(root.join("Cargo.toml")).unwrap();
    let mut lost: Vec<String> = std::fs::read_dir(root.join("tests"))
        .unwrap()
        .flatten()
        .filter_map(|e| e.file_name().to_str()?.strip_suffix(".rs").map(str::to_string))
        .filter(|n| n != "main" && !main.contains(&format!("\nmod {n};")) && !cargo.contains(&format!("\"tests/{n}.rs\"")))
        .collect();
    lost.sort();
    assert!(lost.is_empty(), "add `mod <file>;` to tests/main.rs for each file: {lost:?}");
}
