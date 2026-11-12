use super::*;

fn chars(s: &str) -> Vec<char> {
    s.chars().collect()
}

// NOTE-28: the cursor stays on the same char between read and write mode.
#[test]
fn markup_and_decoration_are_skipped() {
    let src = chars("- an **owner** of `x`");
    let shown = chars("• an owner of  x ");
    let map = align(&shown, &src);
    let i = shown.iter().position(|c| *c == 'w').unwrap();
    assert_eq!(src[to_source(&map, i, src.len())], 'w');
    let j = src.iter().position(|c| *c == 'x').unwrap();
    assert_eq!(shown[to_shown(&map, j)], 'x');
    // The bullet maps to the first word after it.
    assert_eq!(src[to_source(&map, 0, src.len())], 'a');
}

#[test]
fn a_table_row_maps_cell_by_cell() {
    let src = chars("| Warm | 31–180 days |");
    let shown = chars("│ Warm │ 31–180 days │");
    let map = align(&shown, &src);
    let i = shown.iter().position(|c| *c == '3').unwrap();
    assert_eq!(src[to_source(&map, i, src.len())], '3');
}
