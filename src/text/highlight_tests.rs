use super::*;

fn kinds(lang: &str, code: &str) -> Vec<(Kind, String)> {
    let lines: Vec<String> = code.lines().map(str::to_string).collect();
    pieces(lang, &lines).unwrap_or_default().into_iter().flatten().collect()
}

#[test]
fn the_default_languages_are_on() {
    for (lang, code, want) in [
        ("rust", "fn main() { let x = 42; } // c", Kind::Keyword),
        ("sh", "echo \"hi\" # note", Kind::String),
        ("python", "def f():\n    return 1", Kind::Keyword),
        ("go", "package main", Kind::Keyword),
        ("java", "public class A {}", Kind::Keyword),
        ("yaml", "name: zendo", Kind::Name),
        ("json", "{\"a\": 1}", Kind::Constant),
    ] {
        let k = kinds(lang, code);
        assert!(k.iter().any(|(kind, _)| *kind == want), "{lang}: {k:?}");
        assert_eq!(k.iter().map(|(_, s)| s.as_str()).collect::<String>(), code.replace('\n', ""), "{lang}: no text lost");
    }
    assert!(pieces("ruby", &["puts 1".into()]).is_none(), "ruby is not on by default");
    assert!(pieces("nonsense", &["x".into()]).is_none());
}

#[test]
fn a_language_is_on_by_config_or_by_plugin_file() {
    let dir = std::env::temp_dir().join(format!("zendo-syntax-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("ears.sublime-syntax"),
        "%YAML 1.2\n---\nname: EARS\nfile_extensions: [ears]\nscope: source.ears\ncontexts:\n  main:\n    - match: '\\b(WHEN|SHALL)\\b'\n      scope: keyword.control.ears\n",
    )
    .unwrap();
    std::fs::write(dir.join("broken.sublime-syntax"), "not: [yaml").unwrap();
    let e = build(Some(&Config { languages: Some(vec!["ruby".into(), "klingon".into()]) }), &dir);
    assert!(e.on.contains("Ruby") && e.on.contains("EARS") && !e.on.contains("Rust"), "{:?}", e.on);
    assert_eq!(e.problems.len(), 2, "{:?}", e.problems);
    let _ = std::fs::remove_dir_all(&dir);
}

// FILES-21: a whole file in the note pane: every language that zendo knows, TOML too, and the file name when
// the file has no extension.
#[test]
fn a_whole_file_uses_every_language_that_zendo_knows() {
    let flat = |lang: &str, code: &str| -> Vec<(Kind, String)> {
        let lines: Vec<String> = code.lines().map(str::to_string).collect();
        file_pieces(lang, &lines).unwrap_or_default().into_iter().flatten().collect()
    };
    for (lang, code, want) in [
        ("rb", "def f; end", Kind::Keyword),
        ("toml", "[package]\nname = \"zendo\" # c", Kind::String),
        ("toml", "[package]", Kind::Name),
        ("ts", "const x = 1;", Kind::Keyword),
        ("Makefile", "all:\n\techo hi # c", Kind::Comment),
    ] {
        let k = flat(lang, code);
        assert!(k.iter().any(|(kind, _)| *kind == want), "{lang}: {k:?}");
        assert_eq!(k.iter().map(|(_, s)| s.as_str()).collect::<String>(), code.replace('\n', ""), "{lang}: no text lost");
    }
    assert!(pieces("rb", &["def f; end".into()]).is_none(), "a fence in a note still uses only the languages that are on");
}
