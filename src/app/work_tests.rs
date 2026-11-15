//! The work without an app (ADR 0008).
use super::*;
use chrono::Local;

fn job(rel: &str) -> RecapJob {
    RecapJob { rel: rel.into(), ..RecapJob::default() }
}

// ADR 0008: the delta opens at its top; the other base starts at the top too; the draw keeps the
// scroll in the text.
#[test]
fn the_delta_opens_at_the_top_and_stays_in_the_text() {
    let mut w = Work::default();
    assert!(w.toggle_delta() && w.delta_open());
    w.scroll_delta(false, 30);
    w.scroll_delta(true, 5);
    assert_eq!(w.delta_scroll(), 25);
    assert_eq!(w.fit_delta(10), 10, "the last first row");
    w.delta_bottom();
    assert_eq!(w.fit_delta(40), 40);
    assert_eq!(w.base(), Base::LastVisit);
    w.switch_base();
    assert_eq!((w.base(), w.delta_scroll()), (Base::GitHead, 0));
    w.scroll_delta(true, 3);
    assert_eq!(w.delta_scroll(), 0);
    w.scroll_delta(false, 3);
    assert!(!w.toggle_delta() && w.delta_scroll() == 0);
    w.toggle_delta();
    w.close_delta();
    assert!(!w.delta_open());
}

// ADR 0008: a visit counts as work after an edit, or after enough scrolls and clicks.
#[test]
fn a_visit_counts_after_an_edit_or_enough_moves() {
    let mut w = Work::default();
    w.interact(100);
    assert!(!w.visit_counts(), "no visit: nothing to count");
    w.start_visit(Some("a.md".into()));
    w.interact(ENGAGED - 1);
    assert!(!w.visit_counts());
    w.interact(1);
    assert!(w.visit_counts());
    w.start_visit(Some("b.md".into()));
    assert!(!w.visit_counts(), "a new visit starts from nothing");
    w.mark_edited();
    assert!(w.visit_counts());
    assert_eq!(w.take_visit().map(|v| v.path), Some("b.md".into()));
    assert!(w.visit().is_none());
}

// ADR 0008: the agent has one job in the background; a recap that is in work or waits is not asked again.
#[test]
fn a_recap_waits_in_the_queue_or_in_the_job() {
    let mut w = Work::default();
    assert!(w.free() && !w.recap_waits("a.md"));
    w.ask_recap("a.md".into());
    assert!(w.recap_waits("a.md") && w.free());
    assert_eq!(w.next_recap().as_deref(), Some("a.md"));
    assert!(!w.recap_waits("a.md") && w.next_recap().is_none());
    w.start(job("a.md"));
    assert!(w.recap_waits("a.md") && !w.free());
    w.job_mut().unwrap().buf.push_str("text");
    assert_eq!(w.job().map(|j| j.buf.as_str()), Some("text"));
    assert_eq!(w.take_job().map(|j| j.rel), Some("a.md".into()));
    assert!(w.free());
}

// ADR 0008: an automatic recap waits until the user is idle, and the hour has a limit.
#[test]
fn automatic_recaps_wait_for_an_idle_user_and_have_a_limit() {
    let mut w = Work::default();
    assert!(!w.auto_allowed(), "a key came now");
    w.input_at(Instant::now() - IDLE - Duration::from_secs(1));
    assert!(w.auto_allowed());
    (0..AUTO_PER_HOUR).for_each(|_| w.count_auto());
    assert!(!w.auto_allowed(), "the hour is full");
    w.input_now();
    assert!(!w.auto_allowed());
}

// ADR 0008: the second press of the review key within the chord asks; one review of a file waits
// at a time; the review goes when another file is open.
#[test]
fn a_review_needs_two_presses_and_goes_with_its_file() {
    let mut w = Work::default();
    assert!(!w.review_chord(Duration::from_secs(2)), "the first press waits");
    assert!(w.review_chord(Duration::from_secs(2)));
    assert!(!w.review_chord(Duration::ZERO) && !w.review_chord(Duration::ZERO), "too late: a first press again");
    assert!(w.ask_review("a.md".into()) && !w.ask_review("a.md".into()));
    assert!(w.review_waits("a.md"));
    assert_eq!((w.next_review().as_deref(), w.next_review()), (Some("a.md"), None));
    w.show_review(Review { rel: "a.md".into(), summary: "s".into(), lines: vec![], at: Local::now() });
    w.keep_review_of(Some("a.md"));
    assert!(w.review().is_some());
    w.keep_review_of(Some("b.md"));
    assert!(w.review().is_none());
}
