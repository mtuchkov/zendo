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

// FLOAT-16: each theme has a float color of its own, not the color of the note or of the side panes.
#[test]
fn each_theme_has_a_float_color_of_its_own() {
    for t in THEMES {
        assert!(t.float != t.bg && t.float != t.panel, "{}", t.name);
    }
}

/// The relative luminance of an RGB color (WCAG 2).
fn luminance(c: Color) -> f64 {
    let Color::Rgb(r, g, b) = c else { panic!("{c:?} is not RGB") };
    let lin = |v: u8| {
        let s = f64::from(v) / 255.0;
        if s <= 0.039_28 { s / 12.92 } else { ((s + 0.055) / 1.055).powf(2.4) }
    };
    0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b)
}

/// The contrast ratio of two colors (WCAG 2): 1 to 21.
fn contrast(a: Color, b: Color) -> f64 {
    let (x, y) = (luminance(a), luminance(b));
    (x.max(y) + 0.05) / (x.min(y) + 0.05)
}

// FLOAT-16: the float is sand, a light and warm color, and its text is dark ink that reads on it.
#[test]
fn the_float_is_sand_under_dark_ink() {
    for t in THEMES {
        let f = t.on_float();
        assert_eq!(f.bg, t.float, "{}", t.name);
        let Color::Rgb(r, g, b) = f.bg else { panic!("{}", t.name) };
        assert!(luminance(f.bg) > 0.5 && r > g && g > b, "{}: light and warm", t.name);
        assert!(contrast(f.fg, f.bg) >= 7.0, "{}: the text reads", t.name);
        assert!(contrast(f.dim, f.bg) >= 3.0, "{}: the dim text reads", t.name);
        assert!(contrast(f.line, f.bg) >= 1.5, "{}: the border shows", t.name);
    }
}

// FLOAT-2, FLOAT-16: code in the float stands on a panel of its own, and the float has a name of
// its own, so the render cache does not give it text in the colors of its theme.
#[test]
fn the_float_has_a_code_panel_and_a_name_of_its_own() {
    for t in THEMES {
        let f = t.on_float();
        assert_ne!(f.panel, f.bg, "{}: code stands out", t.name);
        assert_ne!(f.name, t.name, "{}: the render cache keeps them apart", t.name);
    }
}
