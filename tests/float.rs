//! The new note in a float (docs/zendo/float.md): alt-n from every pane, only the float on the
//! screen, esc saves and gives the screen back.

mod common;
use common::http::mock_api;
use common::*;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use zendo::agent::api::{Api, Credential, Vendor};
use zendo::app::{App, Focus, Mode};

const NOTE: &str = "docs/notes/2026-11-12-idea.md";
const DESIGN: &str = "docs/design.md";

fn files() -> Vec<(&'static str, &'static str)> {
    let design = "# Design\n\n".to_string() + &(1..=30).map(|i| format!("Line {i} of the design.\n\n")).collect::<String>();
    vec![(NOTE, "# Idea\n\nThe first paragraph of the idea.\n"), (DESIGN, Box::leak(design.into_boxed_str()))]
}

fn alt(a: &mut App, c: char) {
    a.on_key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::ALT));
}

fn in_float(a: &App) -> bool {
    a.writer.float().is_some() && a.mode == Mode::Write && a.writer.editor().text() == "# \n\n" && a.writer.path().is_none()
}

/// The notes folder after the test: the files in it.
fn notes(root: &std::path::Path) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(root.join("docs/notes")).unwrap().flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect();
    v.sort();
    v
}

// FLOAT-1, FLOAT-17: alt-n opens a new note in write mode in a float, from read mode, write mode,
// the chat, the file tree and the delta; the note has an empty title line, and the cursor is on the
// first line of the body.
#[tokio::test]
async fn alt_n_opens_a_float_from_every_pane() {
    let root = repo("float-every-pane", &files());
    type Setup = (&'static str, fn(&mut App));
    let setups: [Setup; 5] = [
        ("read", |_| {}),
        ("write", |a| key(a, KeyCode::Char('i'))),
        ("chat", |a| key(a, KeyCode::Char('c'))),
        ("tree", |a| alt(a, 'b')),
        ("delta", |a| key(a, KeyCode::Char('d'))),
    ];
    for (name, setup) in setups {
        let mut a = app(&root);
        a.open_file(root.join(NOTE));
        draw(&mut a, 160, 40);
        setup(&mut a);
        draw(&mut a, 160, 40);
        alt(&mut a, 'n');
        assert!(in_float(&a), "{name}: a float with an empty note");
        assert_eq!((a.writer.editor().row, a.writer.editor().col), (2, 0), "{name}: the cursor on the first line of the body");
        a.cli.stop_all();
    }
    let _ = std::fs::remove_dir_all(&root);
}

// FLOAT-2, FLOAT-6: the float shows the notes folder, `new note`, the branch and `[ esc close ]`;
// the panes behind are dim, and the status bar has no key hints.
#[tokio::test]
async fn the_float_hides_the_panes_and_the_hints() {
    let root = repo("float-hides", &files());
    let mut a = app(&root);
    a.open_file(root.join(NOTE));
    a.files.set_branch(Some("master".into()));
    key(&mut a, KeyCode::Char('c'));
    let before = draw(&mut a, 120, 40);
    let hint = a.keys.first(zendo::keys::Pane::Chat, "send");
    assert!(before[38].contains(&hint) || before[39].contains(&hint), "the chat shows its hints first:\n{}", before.join("\n"));
    alt(&mut a, 'n');
    let screen = draw(&mut a, 120, 40);
    let top = screen.iter().find(|l| l.contains("new note")).unwrap_or_else(|| panic!("no top line:\n{}", screen.join("\n")));
    assert!(top.contains("docs/notes/new note") && top.contains("master") && top.contains("[ esc close ]"), "{top}");
    assert!(!screen[39].contains(&hint), "no key hints:\n{}", screen.join("\n"));
    let (dim, fg, ink) = (float_colors(&mut a, 120, 40), a.theme().fg, a.theme().on_float().fg);
    assert_ne!(dim.0, fg, "the screen behind is dim");
    assert_eq!(dim.1, ink, "the note in the float is not: it has the ink of the float");
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(&root);
}

/// The color of a char of the screen behind (top left) and of the title in the float.
fn float_colors(a: &mut App, w: u16, h: u16) -> (ratatui::style::Color, ratatui::style::Color) {
    typ(a, "Tea");
    let mut term = ratatui::Terminal::new(ratatui::backend::TestBackend::new(w, h)).unwrap();
    term.draw(|f| a.draw(f)).unwrap();
    let buf = term.backend().buffer().clone();
    let behind = (0..w).map(|x| &buf[(x, 0)]).find(|c| c.symbol() != " ").map(|c| c.fg).unwrap();
    let title = (0..h).flat_map(|y| (0..w - 2).map(move |x| (x, y))).find(|p| buf[*p].symbol() == "T" && buf[(p.0 + 1, p.1)].symbol() == "e" && buf[(p.0 + 2, p.1)].symbol() == "a").unwrap();
    (behind, buf[title].fg)
}

// FLOAT-3, FLOAT-6, FLOAT-7, FLOAT-13: esc saves the note and gives back the file, the pane, the
// mode, the cursor and the selection; the date and time stand under the title; the new note does
// not open in the note pane.
#[tokio::test]
async fn esc_saves_the_float_and_restores_the_screen() {
    let root = repo("float-esc", &files());
    let mut a = app(&root);
    a.open_file(root.join(DESIGN));
    draw(&mut a, 120, 40);
    a.reader.to_row(12);
    a.reader.select_from((10, 0));
    let scroll = a.reader.scroll();
    alt(&mut a, 'n');
    typ(&mut a, "A line of text.");
    title(&mut a, "Tea");
    let screen = draw(&mut a, 120, 40).join("\n");
    let today = chrono::Local::now().format("%a %-d %b").to_string();
    assert!(screen.contains(&today), "the date under the title:\n{screen}");
    key(&mut a, KeyCode::Esc);
    assert!(a.writer.float().is_none(), "esc closes the float");
    let saved: Vec<String> = notes(&root).into_iter().filter(|n| n.ends_with("-tea.md")).collect();
    assert_eq!(saved.len(), 1, "{:?}", notes(&root));
    let text = std::fs::read_to_string(root.join("docs/notes").join(&saved[0])).unwrap();
    assert!(text.starts_with("# Tea\n") && text.contains("A line of text."), "{text}");
    assert_eq!(a.files.current_path(), Some(root.join(DESIGN).as_path()), "the same file");
    assert_eq!((a.mode, a.focus, a.reader.row(), a.reader.anchor(), a.reader.scroll()), (Mode::Read, Focus::Note, 12, Some((10, 0)), scroll));
    assert!(a.files.notes().iter().any(|n| n.title == "Tea"), "the new note is in the notes list");
    let _ = std::fs::remove_dir_all(&root);
}

// FLOAT-4: the open file in write mode is saved before the float opens, and is in write mode again
// after it.
#[tokio::test]
async fn the_float_saves_the_open_file_first() {
    let root = repo("float-save-first", &files());
    let mut a = app(&root);
    a.open_file(root.join(NOTE));
    draw(&mut a, 120, 40);
    key(&mut a, KeyCode::Char('i'));
    draw(&mut a, 120, 40);
    typ(&mut a, "Unsaved ");
    alt(&mut a, 'n');
    assert!(std::fs::read_to_string(root.join(NOTE)).unwrap().contains("Unsaved "), "saved before the float");
    key(&mut a, KeyCode::Esc);
    assert_eq!((a.mode, a.writer.file()), (Mode::Write, Some(root.join(NOTE).as_path())), "write mode again");
    let _ = std::fs::remove_dir_all(&root);
}

// FLOAT-5, FLOAT-15: `n` in read mode and in the notes list opens the float; enter in the notes
// list opens a note in the note pane.
#[tokio::test]
async fn n_opens_the_float() {
    let root = repo("float-n", &files());
    let mut a = app(&root);
    a.open_file(root.join(NOTE));
    key(&mut a, KeyCode::Char('n'));
    assert!(in_float(&a), "n in read mode");
    key(&mut a, KeyCode::Esc);
    key(&mut a, KeyCode::Char('o'));
    assert_eq!(a.mode, Mode::Notes);
    key(&mut a, KeyCode::Char('n'));
    assert!(in_float(&a), "n in the notes list");
    key(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Notes, "back in the notes list");
    key(&mut a, KeyCode::Enter);
    assert!(a.writer.float().is_none() && a.mode == Mode::Read && a.files.current().is_some(), "a note that is there opens in the note pane");
    let _ = std::fs::remove_dir_all(&root);
}

// FLOAT-6: after the first save, the top line shows the path of the note.
#[tokio::test]
async fn the_top_line_shows_the_path_after_the_first_save() {
    let root = repo("float-path", &files());
    let mut a = app(&root);
    alt(&mut a, 'n');
    typ(&mut a, "Green builds");
    a.save();
    let screen = draw(&mut a, 120, 40).join("\n");
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();
    assert!(screen.contains(&format!("docs/notes/{date}-green-builds.md")), "{screen}");
    assert!(a.writer.float().is_some(), "the float stays open after a save");
    let _ = std::fs::remove_dir_all(&root);
}

/// The columns of the float: the first and the last column of its border on the row of its top line.
fn float_box(screen: &[String]) -> (usize, usize) {
    let row = screen.iter().find(|l| l.contains("[ esc close ]")).unwrap();
    let chars: Vec<char> = row.chars().collect();
    let left = chars.iter().position(|c| *c == '│').unwrap();
    let right = chars.iter().rposition(|c| *c == '│').unwrap();
    (left, right)
}

// FLOAT-8, FLOAT-9: the float is the text column wide with a margin of 4 on each side, over the
// note pane.
#[tokio::test]
async fn the_float_fits_the_text_column() {
    let root = repo("float-width", &files());
    let mut a = app(&root);
    a.open_file(root.join(NOTE));
    alt(&mut a, 'n');
    let screen = draw(&mut a, 160, 40);
    let (l, r) = float_box(&screen);
    assert_eq!(r - l - 1, a.column_width() as usize + 8, "the column and two margins");
    let centre = (l + r) / 2;
    assert!(centre.abs_diff(80) <= 1, "over the note pane, alone on the screen: {l}..{r}");
    let top = screen.iter().position(|row| row.contains("[ esc close ]")).unwrap();
    assert_eq!(top, 2, "one row above the box, then its border");
    let _ = std::fs::remove_dir_all(&root);
}

// FLOAT-9: where the note pane is narrower than the float, the float stands at the center of the
// screen.
#[tokio::test]
async fn a_narrow_note_pane_centers_the_float() {
    let root = repo("float-narrow", &files());
    let mut a = app(&root);
    a.open_file(root.join(NOTE));
    key(&mut a, KeyCode::Char('c'));
    alt(&mut a, 'b');
    draw(&mut a, 100, 40);
    alt(&mut a, 'n');
    let screen = draw(&mut a, 100, 40);
    let (l, r) = float_box(&screen);
    assert!(((l + r) / 2).abs_diff(50) <= 1, "at the center: {l}..{r}");
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(&root);
}

// FLOAT-10, FLOAT-11: in the float, the keys of write mode work, and the keys that open a pane,
// move the focus, or open another float do nothing.
#[tokio::test]
async fn pane_keys_do_nothing_in_the_float() {
    let root = repo("float-keys", &files());
    let mut a = app(&root);
    a.open_file(root.join(NOTE));
    alt(&mut a, 'n');
    typ(&mut a, "Tea");
    ctrl(&mut a, 'g');
    alt(&mut a, 'b');
    ctrl(&mut a, 'd');
    a.on_key(KeyEvent::new(KeyCode::F(5), KeyModifiers::NONE));
    alt(&mut a, ']');
    alt(&mut a, 'n');
    assert!(a.writer.float().is_some() && a.mode == Mode::Write, "still in the float");
    assert_eq!((a.threads.shown(), a.state.tree, a.work.delta_open(), a.cli.pane), (false, false, false, None), "no pane opened");
    assert_eq!(a.writer.editor().text(), "# \n\nTea", "the same note");
    key(&mut a, KeyCode::Tab);
    assert_eq!(a.focus, Focus::Note, "tab does not move the focus");
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(&root);
}

// FLOAT-12: a float without text makes no file.
#[tokio::test]
async fn an_empty_float_makes_no_file() {
    let root = repo("float-empty", &files());
    let mut a = app(&root);
    let before = notes(&root);
    alt(&mut a, 'n');
    key(&mut a, KeyCode::Esc);
    assert!(a.writer.float().is_none());
    assert_eq!(notes(&root), before);
    let _ = std::fs::remove_dir_all(&root);
}

// FLOAT-16: the float has its own background, the float color of the theme; the screen behind
// keeps the background of the note.
#[tokio::test]
async fn the_float_has_the_float_color_of_the_theme() {
    let root = repo("float-panel-bg", &files());
    let mut a = app(&root);
    a.open_file(root.join(NOTE));
    alt(&mut a, 'n');
    typ(&mut a, "Tea");
    let mut term = ratatui::Terminal::new(ratatui::backend::TestBackend::new(120, 40)).unwrap();
    term.draw(|f| a.draw(f)).unwrap();
    let buf = term.backend().buffer().clone();
    let t = *a.theme();
    let r = a.writer.float().unwrap().rect;
    let border = (r.left()..r.right()).flat_map(|x| [(x, r.top()), (x, r.bottom() - 1)]).chain((r.top()..r.bottom()).flat_map(|y| [(r.left(), y), (r.right() - 1, y)]));
    let empty = (r.left()..r.right()).map(|x| (x, r.bottom() - 2));
    assert!(border.chain(empty).all(|p| buf[p].bg == t.float), "the border and the empty rows of the float have the float color");
    let tea = (r.top()..r.bottom()).flat_map(|y| (r.left()..r.right() - 2).map(move |x| (x, y))).find(|&(x, y)| buf[(x, y)].symbol() == "T" && buf[(x + 1, y)].symbol() == "e" && buf[(x + 2, y)].symbol() == "a").unwrap();
    assert_eq!(buf[tea].fg, t.on_float().fg, "the text of the float has the ink of the float");
    assert_eq!(buf[(0, 0)].bg, t.bg, "the screen behind keeps the note color");
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(&root);
}

// FLOAT-2, FLOAT-16: the rows of the note in the float, not only the cursor row, have the ink of the
// float, and inline code stands on the code panel of the float.
#[tokio::test]
async fn the_rows_of_the_float_have_its_ink() {
    let root = repo("float-ink", &files());
    let mut a = app(&root);
    a.open_file(root.join(NOTE));
    alt(&mut a, 'n');
    typ(&mut a, "Tea");
    key(&mut a, KeyCode::Enter);
    typ(&mut a, "Leaf `cup`");
    key(&mut a, KeyCode::Enter);
    key(&mut a, KeyCode::Enter);
    typ(&mut a, "pot");
    let mut term = ratatui::Terminal::new(ratatui::backend::TestBackend::new(120, 40)).unwrap();
    term.draw(|f| a.draw(f)).unwrap();
    let buf = term.backend().buffer().clone();
    let ink = a.theme().on_float();
    let r = a.writer.float().unwrap().rect;
    let cells = |y: u16| (r.left()..r.right()).map(move |x| (x, y));
    let leaf = (r.top() + 2..r.bottom()).flat_map(cells).find(|&p| buf[p].symbol() == "L").unwrap();
    assert_eq!(buf[leaf].fg, ink.fg, "a row off the cursor has the ink of the float");
    let cup = cells(leaf.1).skip_while(|&p| p != leaf).find(|&p| buf[p].symbol() == "c").unwrap();
    assert_eq!(buf[cup].bg, ink.panel, "inline code stands on the code panel of the float");
    assert_ne!(buf[cup].bg, buf[leaf].bg, "inline code stands out of the text");
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(&root);
}

// FLOAT-22: until the note has a title, `new note` stands in its place, in dim colors.
#[tokio::test]
async fn the_float_shows_new_note_until_the_title() {
    let root = repo("float-new-note", &files());
    let mut a = app(&root);
    let in_place = |screen: &[String]| screen.iter().any(|l| l.contains('│') && l.replace('│', "").trim() == "new note");
    alt(&mut a, 'n');
    typ(&mut a, "Leaf");
    assert!(in_place(&draw(&mut a, 120, 40)), "`new note` in place of the title");
    title(&mut a, "Tea");
    key(&mut a, KeyCode::Down);
    let screen = draw(&mut a, 120, 40);
    assert!(!in_place(&screen), "the title in its place:\n{}", screen.join("\n"));
    let _ = std::fs::remove_dir_all(&root);
}

/// Type a title into the title line of the float, from the body.
fn title(a: &mut App, text: &str) {
    while a.writer.editor().row > 0 {
        key(a, KeyCode::Up);
    }
    key(a, KeyCode::End);
    typ(a, text);
}

/// Type lines of text in write mode, one after the other.
fn lines(a: &mut App, lines: &[&str]) {
    for (i, l) in lines.iter().enumerate() {
        if i > 0 {
            key(a, KeyCode::Enter);
        }
        typ(a, l);
    }
}

/// The text of the note in the notes folder whose name has `part`.
fn note_with(root: &std::path::Path, part: &str) -> Option<String> {
    notes(root).into_iter().find(|n| n.contains(part)).and_then(|n| std::fs::read_to_string(root.join("docs/notes").join(n)).ok())
}

// FLOAT-18, FLOAT-20, FLOAT-23: after three lines of text, the name model writes the title into the
// title line; the cursor stays on its char; the file gets the name of the title.
#[tokio::test]
async fn the_name_model_titles_the_note_after_three_lines() {
    let root = repo("float-title-live", &files());
    let mut a = app(&root);
    alt(&mut a, 'n');
    lines(&mut a, &["Green tea.", "Black tea.", "White"]);
    a.save();
    until(&mut a, |a| a.writer.editor().lines[0].len() > 2).await;
    assert_eq!(a.writer.editor().lines[..2], ["# Mock title of 3 lines".to_string(), String::new()], "the title and an empty line");
    assert_eq!((a.writer.editor().row, a.writer.editor().col), (4, 5), "the cursor on the same char");
    typ(&mut a, " tea.");
    a.save();
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();
    let named = root.join(format!("docs/notes/{date}-mock-title-of-3-lines.md"));
    assert_eq!(a.writer.path(), Some(&named), "the file has the name of the title: {:?}", notes(&root));
    assert!(std::fs::read_to_string(&named).unwrap().contains("White tea."));
    assert_eq!(notes(&root).len(), 2, "no file of the old name: {:?}", notes(&root));
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(&root);
}

// FLOAT-19, FLOAT-21: closing the float titles a note without a title, titles again a note with the
// title of the agent and new text, and keeps the title of the user.
#[tokio::test]
async fn closing_the_float_titles_or_updates_the_note() {
    let root = repo("float-title-close", &files());
    let mut a = app(&root);
    alt(&mut a, 'n');
    lines(&mut a, &["One.", "Two.", "Three."]);
    title(&mut a, "Mine");
    key(&mut a, KeyCode::Esc);
    alt(&mut a, 'n');
    typ(&mut a, "Only one line.");
    key(&mut a, KeyCode::Esc);
    until(&mut a, |_| note_with(&root, "mock-title-of-1-lines").is_some()).await;
    assert!(note_with(&root, "mock-title-of-1-lines").unwrap().starts_with("# Mock title of 1 lines\n\nOnly one line."));
    alt(&mut a, 'n');
    lines(&mut a, &["Cups.", "Pots.", "Leaves."]);
    a.save();
    until(&mut a, |a| a.writer.editor().lines[0].len() > 2).await;
    key(&mut a, KeyCode::Enter);
    typ(&mut a, "Water.");
    key(&mut a, KeyCode::Esc);
    until(&mut a, |_| note_with(&root, "mock-title-of-4-lines").is_some()).await;
    assert!(note_with(&root, "mock-title-of-4-lines").unwrap().starts_with("# Mock title of 4 lines\n"));
    assert!(note_with(&root, "mine").is_some_and(|t| t.starts_with("# Mine\n")), "the title of the user stays: {:?}", notes(&root));
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(&root);
}

/// The Anthropic API at a mock server that answers `title` to each question.
fn title_api(title: &str) -> (common::http::MockApi, Api) {
    let server = mock_api(serde_json::json!({"content": [{"type": "text", "text": format!("TITLE: {title}")}]}));
    let api = Api { vendor: Vendor::Anthropic, base: server.url.clone(), model: "claude-haiku-5-5".into(), cred: Credential::Key("test-key".into()) };
    (server, api)
}

// FLOAT-18, FLOAT-23, NAMES-15: with an API, the title comes from one request to the API; the agent
// does not start for it.
#[tokio::test]
async fn the_api_titles_the_note_after_three_lines() {
    let root = repo("float-title-api", &files());
    let mut a = app(&root);
    let (server, api) = title_api("Kinds of tea");
    a.link.set_api(Some(api));
    alt(&mut a, 'n');
    lines(&mut a, &["Green tea.", "Black tea.", "White"]);
    a.save();
    until(&mut a, |a| a.writer.editor().lines[0].len() > 2).await;
    assert_eq!(a.writer.editor().lines[0], "# Kinds of tea");
    let reqs = server.requests.lock().unwrap().clone();
    assert_eq!(reqs.len(), 1, "{reqs:#?}");
    let prompt = reqs[0].body["messages"][0]["content"][0]["text"].as_str().unwrap_or_default();
    assert!(prompt.contains("TITLE: <the title>") && prompt.contains("Black tea."), "{prompt}");
    assert!(!a.link.running(), "the agent does not start for a title");
    let _ = std::fs::remove_dir_all(&root);
}

// FLOAT-19, NAMES-15: with an API, closing the float titles the note in its file through the API,
// without the agent.
#[tokio::test]
async fn closing_the_float_titles_the_note_through_the_api() {
    let root = repo("float-title-close-api", &files());
    let mut a = app(&root);
    let (server, api) = title_api("Single sentence");
    a.link.set_api(Some(api));
    alt(&mut a, 'n');
    typ(&mut a, "Only one line.");
    key(&mut a, KeyCode::Esc);
    until(&mut a, |_| note_with(&root, "single-sentence").is_some()).await;
    assert!(note_with(&root, "single-sentence").unwrap().starts_with("# Single sentence\n\nOnly one line."));
    assert_eq!(server.requests.lock().unwrap().len(), 1);
    assert!(!a.link.running(), "the agent does not start for a title");
    let _ = std::fs::remove_dir_all(&root);
}
