//! The date selector of the rich editors: `//` puts the date of today; up and down change the
//! selected part, left and right go to the next part (TEXT-15, TEXT-16, TEXT-17).
use super::*;
use chrono::{Datelike, Months, NaiveDate};

/// A part of a date.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Part {
    Year,
    Month,
    Day,
}

/// The order of the parts and the char between them, from the locale of the system.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DateFormat {
    order: [Part; 3],
    sep: char,
}

impl DateFormat {
    /// The format of the locale of the system: `LC_ALL`, `LC_TIME`, then `LANG`.
    pub fn system() -> Self {
        let var = ["LC_ALL", "LC_TIME", "LANG"].into_iter().filter_map(|v| std::env::var(v).ok()).find(|v| !v.is_empty());
        Self::from_locale(&var.unwrap_or_default())
    }

    /// The format of a locale name such as `ru_RU.UTF-8`; ISO 8601 for `C` and the unknown ones.
    pub fn from_locale(locale: &str) -> Self {
        use Part::*;
        let name = locale.split(['.', '@']).next().unwrap_or("");
        let lang = name.split('_').next().unwrap_or("");
        let f = |order, sep| Self { order, sep };
        match (lang, name) {
            ("en", "en_US" | "en_PH" | "en") => f([Month, Day, Year], '/'),
            ("en" | "fr" | "es" | "it" | "pt" | "el" | "ga" | "vi", _) => f([Day, Month, Year], '/'),
            ("ru" | "de" | "uk" | "be" | "pl" | "cs" | "sk" | "fi" | "nb" | "nn" | "no" | "da" | "tr" | "ro" | "bg" | "sr" | "hr" | "sl" | "et" | "lv" | "kk" | "az" | "is", _) => f([Day, Month, Year], '.'),
            ("nl", _) => f([Day, Month, Year], '-'),
            ("ja" | "zh", _) => f([Year, Month, Day], '/'),
            ("hu" | "ko", _) => f([Year, Month, Day], '.'),
            _ => f([Year, Month, Day], '-'),
        }
    }

    pub fn format(&self, d: NaiveDate) -> String {
        let part = |p| match p {
            Part::Year => format!("{:04}", d.year()),
            Part::Month => format!("{:02}", d.month()),
            Part::Day => format!("{:02}", d.day()),
        };
        self.order.map(part).join(&self.sep.to_string())
    }

    /// The char range of the part `i` in the text of a date.
    fn range(&self, i: usize) -> (usize, usize) {
        let width = |p: &Part| if *p == Part::Year { 4 } else { 2 };
        let start: usize = self.order[..i].iter().map(|p| width(p) + 1).sum();
        (start, start + width(&self.order[i]))
    }
}

/// The date selector in the text: where the date is, the date, and the selected part.
#[derive(Clone, Debug)]
pub struct DatePick {
    row: usize,
    col: usize,
    date: NaiveDate,
    part: usize,
    fmt: DateFormat,
}

/// A key in the date selector.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DateKey {
    Up,
    Down,
    Left,
    Right,
    Enter,
    Other,
}

/// The length of a date in chars.
const LEN: usize = 10;

impl Editor {
    /// `//` before the cursor, at the start of a word, outside code: the date of `today` takes its
    /// place, and the selector starts on its first part. False when the slashes stay text.
    pub fn slashes_to_date(&mut self, today: NaiveDate, fmt: DateFormat) -> bool {
        let chars: Vec<char> = self.lines[self.row].chars().take(self.col).collect();
        let n = chars.len();
        let word_start = n == 2 || n > 2 && chars[n - 3].is_whitespace();
        let in_code = self.in_fence() || chars.iter().filter(|c| **c == '`').count() % 2 == 1;
        if !(n >= 2 && chars[n - 2..] == ['/', '/'] && word_start) || in_code {
            return false;
        }
        self.backspace();
        self.backspace();
        let col = self.col;
        self.insert_str(&fmt.format(today));
        self.date = Some(DatePick { row: self.row, col, date: today, part: 0, fmt });
        self.select_part();
        true
    }

    /// A key in the date selector. True when the selector took it; false when the selector ended
    /// and the key goes on to the editor.
    pub fn date_key(&mut self, key: DateKey) -> bool {
        let Some(mut p) = self.date.take().filter(|p| self.date_is_there(p)) else { return false };
        match key {
            DateKey::Up | DateKey::Down => {
                p.date = step(p.date, p.fmt.order[p.part], key == DateKey::Up);
                let (b1, b2) = (self.byte(p.row, p.col), self.byte(p.row, p.col + LEN));
                self.lines[p.row].replace_range(b1..b2, &p.fmt.format(p.date));
                self.touch();
            }
            DateKey::Left if p.part > 0 => p.part -= 1,
            DateKey::Right if p.part < 2 => p.part += 1,
            DateKey::Left => return self.leave_date(p.col, true),
            _ => return self.leave_date(p.col + LEN, key != DateKey::Other),
        }
        self.date = Some(p);
        self.select_part();
        true
    }

    /// The date of the selector is still in the text where it was put.
    fn date_is_there(&self, p: &DatePick) -> bool {
        let line: String = self.lines.get(p.row).map(|l| l.chars().skip(p.col).take(LEN).collect()).unwrap_or_default();
        line == p.fmt.format(p.date)
    }

    fn leave_date(&mut self, col: usize, took: bool) -> bool {
        self.anchor = None;
        self.col = col;
        took
    }

    /// The selected part shows as the selection; the cursor is at its end.
    fn select_part(&mut self) {
        let Some(p) = &self.date else { return };
        let (a, b) = p.fmt.range(p.part);
        self.anchor = Some((p.row, p.col + a));
        (self.row, self.col) = (p.row, p.col + b);
    }
}

/// The date a step later or earlier in one part: a day, a month or a year.
fn step(d: NaiveDate, part: Part, up: bool) -> NaiveDate {
    let months = Months::new(if part == Part::Year { 12 } else { 1 });
    let next = match (part, up) {
        (Part::Day, true) => d.succ_opt(),
        (Part::Day, false) => d.pred_opt(),
        (_, true) => d.checked_add_months(months),
        (_, false) => d.checked_sub_months(months),
    };
    next.unwrap_or(d)
}
