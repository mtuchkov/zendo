//! The audio output of the music (ADR 0006, LOFI-14): CoreAudio through `cpal` on macOS, a system
//! player on Linux. Each opens and runs on a thread of its own for the life of zendo, and reports
//! through the control whether it plays; over SSH there is none (LOFI-3, LOFI-4).

use super::Control;
use super::synth::Synth;
use std::sync::Arc;
use std::time::Duration;

/// How often the output thread looks at the commands while it does not play.
const POLL: Duration = Duration::from_millis(50);
/// How long after a stop the output rests: the fade out (time constant 0.35 s) reaches −60 dB in
/// about 2.4 s, so the synth is silent and starts again from its fade in.
#[cfg(target_os = "macos")]
const FADE: Duration = Duration::from_secs(3);

/// Over SSH the sound would play on the other machine, or nowhere.
fn ssh() -> bool {
    std::env::var_os("SSH_CONNECTION").is_some() || std::env::var_os("SSH_TTY").is_some()
}

/// A seed that differs at each start, so the music differs too.
fn seed() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(7, |d| d.as_nanos() as u64)
}

/// The output that the music would use, without playing: for `zendo doctor` (ADR 0006).
pub(super) fn probe() -> Result<String, String> {
    if ssh() {
        return Err("no audio output over SSH".into());
    }
    probe_system()
}

#[cfg(target_os = "macos")]
fn probe_system() -> Result<String, String> {
    use cpal::traits::HostTrait;
    cpal::default_host().default_output_device().map(|_| "CoreAudio".to_string()).ok_or_else(|| "no audio output".to_string())
}

#[cfg(not(target_os = "macos"))]
fn probe_system() -> Result<String, String> {
    let path = std::env::var_os("PATH").unwrap_or_default();
    let found = |p: &&(&str, &[&str])| std::env::split_paths(&path).any(|d| d.join(p.0).is_file());
    PLAYERS.iter().find(found).map(|p| p.0.to_string()).ok_or_else(|| "no audio player: paplay, aplay or pw-cat".into())
}

/// Open the output on a thread of its own, without waiting for it: the thread reports to `control`
/// whether it plays (LOFI-4).
pub(super) fn start(control: Arc<Control>) {
    if ssh() {
        return control.report(false);
    }
    let thread = control.clone();
    if std::thread::Builder::new().name("zendo-music".into()).spawn(move || run(thread)).is_err() {
        control.report(false);
    }
}

/// Render `frames` stereo frames into `out`, which has `channels` samples in each frame.
fn fill(synth: &mut Synth, stereo: &mut Vec<f32>, out: &mut [f32], channels: usize) {
    let frames = out.len() / channels.max(1);
    stereo.resize(frames * 2, 0.0);
    synth.render(stereo);
    for (frame, lr) in out.chunks_mut(channels.max(1)).zip(stereo.chunks(2)) {
        for (i, s) in frame.iter_mut().enumerate() {
            *s = lr[i.min(1)];
        }
    }
}

#[cfg(target_os = "macos")]
fn run(control: Arc<Control>) {
    match open_stream(control.clone()) {
        Ok(stream) => {
            control.report(true);
            keep(&control, &stream);
        }
        Err(_) => control.report(false),
    }
}

/// The CoreAudio stream of the default output; an output that goes away reports it (LOFI-3).
#[cfg(target_os = "macos")]
fn open_stream(control: Arc<Control>) -> Result<cpal::Stream, String> {
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    let device = cpal::default_host().default_output_device().ok_or("no audio output")?;
    let supported = device.default_output_config().map_err(|e| e.to_string())?;
    if supported.sample_format() != cpal::SampleFormat::F32 {
        return Err(format!("the audio output takes {}, not f32", supported.sample_format()));
    }
    let config = supported.config();
    let channels = config.channels as usize;
    let mut synth = Synth::new(config.sample_rate, seed());
    let (mut chimes, mut stereo) = (control.chimes(), Vec::with_capacity(16384));
    let lost = control.clone();
    let data = move |out: &mut [f32], _: &cpal::OutputCallbackInfo| {
        super::follow(&control, &mut synth, &mut chimes);
        fill(&mut synth, &mut stereo, out, channels);
    };
    let error = move |e: cpal::Error| {
        if matches!(e.kind(), cpal::ErrorKind::DeviceNotAvailable | cpal::ErrorKind::HostUnavailable) {
            lost.report(false);
        }
    };
    let stream = device.build_output_stream(config, data, error, None).map_err(|e| e.to_string())?;
    stream.play().map_err(|e| e.to_string())?;
    Ok(stream)
}

/// Keep the stream for the life of zendo. While the music is off or paused, the stream rests after
/// the fade out, so the system can sleep; it plays again with the music.
#[cfg(target_os = "macos")]
fn keep(control: &Control, stream: &cpal::Stream) {
    use cpal::traits::StreamTrait;
    let (mut off, mut resting) = (None::<std::time::Instant>, false);
    while !control.failed() {
        std::thread::sleep(POLL);
        if control.playing() {
            off = None;
            if std::mem::take(&mut resting) && stream.play().is_err() {
                control.report(false);
            }
        } else if !resting && off.get_or_insert_with(std::time::Instant::now).elapsed() >= FADE {
            resting = stream.pause().is_ok();
        }
    }
}

/// The sample rate of the samples that go to a player on Linux.
#[cfg(not(target_os = "macos"))]
const RATE: u32 = 48_000;

/// The players that read 16-bit stereo samples from stdin, in the order zendo tries them, each with
/// a latency of 100 ms, so a stop, a pause and a chime come at once (LOFI-13, LOFI-15). `paplay`
/// comes first: it plays through PipeWire too.
#[cfg(not(target_os = "macos"))]
const PLAYERS: [(&str, &[&str]); 3] = [
    ("paplay", &["--raw", "--format=s16le", "--rate=48000", "--channels=2", "--latency-msec=100"]),
    ("aplay", &["-q", "-t", "raw", "-f", "S16_LE", "-r", "48000", "-c", "2", "-B", "100000"]),
    ("pw-cat", &["--playback", "--raw", "--format", "s16", "--rate", "48000", "--channels", "2", "--latency", "100ms", "-"]),
];

/// Start a player; None when it does not start, or ends at once, as without a sound server.
#[cfg(not(target_os = "macos"))]
fn player((program, args): &(&str, &[&str])) -> Option<std::process::Child> {
    use std::process::{Command, Stdio};
    let mut child = Command::new(program).args(*args).stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().ok()?;
    std::thread::sleep(Duration::from_millis(150));
    match child.try_wait() {
        Ok(None) => Some(child),
        _ => None,
    }
}

#[cfg(not(target_os = "macos"))]
fn run(control: Arc<Control>) {
    let Some(mut child) = PLAYERS.iter().find_map(player) else { return control.report(false) };
    if let Some(pipe) = child.stdin.take() {
        control.report(true);
        feed(&control, pipe);
    }
    // The player died or has no input: the music is not available, and the bell rings (LOFI-3).
    control.report(false);
    let _ = child.kill();
    let _ = child.wait();
}

/// Write the music into the pipe of the player until the player goes away. The pipe blocks while
/// the player is full, so the music keeps its pace; while the music is silent, nothing goes.
#[cfg(not(target_os = "macos"))]
fn feed(control: &Control, mut pipe: std::process::ChildStdin) {
    use std::io::Write;
    let mut synth = Synth::new(RATE, seed());
    let (mut chimes, mut stereo, mut block) = (control.chimes(), vec![0.0f32; 2048], vec![0.0f32; 2048]);
    let mut bytes = Vec::with_capacity(4096);
    loop {
        super::follow(control, &mut synth, &mut chimes);
        if !control.playing() && synth.silent() {
            std::thread::sleep(POLL);
            continue;
        }
        fill(&mut synth, &mut stereo, &mut block, 2);
        bytes.clear();
        bytes.extend(block.iter().flat_map(|s| ((s.clamp(-1.0, 1.0) * 32767.0) as i16).to_le_bytes()));
        if pipe.write_all(&bytes).is_err() {
            return;
        }
    }
}
