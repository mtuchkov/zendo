//! The chips of the chat input before a question goes (docs/zendo/chat-chips.md).

mod common;
use common::*;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use zendo::agent::Attachment;
use zendo::app::{App, CHAT_KEY, Focus, Item};

const A: &str = "docs/notes/a.md";
const B: &str = "docs/notes/b.md";

fn files() -> Vec<(&'static str, &'static str)> {
    vec![(A, "# A\n\ntext\n"), (B, "# B\n\nmore\n")]
}

/// The screen as text, and its cells for the colors.
fn draw_buf(a: &mut App, w: u16, h: u16) -> (Vec<String>, Buffer) {
    let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
    term.draw(|f| a.draw(f)).unwrap();
    let buf = term.backend().buffer().clone();
    // A label keeps its words together with no-break spaces; they look as spaces.
    let rows = (0..h).map(|y| (0..w).map(|x| buf[(x, y)].symbol().replace('\u{a0}', " ")).collect::<String>()).collect();
    (rows, buf)
}

fn click(a: &mut App, x: u16, y: u16) {
    for kind in [MouseEventKind::Down(MouseButton::Left), MouseEventKind::Up(MouseButton::Left)] {
        a.on_mouse(MouseEvent { kind, column: x, row: y, modifiers: KeyModifiers::NONE });
    }
}

/// Draw, then double-click the chip `n` of the chat input.
fn double_click_chip(a: &mut App, n: usize) {
    draw_buf(a, 160, 30);
    let (r, _) = a.frame.chip_rects()[n].clone();
    click(a, r.x + 1, r.y);
    click(a, r.x + 1, r.y);
}

/// The chat on the screen, with the keys in its input.
fn chat(root: &Path) -> App {
    let mut a = app(root);
    a.open_panel();
    a.focus = Focus::Claude;
    a
}

/// Ask in the chat and wait until the turn ends.
async fn ask(a: &mut App, q: &str) -> String {
    a.focus = Focus::Claude;
    typ(a, q);
    key(a, KeyCode::Enter);
    until(a, |a| a.threads.get(CHAT_KEY).is_some_and(|t| t.busy.is_none() && !t.items.is_empty())).await;
    last_answer(a)
}

/// The text of the agent after the last question.
fn last_answer(a: &App) -> String {
    let items = &a.threads[CHAT_KEY].items;
    let from = items.iter().rposition(|i| matches!(i, Item::User(_))).unwrap_or(0);
    items[from..].iter().filter_map(|i| if let Item::Agent(s) = i { Some(s.as_str()) } else { None }).collect()
}

// CLAUDE-57: a question with the chip of a gone file goes, with its label and without the file.
#[tokio::test]
async fn a_red_chip_goes_without_its_file() {
    let root = repo("chip-red", &files());
    let mut a = app(&root);
    a.open_panel();
    a.focus = Focus::Claude;
    a.on_paste(root.join(B).display().to_string());
    std::fs::remove_file(root.join(B)).unwrap();
    typ(&mut a, "what is in it");
    key(&mut a, KeyCode::Enter);
    until(&mut a, |a| a.threads.get(CHAT_KEY).is_some_and(|t| t.busy.is_none() && !t.items.is_empty())).await;
    let answer = last_answer(&a);
    assert!(!answer.contains("file link"), "the gone file does not go: {answer}");
    let asked = a.threads[CHAT_KEY].items.iter().find_map(|i| if let Item::User(s) = i { Some(s.clone()) } else { None }).unwrap_or_default();
    assert!(asked.starts_with("⟦otes/b.md\u{2063}"), "the transcript keeps the label: {asked}");
    let _ = std::fs::remove_dir_all(&root);
}

// CLAUDE-48 to CLAUDE-51: a double-click on a text chip shows its text in a window of the dimmed
// chip color, with its bottom right corner at the top right corner of the chat input; other keys
// do nothing, and `esc` closes it.
#[tokio::test]
async fn a_double_click_on_a_chip_shows_its_text_over_the_input() {
    let root = repo("chip-window", &files());
    let mut a = chat(&root);
    a.on_paste("first line\nsecond line\nthird line\n".into());
    draw_buf(&mut a, 160, 30);
    let (r, _) = a.frame.chip_rects()[0].clone();
    click(&mut a, r.x + 1, r.y);
    assert!(a.composer().view().is_none(), "one click changes nothing");
    click(&mut a, r.x + 1, r.y);
    assert!(a.composer().view().is_some(), "a double-click opens the window");
    let (rows, buf) = draw_buf(&mut a, 160, 30);
    let v = a.frame.chip_view_rect().expect("the chip window");
    let input = a.frame.input_rect().expect("the chat input");
    assert_eq!((v.right(), v.bottom()), (input.right(), input.y), "{v:?} {input:?}");
    assert!(v.width <= a.column_width() + 4, "at most the text column wide");
    let top = &rows[v.y as usize];
    assert!(top.contains("first line") && top.contains("[ esc close ]"), "{top}");
    assert!(rows.iter().any(|r| r.contains("second line") && !r.contains("> second")), "the text without the quote marks");
    let t = *a.theme();
    let bg = zendo::docs::delta::mix(t.panel, t.acc2, zendo::ui::CHIP_DIM);
    assert_eq!(buf[(v.x + 2, v.y + 2)].bg, bg, "the dimmed color of the chip");
    let before = a.composer().input().text();
    typ(&mut a, "x");
    key(&mut a, KeyCode::Down);
    assert_eq!(a.composer().input().text(), before, "other keys do nothing");
    assert!(a.composer().view().is_some());
    key(&mut a, KeyCode::Esc);
    assert!(a.composer().view().is_none(), "esc closes the window");
    assert_eq!(a.composer().input().text(), before, "and changes nothing else");
    let _ = std::fs::remove_dir_all(&root);
}

// CLAUDE-48: the chip of a text file shows the text of the file.
#[tokio::test]
async fn a_file_chip_shows_the_text_of_its_file() {
    let root = repo("chip-window-file", &files());
    let mut a = chat(&root);
    a.on_paste(root.join(A).display().to_string());
    double_click_chip(&mut a, 0);
    let (rows, _) = draw_buf(&mut a, 160, 30);
    assert!(rows.iter().any(|r| r.contains("# A")), "{rows:#?}");
    let _ = std::fs::remove_dir_all(&root);
}

static OPENED: Mutex<Vec<PathBuf>> = Mutex::new(Vec::new());

fn record(p: &Path) -> std::io::Result<()> {
    OPENED.lock().unwrap().push(p.to_path_buf());
    Ok(())
}

fn fail(_: &Path) -> std::io::Result<()> {
    Err(std::io::Error::other("no program for it"))
}

// CLAUDE-52: an image chip, and the chip of a file that is not text, open in the program of the
// system; no window opens.
#[tokio::test]
async fn a_double_click_on_an_image_chip_opens_it_outside() {
    let root = repo("chip-outside", &files());
    let (png, bin) = (root.join("shot.png"), root.join("data.bin"));
    std::fs::write(&png, b"png").unwrap();
    std::fs::write(&bin, [0u8, 159, 146, 150]).unwrap();
    let mut a = chat(&root);
    a.opener = record;
    a.composer_mut().add_chip("Image #1".into(), None, Some(Attachment::Image(png.clone())), false);
    a.on_paste(bin.display().to_string());
    double_click_chip(&mut a, 0);
    assert!(a.composer().view().is_none());
    double_click_chip(&mut a, 1);
    assert!(a.composer().view().is_none());
    let opened = OPENED.lock().unwrap().clone();
    assert!(opened.contains(&png) && opened.contains(&bin), "{opened:?}");
    let _ = std::fs::remove_dir_all(&root);
}

// CLAUDE-53: a gone file, and a program that does not start, show their error in the chip window.
#[tokio::test]
async fn a_gone_file_shows_its_error_in_the_chip_window() {
    let root = repo("chip-error", &files());
    let png = root.join("shot.png");
    std::fs::write(&png, b"png").unwrap();
    let mut a = chat(&root);
    a.opener = fail;
    a.on_paste(root.join(B).display().to_string());
    a.composer_mut().add_chip("Image #1".into(), None, Some(Attachment::Image(png)), false);
    std::fs::remove_file(root.join(B)).unwrap();
    double_click_chip(&mut a, 0);
    let (rows, _) = draw_buf(&mut a, 160, 30);
    assert!(rows.iter().any(|r| r.contains("is gone")), "{rows:#?}");
    key(&mut a, KeyCode::Esc);
    double_click_chip(&mut a, 1);
    let (rows, _) = draw_buf(&mut a, 160, 30);
    assert!(rows.iter().any(|r| r.contains("no program for it")), "{rows:#?}");
    let _ = std::fs::remove_dir_all(&root);
}

// CLAUDE-54 to CLAUDE-56: the line above the input says that the open document goes; a click
// unselects it, and the question goes without anything of the document; another document, or a
// change of the text, selects it again.
#[tokio::test]
async fn the_open_document_can_stay_out_of_the_question() {
    let root = repo("chip-open-doc", &files());
    let mut a = app(&root);
    a.open_file(root.join(A));
    a.open_panel();
    a.focus = Focus::Claude;
    let line = |a: &mut App| draw_buf(a, 160, 30).0.into_iter().find(|r| r.contains("open document")).unwrap_or_default();
    assert!(line(&mut a).contains("[x] open document"));
    let r = a.frame.doc_line().expect("the line of the open document");
    click(&mut a, r.x, r.y);
    assert!(line(&mut a).contains("[ ] open document"));
    let answer = ask(&mut a, "what is it").await;
    assert!(answer.contains("No note file") && !answer.contains("file link"), "{answer}");
    assert!(line(&mut a).contains("[ ] open document"), "the choice stays");
    a.open_file(root.join(B));
    a.tick();
    assert!(line(&mut a).contains("[x] open document"), "another document");
    a.open_file(root.join(A));
    a.tick();
    assert!(line(&mut a).contains("[x] open document"), "back to the first one: selected again");
    let r = a.frame.doc_line().unwrap();
    click(&mut a, r.x, r.y);
    assert!(line(&mut a).contains("[ ] open document"));
    std::fs::write(root.join(A), "# A\n\na longer text now\n").unwrap();
    a.tick();
    assert!(line(&mut a).contains("[x] open document"), "the text changed");
    let answer = ask(&mut a, "and now").await;
    assert!(answer.contains("file link"), "{answer}");
    typ(&mut a, "/");
    assert!(!line(&mut a).contains("open document"), "no line for a slash command");
    let _ = std::fs::remove_dir_all(&root);
}

// CLAUDE-62, CLAUDE-63, CLAUDE-65: the transcript shows a question with its chips as chips; a
// double-click on one opens its chip window; a chip that zendo does not know says so.
#[tokio::test]
async fn the_transcript_shows_chips_and_opens_them() {
    let root = repo("chip-transcript", &files());
    let mut a = chat(&root);
    a.on_paste("first line\nsecond line\n".into());
    ask(&mut a, "why").await;
    let (rows, _) = draw_buf(&mut a, 160, 30);
    let (r, _) = a.frame.transcript_chips().last().cloned().expect("a chip in the transcript");
    let on_screen: String = rows[r.y as usize].chars().skip(r.x as usize).take(r.width as usize).collect();
    assert!(on_screen.contains("first line") || rows[r.y as usize].contains("first line"), "the chip shows its label: {rows:#?}");
    click(&mut a, r.x + 1, r.y);
    click(&mut a, r.x + 1, r.y);
    let (rows, _) = draw_buf(&mut a, 160, 30);
    assert!(rows.iter().any(|r| r.contains("second line")), "the window: {rows:#?}");
    key(&mut a, KeyCode::Esc);
    a.on_agent(zendo::agent::Event::Replay { key: CHAT_KEY.into(), user: true, text: "see ⟦ates/specificat⟧ there".into() });
    draw_buf(&mut a, 160, 30);
    let n = a.frame.transcript_chips().len();
    let (r, _) = a.frame.transcript_chips()[n - 1].clone();
    click(&mut a, r.x + 1, r.y);
    click(&mut a, r.x + 1, r.y);
    let (rows, _) = draw_buf(&mut a, 160, 30);
    assert!(rows.iter().any(|r| r.contains("does not know")), "{rows:#?}");
    let _ = std::fs::remove_dir_all(&root);
}

// CLAUDE-64: a question of an earlier session shows each `⟦…⟧` in it as a chip with a label from
// its text, and `↩` for text from the chat.
#[tokio::test]
async fn a_replayed_question_shows_its_chips() {
    let root = repo("chip-replay", &files());
    let mut a = chat(&root);
    let text = "Question: look ⟦Text from this chat:\n> the old answer⟧ again";
    a.on_agent(zendo::agent::Event::Replay { key: CHAT_KEY.into(), user: true, text: text.into() });
    let (rows, _) = draw_buf(&mut a, 160, 30);
    assert!(rows.iter().any(|r| r.contains("› look")) && rows.iter().any(|r| r.contains("↩ the old answer")), "{rows:#?}");
    assert!(!rows.iter().any(|r| r.contains("Text from")), "the chip, not its text: {rows:#?}");
    let _ = std::fs::remove_dir_all(&root);
}
