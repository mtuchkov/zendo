use super::*;
use std::time::Duration;

// ATTN-3: the pulse of a blinking button goes from 0 to 1 in 1 s and back to 0 in 2 s, smoothly.
#[test]
fn the_pulse_goes_from_0_to_1_and_back_in_2_s() {
    let at = |ms: u64| pulse(Duration::from_millis(ms));
    assert!(at(0).abs() < 1e-6);
    assert!((at(1000) - 1.0).abs() < 1e-6);
    assert!(at(2000).abs() < 1e-6);
    assert!((at(500) - 0.5).abs() < 1e-6);
    let rise: Vec<f32> = (0..=20).map(|i| at(50 * i)).collect();
    assert!(rise.windows(2).all(|w| w[0] < w[1]), "rises in the first second: {rise:?}");
}

// ATTN-3: a mix of two colors.
#[test]
fn mix_gives_the_ends_and_the_middle() {
    let (a, b) = (Color::Rgb(0, 0, 0), Color::Rgb(200, 100, 50));
    assert_eq!(mix(a, b, 0.0), a);
    assert_eq!(mix(a, b, 1.0), b);
    assert_eq!(mix(a, b, 0.5), Color::Rgb(100, 50, 25));
    assert_eq!(mix(a, b, 2.0), b, "k stays in 0 to 1");
    assert_eq!(mix(Color::Reset, b, 0.4), Color::Reset, "no RGB: the nearer end");
    assert_eq!(mix(Color::Reset, b, 0.6), b);
}
