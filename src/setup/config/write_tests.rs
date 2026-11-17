use super::*;
use crate::setup::config::read::read_text;

/// The file `text` after `change` of its settings.
fn save(text: &str, change: impl FnOnce(&mut UserConfig)) -> String {
    let (old, _) = read_text(text);
    let mut new = copy(&old);
    change(&mut new);
    String::from_utf8(edit_bytes(text.as_bytes(), &old, &new)).unwrap()
}

const BROKEN: &str = "\
agent = \"my-agent\"
width = \"wide\"
sound = true

[name_models
claude = \"sonnet\"

[notes]
path = \"/n\" # the clone
machine = \"m\"

# my env
[env]
A = \"1\"
";

// CMD-19: `a saved setting changes one line`, in a file with an error: the line of the setting
// changes; without it, one line comes at the end of the settings before the first table.
#[test]
fn a_saved_setting_changes_one_line() {
    assert_eq!(save(BROKEN, |c| c.sound = Some(false)), BROKEN.replace("sound = true", "sound = false"));
    let without = BROKEN.replace("sound = true\n", "");
    let added = save(&without, |c| c.music = Some(false));
    assert_eq!(added, without.replace("width = \"wide\"\n", "width = \"wide\"\nmusic = false\n"));
}

// CMD-19: in a file without an error, the comments and the format stay; a new setting comes after
// the other settings, before the first table.
#[test]
fn a_saved_setting_keeps_the_comments() {
    let text = "# my zendo\nagent = \"my-agent\"  # mine\nsound = true # the bell\n\n[notes]\npath = \"/n\" # here\nmachine = \"m\"\n";
    assert_eq!(save(text, |c| c.sound = Some(false)), text.replace("sound = true # the bell", "sound = false # the bell"));
    let added = save(text, |c| c.music = Some(true));
    assert_eq!(added, text.replace("sound = true # the bell\n", "sound = true # the bell\nmusic = true\n"));
}

// CMD-19: a file that starts with a table gets the new setting on the first line.
#[test]
fn a_setting_goes_before_the_first_table() {
    let text = "[notes]\npath = \"/n\"\nmachine = \"m\"\n[name_models\n";
    assert_eq!(save(text, |c| c.sound = Some(false)), format!("sound = false\n{text}"));
}

// CMD-19: a setting in a table, as the notes repo or NODE_OPTIONS of `zendo setup`, changes only
// its line in the table; the comments of the table and the lines after it stay. With an error in
// the file and without.
#[test]
fn a_setting_in_a_table_changes_one_line() {
    let machine = save(BROKEN, |c| c.notes.as_mut().unwrap().machine = "laptop".into());
    assert_eq!(machine, BROKEN.replace("machine = \"m\"", "machine = \"laptop\""));
    let env = save(BROKEN, |c| drop(c.env.insert("NODE_OPTIONS".into(), "--use-system-ca".into())));
    assert_eq!(env, BROKEN.replace("A = \"1\"\n", "A = \"1\"\nNODE_OPTIONS = \"--use-system-ca\"\n"));
    let good = "[env]\n# proxy for work\nHTTPS_PROXY = \"http://p\" # office\n";
    let env = save(good, |c| drop(c.env.insert("NODE_OPTIONS".into(), "x".into())));
    assert_eq!(env, format!("{good}NODE_OPTIONS = \"x\"\n"));
}

// CMD-19: a new table goes at the end of the file, after an empty line.
#[test]
fn a_new_table_goes_at_the_end() {
    let text = "sound = tru\n";
    let out = save(text, |c| drop(c.name_models.insert("claude".into(), "sonnet".into())));
    assert_eq!(out, "sound = tru\n\n[name_models]\nclaude = \"sonnet\"\n");
}

// CMD-19: a file with CRLF line endings keeps them; only the line of the setting changes.
#[test]
fn crlf_line_endings_stay() {
    let text = "# c\r\nagent = \"a\"\r\nsound = true\r\n\r\n[notes]\r\npath = \"/n\"\r\nmachine = \"m\"\r\n";
    assert_eq!(save(text, |c| c.sound = Some(false)), text.replace("sound = true", "sound = false"));
    let broken = format!("{text}[name_models\r\n");
    assert_eq!(save(&broken, |c| c.sound = Some(false)), broken.replace("sound = true", "sound = false"));
}

// CMD-19: a setting written twice is saved once: the first line changes, the later ones go, so the
// next start reads the saved value.
#[test]
fn a_setting_written_twice_is_saved_once() {
    let out = save("sound = true\nmusic = true\nsound = true\n", |c| c.sound = Some(false));
    assert_eq!(out, "sound = false\nmusic = true\n");
    assert_eq!(read_text(&out).0.sound, Some(false));
}

// CMD-19: an indented table: the new setting goes before it, not into it.
#[test]
fn a_setting_goes_before_an_indented_table() {
    let text = "agent = \"a\"\n  [notes]\n  path = \"/n\"\n  machine = \"m\"\n[name_models\n";
    let out = save(text, |c| c.sound = Some(false));
    assert_eq!(out, text.replace("agent = \"a\"\n", "agent = \"a\"\nsound = false\n"));
}

// CMD-17, CMD-19: bytes that are not UTF-8, as a comment in Latin-1, stay as they are.
#[test]
fn bytes_that_are_not_utf8_stay() {
    let text: &[u8] = b"# caf\xE9\nsound = true\n";
    let (old, _) = read_text(&String::from_utf8_lossy(text));
    let mut new = copy(&old);
    new.sound = Some(false);
    assert_eq!(edit_bytes(text, &old, &new), b"# caf\xE9\nsound = false\n");
}

// CMD-19: a setting that goes away takes its line, or its table, with it.
#[test]
fn a_removed_setting_takes_its_lines() {
    let text = "agent = \"my-agent\"\nsound = true\n[name_models\n";
    assert_eq!(save(text, |c| c.agent = None), "sound = true\n[name_models\n");
    let out = save(BROKEN, |c| c.notes = None);
    assert_eq!(out, BROKEN.replace("[notes]\npath = \"/n\" # the clone\nmachine = \"m\"\n\n", ""));
}

// CMD-17: a file with CRLF line endings reads as one with LF, also after a save.
#[test]
fn crlf_reads_back() {
    let text = "# c\r\nagent = \"a\"\r\nsound = true\r\n\r\n[env]\r\nA = \"1\"\r\n";
    let (cfg, problems) = read_text(text);
    assert!(problems.is_empty(), "{problems:?}");
    assert_eq!((cfg.agent.as_deref(), cfg.sound), (Some("a"), Some(true)));
    let broken = format!("{text}[name_models\r\n");
    let (cfg, _) = read_text(&save(&broken, |c| c.sound = Some(false)));
    assert_eq!((cfg.agent.as_deref(), cfg.sound), (Some("a"), Some(false)));
}

// CMD-19: a table written twice: the save goes to the last one, the one that zendo reads.
#[test]
fn a_table_written_twice_saves_into_the_last() {
    let text = "sound = tru\n[notes]\npath = \"/n\"\nmachine = \"a\"\n[notes]\npath = \"/n\"\nmachine = \"b\"\n";
    let out = save(text, |c| c.notes.as_mut().unwrap().machine = "c".into());
    assert_eq!(read_text(&out).0.notes.map(|n| n.machine), Some("c".into()));
    assert_eq!(out, text.replace("machine = \"b\"", "machine = \"c\""));
}

// CMD-19: a dotted setting before the first table changes on its own line, without a new table.
#[test]
fn a_dotted_setting_changes_its_line() {
    let text = "sound = tru\nnotes.path = \"/n\"\nnotes.machine = \"m\"\n";
    let out = save(text, |c| c.notes.as_mut().unwrap().machine = "c".into());
    assert_eq!(out, text.replace("notes.machine = \"m\"", "notes.machine = \"c\""));
}

// CMD-19: a key with a control character is written as valid TOML, and reads back.
#[test]
fn a_key_with_a_control_character_reads_back() {
    let out = save("sound = tru\n", |c| drop(c.env.insert("A\u{1b}B".into(), "1".into())));
    let (cfg, problems) = read_text(&out);
    assert_eq!(cfg.env.get("A\u{1b}B").map(String::as_str), Some("1"), "{out}");
    assert_eq!(problems.len(), 1, "only the old line with an error: {problems:?}");
}

// CMD-19: a new setting of a dotted table comes as one more dotted line after the others, not as a
// second form of the table.
#[test]
fn a_new_dotted_setting_joins_the_others() {
    let text = "sound = tru\nnotes.path = \"/n\"\nnotes.machine = \"m\"\nmusic = true\n";
    let out = save(text, |c| c.notes.as_mut().unwrap().sync_seconds = 99);
    assert_eq!(out, text.replace("notes.machine = \"m\"\n", "notes.machine = \"m\"\nnotes.sync_seconds = 99\n"));
}
