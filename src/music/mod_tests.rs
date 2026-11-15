use super::*;
use crate::setup::config::UserConfig;

// LOFI-11: without the setting the music is on and zendo has not asked yet; `music = false` keeps
// it off.
#[test]
fn the_config_sets_the_music() {
    let fresh = Music::from_config(&UserConfig::default());
    assert!(fresh.on && !fresh.asked);
    let off = Music::from_config(&UserConfig { music: Some(false), ..Default::default() });
    assert!(!off.on && off.asked);
}

// LOFI-3: without an output the music is not available and does not sound.
#[test]
fn no_output_means_not_available() {
    let mut m = Music::default();
    m.start(Output::Missing);
    assert_eq!(m.state(), State::Unavailable);
    m.set(true);
    assert!(!m.sounds());
}

// LOFI-8, LOFI-15: the music sounds while it is on and the terminal has the focus.
#[test]
fn the_music_follows_the_setting_and_the_focus() {
    let mut m = Music::default();
    m.start(Output::Test);
    assert_eq!((m.state(), m.sounds()), (State::Plays, true));
    m.focus(false);
    assert_eq!((m.state(), m.sounds()), (State::Plays, false));
    m.focus(true);
    m.set(false);
    assert_eq!((m.state(), m.sounds()), (State::Stopped, false));
}

// LOFI-13: a chime goes to the music only while it sounds.
#[test]
fn a_chime_goes_to_the_music_only_while_it_sounds() {
    let mut m = Music::default();
    m.start(Output::Test);
    assert!(m.chime());
    m.set(false);
    assert!(!m.chime());
    assert_eq!(m.chimes(), 1);
}

// LOFI-4, LOFI-13: the audio thread takes the commands of the control before each block.
#[test]
fn the_synth_follows_the_control() {
    let mut m = Music::default();
    m.start(Output::Test);
    m.chime();
    let mut s = synth::Synth::new(8000, 1);
    let mut seen = 0;
    follow(&m.control, &mut s, &mut seen);
    assert_eq!(seen, 1);
    follow(&m.control, &mut s, &mut seen);
    assert_eq!(seen, 1);
}

// LOFI-4, LOFI-3: while the output opens, the music is not available yet; a failed output stays so.
#[test]
fn an_opening_output_is_not_available_yet() {
    let c = Control::default();
    assert!(!c.open() && !c.failed());
    c.report(false);
    assert!(c.failed());
}

// LOFI-3, LOFI-13: an output that died while it opened stays dead: a late report that it plays
// does not hide the failure, so the bell still rings.
#[test]
fn a_failure_is_not_overwritten_by_a_late_open() {
    let c = Control::default();
    c.report(false);
    c.report(true);
    assert!(c.failed() && !c.open());
}
