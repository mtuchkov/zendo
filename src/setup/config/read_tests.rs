use super::*;

fn lines_of(problems: &[Problem]) -> Vec<usize> {
    problems.iter().filter_map(|p| p.line).collect()
}

// CMD-17, CMD-18: `a line with an error does not hide the others`: a value of the wrong type and a
// broken table header cost only their own settings, and the problems name their lines.
#[test]
fn a_line_with_an_error_does_not_hide_the_others() {
    let text = "agent = \"my-agent\"\nwidth = \"wide\"\nsound = true\n\n[name_models\nclaude = \"sonnet\"\n\n[notes]\npath = \"/n\"\nmachine = \"m\"\n";
    let (cfg, problems) = read_text(text);
    assert_eq!((cfg.agent.as_deref(), cfg.sound), (Some("my-agent"), Some(true)));
    assert_eq!(cfg.notes.as_ref().map(|n| n.machine.as_str()), Some("m"));
    assert_eq!(cfg.width, None, "the value with an error is not read");
    assert_eq!(lines_of(&problems), [2, 5], "{problems:?}");
    assert!(problems.iter().all(|p| !p.error.is_empty() && !p.error.contains('\n')), "{problems:?}");
}

// CMD-17, CMD-18: a syntax error, the most common typo, costs only its own line: the settings
// before and after it in the same block stay.
#[test]
fn a_syntax_error_costs_only_its_line() {
    let (cfg, problems) = read_text("agent = \"my-agent\"\nsound = tru\nmusic = false\n");
    assert_eq!((cfg.agent.as_deref(), cfg.sound, cfg.music), (Some("my-agent"), None, Some(false)));
    assert_eq!(lines_of(&problems), [2], "{problems:?}");
    let (cfg, problems) = read_text("[profile_dirs]\nA = \"1\"\nB = no quotes\nC = \"3\"\n");
    assert_eq!(cfg.profile_dirs.keys().map(String::as_str).collect::<Vec<_>>(), ["A", "C"]);
    assert_eq!(lines_of(&problems), [3], "{problems:?}");
}

// CMD-17: an array over several lines is one setting, also in a block with an error.
#[test]
fn an_array_over_lines_is_one_setting() {
    let text = "sound = tru\n[keys.read]\nadd_to_chat = [\n  \"+\",\n  \"m\",\n]\n[name_models]\nclaude = \"sonnet\"\n";
    let (cfg, problems) = read_text(text);
    assert_eq!(lines_of(&problems), [1], "{problems:?}");
    assert_eq!(cfg.name_models.get("claude").map(String::as_str), Some("sonnet"));
    assert!(!cfg.keys.is_empty(), "the keys of the read pane are read");
}

// CMD-18: a setting written twice is a problem at its second line; zendo uses the last.
#[test]
fn a_setting_written_twice_is_a_problem() {
    let (cfg, problems) = read_text("sound = true\nmusic = true\nsound = false\n");
    assert_eq!((cfg.sound, cfg.music), (Some(false), Some(true)));
    assert_eq!(lines_of(&problems), [3], "{problems:?}");
    let (_, problems) = read_text("[notes]\npath = \"/a\"\nmachine = \"m\"\n[notes]\npath = \"/b\"\nmachine = \"m\"\n");
    assert_eq!(lines_of(&problems), [4], "{problems:?}");
}

// CMD-17: an indented table header is a header; a `[` in an array over lines is not.
#[test]
fn an_indented_header_is_a_header() {
    let lines = ["agent = \"a\"", "  [notes]", "  path = \"/n\"", "x = [", "  [1, 2],", "]", "[name_models"];
    assert_eq!(headers(&lines), [false, true, false, false, false, false, true]);
}

// CMD-17: a good file has no problems; an empty file is the defaults.
#[test]
fn a_good_file_has_no_problems() {
    let (cfg, problems) = read_text("sound = false\n\n[name_models]\nclaude = \"sonnet\"\n");
    assert!(problems.is_empty(), "{problems:?}");
    assert_eq!((cfg.sound, cfg.name_models.get("claude").map(String::as_str)), (Some(false), Some("sonnet")));
    assert!(read_text("").1.is_empty());
}
