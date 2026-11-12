//! Code blocks and front matter: their highlighted pieces.
use super::*;

/// The second word of the fence of a whole file: `` ```rs file ``.
pub const FILE_FENCE: &str = "file";

/// The highlighted pieces of each line of code: the fenced blocks, and the front matter as YAML.
pub(super) fn code_pieces(logicals: &[Logical]) -> HashMap<usize, Vec<(Kind, String)>> {
    let mut out = HashMap::new();
    let mut put = |lang: &str, idx: &[usize], file: bool| {
        let lines: Vec<String> = idx.iter().map(|i| logicals[*i].text.clone()).collect();
        let pieces = if file { crate::text::highlight::file_pieces(lang, &lines) } else { crate::text::highlight::pieces(lang, &lines) };
        if let Some(p) = pieces {
            for (i, row) in idx.iter().zip(p) {
                out.insert(*i, row);
            }
        }
    };
    let front: Vec<usize> = logicals.iter().enumerate().filter(|(_, l)| l.verbatim && l.text.trim() != "---").map(|(i, _)| i).collect();
    if !front.is_empty() {
        put("yaml", &front, false);
    }
    let mut i = 0;
    while i < logicals.len() {
        let t = logicals[i].text.trim_start();
        if !logicals[i].verbatim && t.starts_with("```") {
            let mut info = t.trim_start_matches('`').split_whitespace();
            let lang = info.next().unwrap_or("").to_string();
            // `file`: the fence of a whole file in the note pane.
            let file = info.next() == Some(FILE_FENCE);
            let start = i + 1;
            let mut end = start;
            while end < logicals.len() && !logicals[end].text.trim_start().starts_with("```") {
                end += 1;
            }
            if !lang.is_empty() {
                put(&lang, &(start..end).collect::<Vec<_>>(), file);
            }
            i = end + 1;
        } else {
            i += 1;
        }
    }
    out
}
