use super::*;

// ATTN-9: the config sets the sound; without the setting the sound is on.
#[test]
fn the_config_sets_the_sound() {
    let off = crate::setup::config::UserConfig { sound: Some(false), ..Default::default() };
    assert!(!Attention::from_config(&off).sound);
    assert!(Attention::from_config(&Default::default()).sound);
}

// ATTN-2, ATTN-4: a blink keeps the time it started; `stop` ends it.
#[test]
fn a_blink_keeps_its_start() {
    let mut at = Attention::default();
    let then = Instant::now() - std::time::Duration::from_secs(1);
    at.blink_since(3, then);
    at.start(3);
    assert_eq!(at.since(3), Some(then), "a second end does not start the pulse again");
    at.stop(3);
    assert_eq!(at.since(3), None);
    assert!(!at.blinking());
}
