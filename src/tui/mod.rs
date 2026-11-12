//! The terminal loop: input on its own thread, one draw for all waiting input, the output of the
//! Claude Code sessions, and the terminal modes.

use crate::app::App;
use crate::ui;
use anyhow::Result;
use ratatui::crossterm::event::{self, DisableBracketedPaste, DisableFocusChange, DisableMouseCapture, EnableBracketedPaste, EnableFocusChange, EnableMouseCapture, Event, KeyEventKind};
use ratatui::crossterm::execute;
use std::time::Duration;
use tokio::sync::mpsc;

/// Terminal input runs on its own thread.
pub fn spawn_input(tx: mpsc::UnboundedSender<Event>) {
    std::thread::spawn(move || {
        loop {
            match event::poll(Duration::from_millis(100)) {
                Ok(true) => match event::read() {
                    Ok(e) => {
                        if tx.send(e).is_err() {
                            return;
                        }
                    }
                    Err(_) => return,
                },
                Ok(false) => {}
                Err(_) => return,
            }
        }
    });
}

pub async fn run(
    term: &mut ratatui::DefaultTerminal,
    app: &mut App,
    keys: &mut mpsc::UnboundedReceiver<Event>,
    _keys_tx: mpsc::UnboundedSender<Event>,
) -> Result<()> {
    let mut tick = tokio::time::interval(Duration::from_millis(250));
    // While a button blinks: a frame each 100 ms, 20 shades in each period (ATTN-3).
    let mut pulse = tokio::time::interval(Duration::from_millis(100));
    pulse.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let printed = app.cli.notify.clone();
    loop {
        term.draw(|f| ui::draw(f, app))?;
        cursor_shape(app);
        if std::mem::take(&mut app.attention.bells) > 0 {
            bell();
        }
        if let Some(text) = app.clip.take() {
            crate::text::select::copy(&text);
            app.last_copy = Some(text.clone());
            let n = text.chars().count();
            app.flash(format!("copied {n} character{}", if n == 1 { "" } else { "s" }));
            continue;
        }
        if app.quit && app.settle_quit() {
            return Ok(());
        }
        tokio::select! {
            Some(e) = keys.recv() => {
                on_input(app, e);
                // A trackpad sends a burst of scroll events: take all that are here, then draw once.
                let until = std::time::Instant::now() + Duration::from_millis(12);
                while !app.quit && app.clip.is_none() && std::time::Instant::now() < until {
                    match keys.try_recv() {
                        Ok(e) => on_input(app, e),
                        Err(_) => break,
                    }
                }
            },
            Some(e) = async {
                match app.agent_events.as_mut() {
                    Some(rx) => rx.recv().await,
                    None => std::future::pending().await,
                }
            } => {
                app.on_agent(e);
                // Drain what else arrived so streaming text draws in one frame.
                while let Some(rx) = app.agent_events.as_mut() {
                    match rx.try_recv() {
                        Ok(e) => app.on_agent(e),
                        Err(_) => break,
                    }
                }
            },
            // A Claude Code session printed: draw its screen.
            _ = printed.notified() => {},
            _ = tick.tick() => app.tick(),
            _ = pulse.tick(), if app.attention.blinking() => {},
        }
    }
}

fn on_input(app: &mut App, e: Event) {
    match e {
        Event::Key(k) if k.kind != KeyEventKind::Release => app.on_key(k),
        Event::Paste(s) => app.on_paste(s),
        Event::Mouse(m) => app.on_mouse(m),
        Event::FocusGained => app.on_focus(true),
        Event::FocusLost => app.on_focus(false),
        _ => {}
    }
}

/// Paste, mouse, and the kitty keyboard protocol where the terminal has it: then ctrl-i is not tab,
/// and shift-enter is not enter.
pub fn terminal_on() {
    use ratatui::crossterm::event::{KeyboardEnhancementFlags, PushKeyboardEnhancementFlags};
    let _ = execute!(std::io::stdout(), EnableBracketedPaste, EnableMouseCapture, EnableFocusChange);
    if ratatui::crossterm::terminal::supports_keyboard_enhancement().unwrap_or(false) {
        let _ = execute!(std::io::stdout(), PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES));
        KITTY.store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

pub fn terminal_off() {
    use ratatui::crossterm::cursor::SetCursorStyle;
    use ratatui::crossterm::event::PopKeyboardEnhancementFlags;
    if KITTY.swap(false, std::sync::atomic::Ordering::Relaxed) {
        let _ = execute!(std::io::stdout(), PopKeyboardEnhancementFlags);
    }
    let _ = execute!(std::io::stdout(), DisableBracketedPaste, DisableMouseCapture, DisableFocusChange, SetCursorStyle::DefaultUserShape);
    SHAPE.store(0, std::sync::atomic::Ordering::Relaxed);
}

/// The terminal bell: the terminal plays its sound (ATTN-1).
fn bell() {
    use std::io::Write;
    let mut out = std::io::stdout();
    let _ = out.write_all(b"\x07").and_then(|_| out.flush());
}

/// The terminal has the kitty keyboard protocol: ctrl-shift-s is not ctrl-s.
pub fn kitty() -> bool {
    KITTY.load(std::sync::atomic::Ordering::Relaxed)
}

static KITTY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static SHAPE: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

/// A blinking block on the char in read mode; a blinking bar where you type.
fn cursor_shape(app: &App) {
    use ratatui::crossterm::cursor::SetCursorStyle;
    use crate::keys::Pane;
    let block = app.key_pane() == Pane::Read && app.mode == crate::app::Mode::Read;
    let want = if block { 1 } else { 2 };
    if SHAPE.swap(want, std::sync::atomic::Ordering::Relaxed) != want {
        let style = if block { SetCursorStyle::BlinkingBlock } else { SetCursorStyle::BlinkingBar };
        let _ = execute!(std::io::stdout(), style);
    }
}
