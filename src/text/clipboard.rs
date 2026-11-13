//! The clipboard: copy with OSC 52 and the system tool, paste, dropped files, a pasted image.

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

/// Copy with OSC 52 (most terminals, also over SSH) and with the system tool if there is one.
pub fn copy(text: &str) {
    use std::io::Write;
    let mut out = std::io::stdout();
    let _ = write!(out, "\x1b]52;c;{}\x07", base64(text.as_bytes()));
    let _ = out.flush();
    let text = text.to_string();
    std::thread::spawn(move || {
        let tools: [(&str, &[&str]); 4] = [
            ("pbcopy", &[]),
            ("wl-copy", &[]),
            ("xclip", &["-selection", "clipboard"]),
            ("xsel", &["--clipboard", "--input"]),
        ];
        for (cmd, args) in tools {
            let child = std::process::Command::new(cmd)
                .args(args)
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn();
            if let Ok(mut child) = child {
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(text.as_bytes());
                }
                if child.wait().is_ok_and(|s| s.success()) {
                    return;
                }
            }
        }
    });
}

/// Read the system clipboard with the system tool. None over SSH or without a tool.
pub fn paste() -> Option<String> {
    let tools: [(&str, &[&str]); 4] = [("pbpaste", &[]), ("wl-paste", &["--no-newline"]), ("xclip", &["-o", "-selection", "clipboard"]), ("xsel", &["--clipboard", "--output"])];
    for (cmd, args) in tools {
        let out = std::process::Command::new(cmd).args(args).stdin(std::process::Stdio::null()).stderr(std::process::Stdio::null()).output();
        if let Ok(o) = out
            && o.status.success()
        {
            return Some(String::from_utf8_lossy(&o.stdout).into_owned());
        }
    }
    None
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
    // Words as a shell splits them: quotes and backslashes.
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
        } else if c.is_whitespace() {
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

/// An image on the system clipboard, saved as a PNG file. None when there is none, or no tool.
pub fn paste_image() -> Option<std::path::PathBuf> {
    use std::process::{Command, Stdio};
    let dir = crate::setup::config::data_dir().join("pastes");
    std::fs::create_dir_all(&dir).ok()?;
    let file = dir.join(format!("paste-{}.png", chrono::Local::now().format("%Y%m%d-%H%M%S")));
    let run = |cmd: &str, args: &[&str]| Command::new(cmd).args(args).stdin(Stdio::null()).stderr(Stdio::null()).output().ok().filter(|o| o.status.success()).map(|o| o.stdout);
    let saved = |bytes: Vec<u8>| (!bytes.is_empty() && std::fs::write(&file, bytes).is_ok()).then(|| file.clone());
    if cfg!(target_os = "macos") {
        let f = file.display().to_string();
        let script = format!("set png to (the clipboard as «class PNGf»)\nset fh to open for access POSIX file \"{f}\" with write permission\nwrite png to fh\nclose access fh");
        if run("osascript", &["-e", &script]).is_some() && std::fs::metadata(&file).is_ok_and(|m| m.len() > 0) {
            return Some(file);
        }
        return None;
    }
    if run("wl-paste", &["--list-types"]).is_some_and(|t| String::from_utf8_lossy(&t).lines().any(|l| l == "image/png")) {
        return run("wl-paste", &["--type", "image/png"]).and_then(saved);
    }
    if run("xclip", &["-selection", "clipboard", "-t", "TARGETS", "-o"]).is_some_and(|t| String::from_utf8_lossy(&t).lines().any(|l| l == "image/png")) {
        return run("xclip", &["-selection", "clipboard", "-t", "image/png", "-o"]).and_then(saved);
    }
    None
}
