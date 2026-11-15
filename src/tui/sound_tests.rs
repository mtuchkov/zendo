use super::*;

// ATTN-12: on a Mac, and not over SSH, zendo plays the sound itself; elsewhere the terminal bell rings.
#[test]
fn zendo_plays_the_sound_itself_only_on_a_local_mac() {
    assert_eq!(player("macos", false), Some(("afplay", "/System/Library/Sounds/Glass.aiff")));
    assert_eq!(player("macos", true), None, "over SSH the sound would play on the other machine");
    assert_eq!(player("linux", false), None);
}

// ATTN-12: a player that is not there does not fail: the terminal bell rings in its place.
#[test]
fn a_missing_player_gives_the_bell() {
    assert!(!play(Some(("zendo-no-such-player", "x"))));
    assert!(!play(None));
}
