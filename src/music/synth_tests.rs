use super::*;

/// A low rate keeps the tests fast in a debug build; the synth works at any rate.
const RATE: u32 = 8000;

fn seconds(synth: &mut Synth, rate: u32, secs: f32) -> Vec<f32> {
    let mut out = vec![0.0; (rate as f32 * secs) as usize * 2];
    synth.render(&mut out);
    out
}

fn rms(samples: &[f32]) -> f32 {
    (samples.iter().map(|x| x * x).sum::<f32>() / samples.len() as f32).sqrt()
}

/// A second quieter than this (−54 dBFS) is a gap. The vinyl alone is near −60 dBFS; the quietest
/// second of the music, the snare, the hats and the tail of the piano at the end of a bar where the
/// second chord does not play, is near −50 dBFS.
const SILENT: f32 = 0.002;

/// Eight minutes at `rate` from a fixed seed, without an audio device: no second is silent, no
/// sample clips, no sample is NaN.
fn renders_without_gaps(rate: u32) {
    let mut synth = Synth::new(rate, 7);
    synth.set_playing(true);
    let out = seconds(&mut synth, rate, 8.0 * 60.0);
    assert!(out.iter().all(|x| x.is_finite()), "a sample is NaN or infinite");
    let loud = out.iter().filter(|x| x.abs() >= 0.99).count();
    assert_eq!(loud, 0, "{loud} samples clip");
    let quiet: Vec<(usize, f32)> = out
        .chunks(rate as usize * 2)
        .map(rms)
        .enumerate()
        .filter(|&(_, r)| r < SILENT)
        .take(10)
        .collect();
    assert!(quiet.is_empty(), "silent seconds: {quiet:?}");
}

// LOFI-2, LOFI-7: `lofi renders without gaps`, at a low rate, fast in a debug build.
#[test]
fn lofi_renders_without_gaps() {
    renders_without_gaps(RATE);
}

// LOFI-2, LOFI-7: the same at 48 kHz, the rate of the output, where the master low-pass is not
// clamped. Slow in a debug build: `cargo test --release --lib music -- --ignored`.
#[test]
#[ignore = "eight minutes at 48 kHz, for release builds"]
fn lofi_renders_without_gaps_at_48_khz() {
    renders_without_gaps(48_000);
}

// LOFI-13: a chime asked in the bar of Fmaj9 sounds on a note of the chord.
#[test]
fn lofi_chime_in_the_key() {
    let mut synth = Synth::new(RATE, 11);
    synth.set_playing(true);
    seconds(&mut synth, RATE, 0.5);
    assert_eq!(synth.chord(), [53, 57, 60, 64, 67], "the first bar is Fmaj9");
    assert_eq!(synth.last_chime(), None);
    synth.chime();
    seconds(&mut synth, RATE, 0.5);
    let note = synth.last_chime().expect("the chime sounded");
    let classes: Vec<u8> = synth.chord().iter().map(|n| n % 12).collect();
    assert!(classes.contains(&(note % 12)), "chime {note} is not in {classes:?}");
}

// LOFI-13: a chime asked at the end of a bar still sounds, on the next eighth note, in the new bar.
#[test]
fn a_chime_at_the_end_of_a_bar_sounds() {
    let mut synth = Synth::new(RATE, 5);
    synth.set_playing(true);
    // A bar is 16 sixteenths of 60 / 72 / 4 s; stop 10 ms before the end of the first one.
    seconds(&mut synth, RATE, 16.0 * 60.0 / 72.0 / 4.0 - 0.01);
    synth.chime();
    seconds(&mut synth, RATE, 0.1);
    let note = synth.last_chime().expect("the chime sounded");
    assert_eq!(synth.chord(), [52, 55, 59, 62, 66], "the second bar is Em9");
    assert!([71, 78].contains(&note), "chime {note} is not the fifth or the ninth of Em9");
}

// LOFI-8: the synth starts silent and fades out to silence when the music stops.
#[test]
fn the_synth_fades_out_to_silence() {
    let mut synth = Synth::new(RATE, 3);
    assert!(synth.silent());
    assert!(seconds(&mut synth, RATE, 0.5).iter().all(|&x| x == 0.0));
    synth.set_playing(true);
    assert!(!synth.silent());
    assert!(rms(&seconds(&mut synth, RATE, 2.0)) > 0.01);
    synth.set_playing(false);
    seconds(&mut synth, RATE, 3.0);
    assert!(synth.silent());
    assert!(seconds(&mut synth, RATE, 0.5).iter().all(|&x| x == 0.0));
}

// LOFI-7: the music plays again after a stop, from the first bar.
#[test]
fn the_synth_plays_again_after_a_stop() {
    let mut synth = Synth::new(RATE, 3);
    synth.set_playing(true);
    seconds(&mut synth, RATE, 5.0);
    synth.set_playing(false);
    seconds(&mut synth, RATE, 3.0);
    synth.set_playing(true);
    assert!(rms(&seconds(&mut synth, RATE, 2.0)) > 0.01);
    assert_eq!(synth.chord(), [53, 57, 60, 64, 67]);
}

// LOFI-4: the same seed gives the same samples, so a test can check the music; another seed
// gives other bars.
#[test]
fn the_same_seed_gives_the_same_music() {
    let play = |seed| {
        let mut synth = Synth::new(RATE, seed);
        synth.set_playing(true);
        seconds(&mut synth, RATE, 20.0)
    };
    assert!(play(42) == play(42));
    assert!(play(42) != play(43));
}

// LOFI-2, LOFI-4: the synth works at the usual rates of audio outputs, with buffers of any size.
#[test]
fn the_synth_works_at_any_rate() {
    for rate in [8000, 22050, 44100, 48000] {
        let mut synth = Synth::new(rate, 1);
        synth.set_playing(true);
        let mut out = vec![0.0; 2 * rate as usize];
        out.chunks_mut(202).for_each(|c| synth.render(c));
        assert!(out.iter().all(|x| x.is_finite() && x.abs() < 0.99), "rate {rate}");
        assert!(rms(&out) > 0.01, "rate {rate}");
    }
}

// LOFI-4: ten seconds at 48 kHz render far faster than real time. Run it in a release build:
// `cargo test --release music -- --ignored --nocapture`.
#[test]
#[ignore = "a timing, for release builds"]
fn render_time_at_48_khz() {
    let mut synth = Synth::new(48000, 9);
    synth.set_playing(true);
    let mut buf = [0.0f32; 1024];
    let start = std::time::Instant::now();
    (0..48000 * 10 * 2 / buf.len()).for_each(|_| synth.render(&mut buf));
    let took = start.elapsed();
    println!("10 s at 48 kHz rendered in {took:?}");
    assert!(took.as_secs_f32() < 1.0);
}
