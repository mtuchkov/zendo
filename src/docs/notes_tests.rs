use super::*;

#[test]
fn a_missing_notes_folder_is_not_created_on_load() {
    let root = std::env::temp_dir().join(format!("zendo-notes-missing-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let store = Store::new(root.clone(), root.join("docs/notes"));
    assert!(store.load_all().unwrap().is_empty());
    assert!(!root.join("docs/notes").exists());
    save(&store.path_for("First", chrono::Local::now().date_naive()), "# First\n").unwrap();
    assert_eq!(store.load_all().unwrap().len(), 1);
}

#[test]
fn slug_is_short_and_clean() {
    assert_eq!(slug("Event log as the source of truth!"), "event-log-as-the-source-of-truth");
    assert_eq!(slug("  Read  replicas / reports "), "read-replicas-reports");
    assert_eq!(slug("Кэш и TTL"), "кэш-и-ttl");
}

#[test]
fn date_prefix_is_removed() {
    assert_eq!(strip_date("2026-11-11-outbox-pattern"), "outbox-pattern");
    assert_eq!(strip_date("outbox-pattern"), "outbox-pattern");
}

#[test]
fn title_and_tags() {
    let body = "# Event log\n\n#orders and #consistency, see #42\n## Decision #not-a-tag\n```\n#nope\n```\n";
    assert_eq!(title_of(body).as_deref(), Some("Event log"));
    assert_eq!(tags_in(body), vec!["orders", "consistency"]);
}
