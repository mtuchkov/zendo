//! The result of the last command of a terminal (docs/zendo/terminal-output.md): the OSC 133 marks
//! of the shell, or the time from `enter` to a free terminal when the shell sends no marks.

/// The lines of the output that a result keeps, from the end (TERM-32).
pub const KEEP: usize = 500;

/// The result of a command (TERM-23, TERM-33).
#[derive(Clone, Debug, PartialEq)]
pub struct Output {
    pub command: String,
    /// `None`: the shell sends no marks, so the exit code is unknown (TERM-27).
    pub exit: Option<i32>,
    /// The output without ANSI codes: at most the last `KEEP` lines.
    pub text: String,
    /// The lines that the result dropped from the start (TERM-32).
    pub dropped: usize,
    /// False while the command runs (TERM-30).
    pub done: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum Phase {
    #[default]
    Idle,
    /// After the prompt: the shell echoes the command.
    Command,
    /// The command runs and prints.
    Output,
}

/// Reads the bytes of a shell, and keeps the result of its last command.
#[derive(Default)]
pub struct Recorder {
    /// The shell sent an OSC 133 mark (TERM-26).
    marks: bool,
    phase: Phase,
    command: Vec<u8>,
    out: Vec<u8>,
    /// The start of a mark that the next bytes complete.
    carry: Vec<u8>,
    /// The command line at the last `enter`, and its time.
    entered: Option<(String, std::time::Instant)>,
    last: Option<Output>,
}

const MARK: &[u8] = b"\x1b]133;";

impl Recorder {
    /// The bytes of the shell, in the order that it prints them.
    pub fn feed(&mut self, bytes: &[u8]) {
        let mut data = std::mem::take(&mut self.carry);
        data.extend_from_slice(bytes);
        let mut rest = &data[..];
        while let Some(at) = find(rest, MARK) {
            self.sink(&rest[..at]);
            let body = &rest[at + MARK.len()..];
            let Some((end, len)) = terminator(body) else {
                self.carry = rest[at..].to_vec();
                return;
            };
            self.mark(&body[..end]);
            rest = &body[end + len..];
        }
        let keep = partial_mark(rest);
        self.sink(&rest[..rest.len() - keep]);
        self.carry = rest[rest.len() - keep..].to_vec();
    }

    /// The user or zendo pressed `enter` in the shell; `line` is the command line. Without marks,
    /// the command starts here (TERM-27).
    pub fn enter(&mut self, line: &str) {
        self.entered = Some((line.trim().to_string(), std::time::Instant::now()));
        if !self.marks {
            (self.phase, self.command, self.out) = (Phase::Output, line.trim().as_bytes().to_vec(), Vec::new());
        }
    }

    /// The terminal is free again: without marks, the command ended (TERM-27). An empty command
    /// line keeps the last result.
    pub fn free(&mut self) {
        if !self.waits() {
            return;
        }
        let empty = plain(&self.command).trim().ends_with(['$', '#', '>', '%']);
        if empty && plain(&self.out).lines().count() <= 2 {
            (self.phase, self.out) = (Phase::Idle, Vec::new());
            return;
        }
        self.finish(None);
    }

    /// A command started without marks and did not end yet.
    pub fn waits(&self) -> bool {
        !self.marks && self.phase == Phase::Output
    }

    /// The time of the `enter` of a command without marks that did not end yet.
    pub fn waits_since(&self) -> Option<std::time::Instant> {
        self.entered.as_ref().map(|(_, t)| *t).filter(|_| self.waits())
    }

    /// The result: the command that runs, or else the last one that ended (TERM-23, TERM-30).
    pub fn result(&self) -> Option<Output> {
        if self.phase == Phase::Output {
            return Some(self.output(None, false));
        }
        self.last.clone()
    }

    fn sink(&mut self, bytes: &[u8]) {
        match self.phase {
            Phase::Command => self.command.extend_from_slice(bytes),
            Phase::Output => self.out.extend_from_slice(bytes),
            Phase::Idle => {}
        }
    }

    /// `A` the prompt, `B` the command, `C` the output, `D;code` the end (TERM-26).
    fn mark(&mut self, body: &[u8]) {
        self.marks = true;
        let mut parts = body.split(|b| *b == b';');
        match (parts.next().unwrap_or_default(), self.phase) {
            (b"B", _) => (self.phase, self.command) = (Phase::Command, Vec::new()),
            (b"C", Phase::Output) => {}
            (b"C", _) => (self.phase, self.out) = (Phase::Output, Vec::new()),
            (b"D", Phase::Output) => {
                let code = parts.next().and_then(|c| std::str::from_utf8(c).ok()).and_then(|c| c.trim().parse().ok());
                self.finish(Some(code.unwrap_or(0)));
            }
            (b"A" | b"D", _) => self.phase = Phase::Idle,
            _ => {}
        }
    }

    fn finish(&mut self, exit: Option<i32>) {
        self.last = Some(self.output(exit, true));
        (self.phase, self.out) = (Phase::Idle, Vec::new());
    }

    fn output(&self, exit: Option<i32>, done: bool) -> Output {
        let typed = plain(&self.command).trim().to_string();
        // Without the B mark, the command line at `enter`.
        let command = if typed.is_empty() { self.entered.as_ref().map(|(l, _)| l.clone()).unwrap_or_default() } else { typed };
        let mut lines: Vec<String> = plain(&self.out).lines().map(str::to_string).collect();
        // Without marks the output starts with the echo of the command and ends with the prompt.
        if !self.marks {
            let short = command.rsplit("$ ").next().unwrap_or(&command).trim().to_string();
            if lines.first().is_some_and(|l| l.trim().is_empty() || (!short.is_empty() && l.trim_end().ends_with(&short))) {
                lines.remove(0);
            }
            if done {
                lines.pop();
            }
        }
        while lines.last().is_some_and(|l| l.trim().is_empty()) {
            lines.pop();
        }
        let dropped = lines.len().saturating_sub(KEEP);
        Output { command, exit, text: lines[dropped..].join("\n"), dropped, done }
    }
}

fn find(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).position(|w| w == needle)
}

/// The end of a mark: BEL, or ESC `\`; its position and length.
fn terminator(body: &[u8]) -> Option<(usize, usize)> {
    let bel = body.iter().position(|b| *b == 0x07).map(|i| (i, 1));
    let st = find(body, b"\x1b\\").map(|i| (i, 2));
    [bel, st].into_iter().flatten().min()
}

/// The length of the end of `bytes` that can be the start of a mark.
fn partial_mark(bytes: &[u8]) -> usize {
    (1..MARK.len()).rev().find(|n| bytes.len() >= *n && bytes.ends_with(&MARK[..*n])).unwrap_or(0)
}

/// The text of terminal bytes without ANSI codes: `\r` starts the line again, backspace deletes
/// (TERM-33).
pub fn plain(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    let mut lines = vec![String::new()];
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        let line = lines.last_mut().expect("a line");
        match c {
            '\x1b' => skip_escape(&mut chars),
            '\n' => lines.push(String::new()),
            '\r' if chars.peek() == Some(&'\n') => {}
            '\r' => line.clear(),
            '\x08' => {
                line.pop();
            }
            '\t' => line.push(c),
            c if c.is_control() => {}
            c => line.push(c),
        }
    }
    lines.join("\n")
}

/// Skip the rest of an escape sequence: CSI, OSC and the other ones.
fn skip_escape(chars: &mut std::iter::Peekable<std::str::Chars>) {
    match chars.next() {
        Some('[') => {
            for c in chars.by_ref() {
                if ('\x40'..='\x7e').contains(&c) {
                    break;
                }
            }
        }
        Some(']' | 'P' | '_' | '^') => {
            while let Some(c) = chars.next() {
                if c == '\x07' || (c == '\x1b' && chars.next_if_eq(&'\\').is_some()) {
                    break;
                }
            }
        }
        Some('(' | ')' | '*' | '+') => {
            chars.next();
        }
        _ => {}
    }
}

#[cfg(test)]
#[path = "output_tests.rs"]
mod tests;
