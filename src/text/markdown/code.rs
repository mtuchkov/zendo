//! Code blocks and front matter: their highlighted pieces.
use super::*;

/// The highlighted pieces of each line of code: the fenced blocks, and the front matter as YAML.
pub(super) fn code_pieces(logicals: &[Logical]) -> HashMap<usize, Vec<(Kind, String)>> {
    let mut out = HashMap::new();
    let mut put = |lang: &str, idx: &[usize]| {
        let lines: Vec<String> = idx.iter().map(|i| logicals[*i].text.clone()).collect();
        if let Some(p) = crate::text::highlight::pieces(lang, &lines) {
            for (i, row) in idx.iter().zip(p) {
                out.insert(*i, row);
            }
        }
    };
    let front: Vec<usize> = logicals.iter().enumerate().filter(|(_, l)| l.verbatim && l.text.trim() != "---").map(|(i, _)| i).collect();
    if !front.is_empty() {
        put("yaml", &front);
    }
    let mut i = 0;
    while i < logicals.len() {
        let t = logicals[i].text.trim_start();
        if !logicals[i].verbatim && t.starts_with("```") {
            let lang = t.trim_start_matches('`').split_whitespace().next().unwrap_or("").to_string();
            let start = i + 1;
            let mut end = start;
            while end < logicals.len() && !logicals[end].text.trim_start().starts_with("```") {
                end += 1;
            }
            if !lang.is_empty() {
                put(&lang, &(start..end).collect::<Vec<_>>());
            }
            i = end + 1;
        } else {
            i += 1;
        }
    }
    out
}
