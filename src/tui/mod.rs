//! The terminal loop: input on its own thread, one draw for all waiting input, the hand-off to the
//! agent's command line, and the terminal modes.

use crate::app::App;
use crate::ui;
use anyhow::Result;
use ratatui::crossterm::event::{self, DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture, Event, KeyEventKind};
use ratatui::crossterm::execute;
use std::time::Duration;
use tokio::sync::mpsc;

/// Terminal input runs on its own thread; it pauses while Claude Code has the terminal.
pub fn spawn_input(tx: mpsc::UnboundedSender<Event>) {
    std::thread::spawn(move || {
        loop {
            if PAUSED.load(std::sync::atomic::Ordering::Relaxed) {
                std::thread::sleep(Duration::from_millis(50));
                continue;
            }
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

static PAUSED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub async fn run(
    term: &mut ratatui::DefaultTerminal,
    app: &mut App,
    keys: &mut mpsc::UnboundedReceiver<Event>,
    _keys_tx: mpsc::UnboundedSender<Event>,
) -> Result<()> {
    let mut tick = tokio::time::interval(Duration::from_millis(250));
    loop {
        term.draw(|f| ui::draw(f, app))?;
        cursor_shape(app);
        if let Some(text) = app.clip.take() {
            crate::text::select::copy(&text);
            app.last_copy = Some(text.clone());
            let n = text.chars().count();
            app.flash(format!("copied {n} character{}", if n == 1 { "" } else { "s" }));
            continue;
        }
        if app.quit {
            return Ok(());
        }
        if let Some(id) = app.handoff.take() {
            handoff(term, app, &id)?;
            continue;
        }
        tokio::select! {
            Some(e) = keys.recv() => {
                on_input(app, e);
                // A trackpad sends a burst of scroll events: take all that are here, then draw once.
                let until = std::time::Instant::now() + Duration::from_millis(12);
                while !app.quit && app.handoff.is_none() && app.clip.is_none() && std::time::Instant::now() < until {
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
            _ = tick.tick() => app.tick(),
        }
    }
}

fn on_input(app: &mut App, e: Event) {
    match e {
        Event::Key(k) if k.kind != KeyEventKind::Release => app.on_key(k),
        Event::Paste(s) => app.on_paste(s),
        Event::Mouse(m) => app.on_mouse(m),
        _ => {}
    }
}

/// Give the terminal to `claude --resume <id>`, then take it back.
fn handoff(term: &mut ratatui::DefaultTerminal, app: &mut App, id: &str) -> Result<()> {
    PAUSED.store(true, std::sync::atomic::Ordering::Relaxed);
    std::thread::sleep(Duration::from_millis(150));
    terminal_off();
    ratatui::restore();
    let Some((cli, args)) = app.handoff_cli() else { return Ok(()) };
    let status = std::process::Command::new(cli).args(args).arg(id).current_dir(&app.cfg.root).status();
    *term = ratatui::init();
    terminal_on();
    term.clear()?;
    PAUSED.store(false, std::sync::atomic::Ordering::Relaxed);
    match status {
        Ok(s) if s.success() => app.flash(format!("back from {cli}")),
        Ok(s) => app.flash(format!("{cli} exited with {s}")),
        Err(e) => app.flash(format!("could not start {cli}: {e}")),
    }
    app.reload();
    Ok(())
}

/// Paste, mouse, and the kitty keyboard protocol where the terminal has it: then ctrl-i is not tab,
/// and shift-enter is not enter.
pub fn terminal_on() {
    use ratatui::crossterm::event::{KeyboardEnhancementFlags, PushKeyboardEnhancementFlags};
    let _ = execute!(std::io::stdout(), EnableBracketedPaste, EnableMouseCapture);
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
    let _ = execute!(std::io::stdout(), DisableBracketedPaste, DisableMouseCapture, SetCursorStyle::DefaultUserShape);
    SHAPE.store(0, std::sync::atomic::Ordering::Relaxed);
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
