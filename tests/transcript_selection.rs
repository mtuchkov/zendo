//! The selection in the chat transcript: a range of its rows, frozen while the transcript has the
//! keys (docs/zendo/panes/transcript-selection.md, ADR 0026).

mod common;
use common::*;
use ratatui::crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;
use zendo::app::{App, Focus, Item};
use zendo::text::select::Region;

fn mouse(a: &mut App, kind: MouseEventKind, x: u16, y: u16) {
    a.on_mouse(MouseEvent { kind, column: x, row: y, modifiers: KeyModifiers::NONE });
}

fn body(a: &App) -> Rect {
    a.frame.regions().iter().find(|(r, _)| *r == Region::Panel).map(|(_, r)| *r).expect("the transcript is on the screen")
}

/// The rows of the transcript on the screen.
fn shown(screen: &[String], b: Rect) -> Vec<String> {
    screen[b.y as usize..b.bottom() as usize].iter().map(|r| r.chars().skip(b.x as usize).take(b.width as usize).collect::<String>().trim_end().to_string()).collect()
}

/// The row and the column of `text` on the screen.
fn at(screen: &[String], text: &str) -> (u16, u16) {
    let y = screen.iter().position(|r| r.contains(text)).unwrap_or_else(|| panic!("{text} is not on the screen: {screen:#?}"));
    let x = screen[y][..screen[y].find(text).unwrap()].chars().count();
    (x as u16, y as u16)
}

/// The chat pane with a thread of `n` rows of text, each with a blank row after it.
fn chat_with_rows(name: &str, n: usize) -> (std::path::PathBuf, App) {
    let root = repo(name, &[("docs/notes/n.md", "# N\n")]);
    let mut a = app(&root);
    a.threads.show();
    a.focus = Focus::Claude;
    let k = a.key();
    a.threads.entry(&k).items.extend((0..n).map(|i| Item::Info(format!("row {i}"))));
    (root, a)
}

/// The number of the first row of text in view.
fn first_number(rows: &[String]) -> usize {
    let row = rows.iter().find(|r| !r.trim().is_empty()).expect("rows with text");
    row.trim().trim_start_matches("row ").parse().unwrap_or_else(|_| panic!("not a numbered row: {row:?}"))
}

// PANES-50: a mouse selection in the transcript is a range of its rows, and reaches rows out of view.
// PANES-52: a drag held at the top row of the pane scrolls the transcript, and the selection grows.
#[test]
fn a_selection_in_the_transcript_reaches_rows_out_of_view() {
    let (root, mut a) = chat_with_rows("transcript-out-of-view", 50);
    let screen = draw(&mut a, 140, 30);
    let b = body(&a);
    let first = first_number(&shown(&screen, b));
    let (x, y) = at(&screen, "row 45");
    mouse(&mut a, MouseEventKind::Down(MouseButton::Left), x + 5, y);
    mouse(&mut a, MouseEventKind::Drag(MouseButton::Left), x, b.y);
    for _ in 0..30 {
        a.edge_scroll();
        draw(&mut a, 140, 30);
    }
    let screen = draw(&mut a, 140, 30);
    let top = first_number(&shown(&screen, b));
    assert!(top + 10 < first, "the view shows earlier rows: {top} after {first}");
    mouse(&mut a, MouseEventKind::Up(MouseButton::Left), x, b.y);
    draw(&mut a, 140, 30);
    let clip = a.pointer.clip().expect("the release copies the selection").to_string();
    assert!(clip.starts_with(&format!("row {top}\n")), "from the row under the pointer: {clip}");
    assert!(clip.ends_with("row 45") && !clip.contains("row 46"), "to the row of the start: {clip}");
    let _ = std::fs::remove_dir_all(&root);
}

// PANES-51: while a selection is in the transcript, its rows stay, and new output waits below.
// PANES-53: the last row of the pane counts the new rows. PANES-54: `End` takes them in.
// PANES-58: the rows in view do not move.
#[tokio::test]
async fn new_output_waits_below_a_selection() {
    let (root, mut a) = chat_with_rows("transcript-new-output", 40);
    a.open_panel();
    a.focus = Focus::Claude;
    typ(&mut a, "hello");
    key(&mut a, KeyCode::Enter);
    let screen = draw(&mut a, 140, 30);
    let b = body(&a);
    let (x, y) = at(&screen, "row 35");
    mouse(&mut a, MouseEventKind::Down(MouseButton::Left), x, y);
    mouse(&mut a, MouseEventKind::Drag(MouseButton::Left), x + 3, y + 2);
    mouse(&mut a, MouseEventKind::Up(MouseButton::Left), x + 3, y + 2);
    let before = shown(&draw(&mut a, 140, 30), b);
    let selected = a.threads.transcript().text();
    assert!(selected.starts_with("row 35") && selected.ends_with("row"), "{selected}");
    let frozen = a.threads.transcript().rows().len();
    let k = a.key();
    until(&mut a, |a| a.threads.get(&k).is_some_and(|t| t.busy.is_none() && t.items.iter().any(|i| matches!(i, Item::Agent(_))))).await;
    let after = shown(&draw(&mut a, 140, 30), b);
    let last = after.len() - 1;
    assert_eq!(after[..last], before[..last], "the rows in view stay");
    assert!(after[last].contains('↓') && after[last].contains("new row") && after[last].ends_with("· End"), "{}", after[last]);
    assert_eq!(a.threads.transcript().text(), selected, "the selection stays on its text");
    key(&mut a, KeyCode::End);
    draw(&mut a, 140, 30);
    let tr = a.threads.transcript();
    let end = tr.rows().iter().rposition(|r| !r.trim().is_empty()).expect("rows with text");
    assert!(tr.rows().len() > frozen, "the new rows are in the transcript");
    assert_eq!(tr.cursor(), (end, 0), "the cursor is on the last row with text");
    assert!(tr.text().starts_with("row 35"), "the selection keeps its start");
    let _ = std::fs::remove_dir_all(&root);
}
