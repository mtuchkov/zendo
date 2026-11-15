//! The pointer without an app (ADR 0008).
use super::*;
use ratatui::layout::Rect;

fn press(p: &mut Pointer, x: u16, y: u16) {
    p.press(Some(Selection::start(Region::Note, Rect::new(0, 0, 80, 20), x, y)));
}

// ADR 0008, PANES-43: a drag and the release make a final selection; the next draw takes its text once.
#[test]
fn a_drag_and_the_release_make_a_final_selection() {
    let mut p = Pointer::default();
    press(&mut p, 2, 3);
    p.drag(9, 3, KeyModifiers::NONE);
    assert_eq!(p.release(KeyModifiers::NONE), Some((Region::Note, false)));
    assert!(p.sel().is_some_and(|s| s.done));
    assert!(p.take_copy_request() && !p.take_copy_request(), "the draw takes the text once");
    p.drag(30, 9, KeyModifiers::NONE);
    assert_eq!(p.sel().map(|s| s.head), Some((9, 3)), "a final selection does not move");
}

// ADR 0008: a click without a drag selects nothing.
#[test]
fn a_release_without_a_drag_leaves_no_selection() {
    let mut p = Pointer::default();
    press(&mut p, 2, 3);
    assert_eq!(p.release(KeyModifiers::NONE), None);
    assert!(p.sel().is_none() && !p.take_copy_request());
}

// ADR 0008, PANES-44: alt in the drag sends the selection to the chat.
#[test]
fn alt_sends_the_selection_to_the_chat() {
    let mut p = Pointer::default();
    press(&mut p, 2, 3);
    p.drag(9, 3, KeyModifiers::ALT);
    assert_eq!(p.release(KeyModifiers::ALT), Some((Region::Note, true)));
}

// ADR 0008, FLOAT-2: the screen behind the float draws without the selection, which comes back.
#[test]
fn the_selection_goes_aside_and_comes_back() {
    let mut p = Pointer::default();
    press(&mut p, 2, 3);
    p.drag(9, 3, KeyModifiers::NONE);
    p.release(KeyModifiers::NONE);
    let aside = p.set_aside();
    assert!(p.sel().is_none() && !p.take_copy_request());
    p.bring_back(aside);
    assert!(p.sel().is_some() && p.take_copy_request());
}

// ADR 0008: a fast run of scroll events is a trackpad: one line for each event.
#[test]
fn a_fast_run_of_scroll_events_moves_by_one_line() {
    let (mut p, t) = (Pointer::default(), Instant::now());
    assert_eq!(p.scroll_step(t), 3, "a wheel notch");
    assert_eq!(p.scroll_step(t + Duration::from_millis(10)), 1);
    assert_eq!(p.scroll_step(t + Duration::from_millis(200)), 3);
}

// ADR 0008: the text for the clipboard and for the chat waits until the event loop takes it.
#[test]
fn the_text_waits_for_the_event_loop() {
    let mut p = Pointer::default();
    p.copy("one".into());
    p.send_to_chat("two".into());
    assert_eq!(p.clip(), Some("one"));
    assert_eq!((p.take_clip().as_deref(), p.take_clip()), (Some("one"), None));
    assert_eq!((p.take_to_chat().as_deref(), p.take_to_chat()), (Some("two"), None));
    assert_eq!(p.last_copy(), None);
    p.copied("one".into());
    assert_eq!(p.last_copy().as_deref(), Some("one"));
}

// ADR 0008, PANES-10: the edge of a pane stays in the grab until the button is up.
#[test]
fn a_pane_edge_stays_in_the_grab_until_the_release() {
    let mut p = Pointer::default();
    p.grab((Focus::Claude, 100, false));
    assert_eq!(p.grabbed(), Some((Focus::Claude, 100, false)));
    p.let_go();
    assert_eq!(p.grabbed(), None);
}
