//! The clipboard: copy with OSC 52 and arboard, the read of the clipboard, dropped files, a pasted
//! image (ADR 0024).

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn base64(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
        let n = (b[0] as u32) << 16 | (b[1] as u32) << 8 | b[2] as u32;
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(B64[(n >> (18 - 6 * i) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

/// What the system clipboard holds.
#[derive(Clone, Debug, PartialEq)]
pub enum Clip {
    /// An image, saved as a PNG file in the data folder.
    Image(std::path::PathBuf),
    Text(String),
    Empty,
    /// The clipboard cannot be read: no display (SSH), or an error of the system.
    Failed(String),
}

/// Copy with OSC 52 (most terminals, also over SSH) and to the system clipboard through arboard, on
/// a thread of its own (ADR 0024). On Linux the thread holds the copy until another program copies.
pub fn copy(text: &str) {
    use std::io::Write;
    let mut out = std::io::stdout();
    let _ = write!(out, "\x1b]52;c;{}\x07", base64(text.as_bytes()));
    let _ = out.flush();
    let text = text.to_string();
    let _ = std::thread::Builder::new().name("zendo-copy".into()).spawn(move || {
        let Ok(mut clipboard) = arboard::Clipboard::new() else { return };
        #[cfg(target_os = "linux")]
        let _ = arboard::SetExtLinux::wait(clipboard.set()).text(text);
        #[cfg(not(target_os = "linux"))]
        let _ = clipboard.set_text(text);
    });
}

/// Read the system clipboard through arboard: an image first, then the text. It can wait for the
/// program that owns the clipboard, so it runs on a worker (IO-1, ADR 0024).
pub fn read() -> Clip {
    let mut clipboard = match arboard::Clipboard::new() {
        Ok(c) => c,
        Err(e) => return Clip::Failed(e.to_string()),
    };
    if let Ok(image) = clipboard.get_image() {
        return save_png(image).map_or_else(Clip::Failed, Clip::Image);
    }
    match clipboard.get_text() {
        Ok(t) if !t.is_empty() => Clip::Text(t),
        Ok(_) | Err(arboard::Error::ContentNotAvailable) => Clip::Empty,
        Err(e) => Clip::Failed(e.to_string()),
    }
}

/// A pasted image as a PNG file in the data folder.
fn save_png(image: arboard::ImageData) -> Result<std::path::PathBuf, String> {
    let dir = crate::setup::config::data_dir().join("pastes");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let file = dir.join(format!("paste-{}.png", chrono::Local::now().format("%Y%m%d-%H%M%S-%3f")));
    let (w, h) = (u32::try_from(image.width).map_err(|e| e.to_string())?, u32::try_from(image.height).map_err(|e| e.to_string())?);
    let rgba = image::RgbaImage::from_raw(w, h, image.bytes.into_owned()).ok_or("the image of the clipboard has a wrong size")?;
    rgba.save_with_format(&file, image::ImageFormat::Png).map_err(|e| e.to_string())?;
    Ok(file)
}

/// The files of a paste, when the paste is nothing but absolute paths: what a terminal pastes when
/// you drop files on it (`/a/b.png`, `/a/My\ File.md`, `'/a/b c.md'`, `file:///a/b.png`).
pub fn dropped_files(s: &str) -> Option<Vec<std::path::PathBuf>> {
    use std::path::PathBuf;
    let t = s.trim();
    if t.is_empty() {
        return None;
    }
    let home = std::env::var("HOME").unwrap_or_default();
    let clean = |w: &str| -> PathBuf {
        let w = w.strip_prefix("file://").map(|r| r.replace("%20", " ")).unwrap_or_else(|| w.to_string());
        match w.strip_prefix("~/") {
            Some(r) => PathBuf::from(&home).join(r),
            None => PathBuf::from(w),
        }
    };
    let ok = |p: &PathBuf| p.is_absolute() && p.is_file();
    // One path, spaces and all.
    let one = clean(t.trim_matches(|c| c == '\'' || c == '"'));
    if ok(&one) {
        return Some(vec![one]);
    }
    // Words as a shell splits them: quotes, backslashes, and ASCII blanks only, so that a narrow
    // no-break space of a macOS file name stays in the name.
    let mut words = Vec::new();
    let mut cur = String::new();
    let (mut quote, mut esc, mut any) = (None::<char>, false, false);
    for c in t.chars() {
        if esc {
            cur.push(c);
            esc = false;
        } else if c == '\\' && quote != Some('\'') {
            esc = true;
        } else if let Some(q) = quote {
            if c == q { quote = None } else { cur.push(c) }
        } else if c == '\'' || c == '"' {
            quote = Some(c);
            any = true;
        } else if c.is_ascii_whitespace() {
            if !cur.is_empty() || any {
                words.push(std::mem::take(&mut cur));
            }
            any = false;
        } else {
            cur.push(c);
        }
    }
    if !cur.is_empty() || any {
        words.push(cur);
    }
    let paths: Vec<PathBuf> = words.iter().map(|w| clean(w)).collect();
    (!paths.is_empty() && paths.iter().all(ok)).then_some(paths)
}

/// Open a file in the program of the system for its kind: `open` on macOS, `xdg-open` elsewhere
/// (CLAUDE-52). The program runs on its own; zendo does not wait for it.
pub fn open_outside(path: &std::path::Path) -> std::io::Result<()> {
    use std::process::{Command, Stdio};
    let program = if cfg!(target_os = "macos") { "open" } else { "xdg-open" };
    let mut child = Command::new(program).arg(path).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn()?;
    std::thread::spawn(move || child.wait());
    Ok(())
}
