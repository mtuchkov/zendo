use super::*;
use std::cell::RefCell;

fn folder(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("zendo-recipe-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// The commands that ran, and what each prints.
struct Mock {
    ran: RefCell<Vec<Vec<String>>>,
    prints: String,
}

impl Run for Mock {
    fn run(&self, argv: &[String]) -> Result<String, String> {
        self.ran.borrow_mut().push(argv.to_vec());
        Ok(self.prints.clone())
    }
}

fn mock(prints: &str) -> Mock {
    Mock { ran: RefCell::new(Vec::new()), prints: prints.into() }
}

// FONT-23, FONT-29, FONT-30, FONT-40, FONT-41: a file line, a whole file, a command with `{size}`,
// an escape sequence and a reload; the commands run as argument lists, without a shell.
#[test]
fn a_recipe_sets_the_font_in_three_ways() {
    let d = folder("three");
    let (conf, whole) = (d.join("config"), d.join("font_name"));
    std::fs::write(&conf, "theme = dark\nfont-family = \"Menlo\"\nfont-size = 13\n").unwrap();
    std::fs::write(&whole, "\"Fira Mono\"").unwrap();
    let toml = format!(
        r#"
[demo]
file = "{conf}"
prefix = "font-family"
line = 'font-family = "{{font}}"'
command = ["gsettings", "set", "x", "font", "{{font}} {{size}}"]
escape = "\u001b]50;{{font}}\u0007"
reload = ["pkill", "-USR1", "demo"]
size = ["gsettings", "get", "x", "font"]

[cosmic]
file = "{whole}"
whole = true
line = '"{{font}}"'
"#,
        conf = conf.display(),
        whole = whole.display()
    );
    let recipes = Recipes::parse(&toml).unwrap();
    let run = mock("'Fira Code 13'\n");
    let escape = recipes.get("demo").unwrap().set("Iosevka Term", &run).unwrap();
    assert_eq!(std::fs::read_to_string(&conf).unwrap(), "theme = dark\nfont-family = \"Iosevka Term\"\nfont-size = 13\n");
    assert_eq!(escape.as_deref(), Some("\x1b]50;Iosevka Term\x07"));
    let ran = run.ran.borrow();
    assert_eq!(ran[0], ["gsettings", "get", "x", "font"], "the size first");
    assert_eq!(ran[1], ["gsettings", "set", "x", "font", "Iosevka Term 13"], "one argument, no shell");
    assert_eq!(ran[2], ["pkill", "-USR1", "demo"], "the reload last");
    assert_eq!(ran.len(), 3);
    recipes.get("cosmic").unwrap().set("Iosevka Term", &mock("")).unwrap();
    assert_eq!(std::fs::read_to_string(&whole).unwrap(), "\"Iosevka Term\"");
    let _ = std::fs::remove_dir_all(&d);
}

// FONT-40: a line that the file does not have yet goes to its end.
#[test]
fn a_missing_line_goes_to_the_end_of_the_file() {
    let d = folder("add");
    let conf = d.join("kitty.conf");
    std::fs::write(&conf, "font_size 12").unwrap();
    let r = Recipe { file: Some(conf.display().to_string()), prefix: Some("font_family".into()), line: Some("font_family {font}".into()), ..Recipe::default() };
    r.set("Fira Code", &mock("")).unwrap();
    assert_eq!(std::fs::read_to_string(&conf).unwrap(), "font_size 12\nfont_family Fira Code\n");
    let _ = std::fs::remove_dir_all(&d);
}

// FONT-43: `{size}` without a number in the output of the `size` command sets nothing.
#[test]
fn a_size_without_a_number_sets_nothing() {
    let r = Recipe { command: Some(vec!["set".into(), "{font} {size}".into()]), size: Some(vec!["get".into()]), ..Recipe::default() };
    let run = mock("'Monospace'");
    let err = r.set("Fira Code", &run).unwrap_err();
    assert!(err.contains("size"), "{err}");
    assert_eq!(run.ran.borrow().len(), 1, "only the size command ran");
}

// FONT-31: the check names a missing file and a command that is not on the PATH.
#[test]
fn the_check_names_a_missing_file_and_command() {
    let d = folder("check");
    let gone = d.join("gone.conf");
    let r = Recipe { file: Some(gone.display().to_string()), line: Some("x {font}".into()), command: Some(vec!["no-such-program-zendo".into()]), ..Recipe::default() };
    let err = r.check(&|p| p == "gsettings").unwrap_err();
    assert!(err.contains("gone.conf") && err.contains("no-such-program-zendo"), "{err}");
    std::fs::write(&gone, "").unwrap();
    let ok = Recipe { file: Some(gone.display().to_string()), line: Some("x {font}".into()), command: Some(vec!["gsettings".into()]), ..Recipe::default() };
    assert_eq!(ok.check(&|p| p == "gsettings"), Ok(()));
    assert!(Recipe::default().check(&|_| true).is_err(), "a recipe with no way sets nothing");
    let _ = std::fs::remove_dir_all(&d);
}

// FONT-32, FONT-35: the file keeps one recipe for each terminal; the font that zendo set stays in
// the recipe, and the rest of the file stays as it is.
#[test]
fn the_font_that_was_set_stays_in_the_recipe() {
    let d = folder("remember");
    let path = d.join("terminal-fonts.toml");
    std::fs::write(&path, "# by the agent\n[ghostty]\nfile = \"~/.config/ghostty/config\"\nline = 'font-family = \"{font}\"'\n\n[kitty]\ncommand = [\"kitten\"]\n").unwrap();
    remember(&path, "ghostty", "Fira Code").unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.starts_with("# by the agent\n[ghostty]") && text.contains("[kitty]"), "{text}");
    let recipes = Recipes::load(&path).unwrap();
    assert_eq!(recipes.get("ghostty").unwrap().font.as_deref(), Some("Fira Code"));
    assert_eq!(recipes.get("kitty").unwrap().font, None);
    assert!(recipes.get("wezterm").is_none());
    let _ = std::fs::remove_dir_all(&d);
}

// FONT-32, FONT-42: the name of the current terminal.
#[test]
fn the_current_terminal_comes_from_the_environment() {
    let name = |vars: &[(&str, &str)]| {
        let vars: Vec<(String, String)> = vars.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
        terminal(&|k| vars.iter().find(|(n, _)| n == k).map(|(_, v)| v.clone()))
    };
    assert_eq!(name(&[("TERM_PROGRAM", "ghostty"), ("TERM", "xterm-ghostty")]), "ghostty");
    assert_eq!(name(&[("TERM_PROGRAM", "iTerm.app")]), "iterm.app");
    assert_eq!(name(&[("GNOME_TERMINAL_SERVICE", ":1.80"), ("VTE_VERSION", "7600")]), "gnome-terminal");
    assert_eq!(name(&[("KITTY_WINDOW_ID", "1"), ("TERM", "xterm-kitty")]), "kitty");
    assert_eq!(name(&[("VTE_VERSION", "7600")]), "vte");
    assert_eq!(name(&[("TERM", "xterm-256color")]), "xterm-256color");
    assert_eq!(name(&[("TERM_PROGRAM", ""), ("TERM", "xterm")]), "xterm", "an empty variable counts as not set");
    assert_eq!(name(&[]), "unknown");
}
