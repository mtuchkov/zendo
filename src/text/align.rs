//! The same char in two views of a text: the source with its markup, and the text as read mode
//! shows it. A greedy match: markup in the source and decoration in the view are skipped.

/// How far the match looks ahead in the source for the next shown char.
const LOOK: usize = 12;

/// For each char of `shown`, the index of the same char in `source`, or `None` for a decoration
/// (a border, a bullet, a pad space).
pub fn align(shown: &[char], source: &[char]) -> Vec<Option<usize>> {
    let mut j = 0;
    (0..shown.len())
        .map(|i| {
            let c = shown[i];
            // Near: the next chars of the source. Far: the next few shown chars, anywhere after, to
            // find the way again after a long piece of markup such as the URL of a link.
            // A space matches only where the source has one: it never jumps over a word.
            let look = if c.is_whitespace() { 1 } else { LOOK };
            let near = (j..source.len().min(j + look)).find(|k| source[*k] == c);
            let word = &shown[i..shown.len().min(i + 4)];
            let far = || (word.len() == 4 && !c.is_whitespace()).then(|| source[j.min(source.len())..].windows(4).position(|w| w == word).map(|k| j + k)).flatten();
            let hit = near.or_else(far);
            if let Some(k) = hit {
                j = k + 1;
            }
            hit
        })
        .collect()
}

/// The source index of the shown char `i`: its own, or the next one that has one.
pub fn to_source(map: &[Option<usize>], i: usize, source_len: usize) -> usize {
    map.iter().skip(i).find_map(|m| *m).or_else(|| map.iter().take(i).rev().find_map(|m| m.map(|k| k + 1))).unwrap_or(0).min(source_len)
}

/// The shown index of the source char `j`: the shown char at it, or the first one after it.
pub fn to_shown(map: &[Option<usize>], j: usize) -> usize {
    map.iter().position(|m| m.is_some_and(|k| k >= j)).unwrap_or(map.len())
}

/// The chars of some lines joined with a space, and where each line starts.
pub fn joined<S: AsRef<str>>(lines: &[S]) -> (Vec<char>, Vec<usize>) {
    let mut chars = Vec::new();
    let mut starts = Vec::new();
    for l in lines {
        if !starts.is_empty() {
            chars.push(' ');
        }
        starts.push(chars.len());
        // A wrapped row can end in its break space; the join puts one space back.
        chars.extend(l.as_ref().trim_end().chars());
    }
    (chars, starts)
}

/// The line and the char of an index into joined lines.
pub fn split(starts: &[usize], i: usize) -> (usize, usize) {
    let k = starts.iter().rposition(|s| *s <= i).unwrap_or(0);
    (k, i - starts[k])
}

#[cfg(test)]
#[path = "align_tests.rs"]
mod tests;
