use super::*;
use crate::ui::theme::THEMES;

fn text(l: &Line) -> String {
    l.spans.iter().map(|s| s.content.as_ref()).collect()
}

#[test]
fn wraps_at_width_with_hanging_indent() {
    let src = plain("- one two three four five six seven eight nine ten");
    let r = render(&src, 20, &THEMES[0], false);
    let lines: Vec<String> = r.lines.iter().map(text).collect();
    assert!(lines.len() > 1, "{lines:?}");
    assert!(lines[0].starts_with("• one"));
    assert!(lines[1].starts_with("  "));
    assert!(lines.iter().all(|l| l.trim_end().width() <= 20), "{lines:?}");
}

#[test]
fn title_is_skipped_and_code_is_padded() {
    let src = plain("# Title\n\ntext\n```\nlet x = 1;\n```");
    let r = render(&src, 30, &THEMES[0], true);
    let lines: Vec<String> = r.lines.iter().map(text).collect();
    assert!(!lines.iter().any(|l| l.contains("Title")));
    assert!(lines.iter().any(|l| l.contains("let x = 1;") && l.width() == 30), "{lines:?}");
}

#[test]
fn inline_links_and_code() {
    let spans = inline("see [[outbox-pattern]] and `order_log` WHEN done", Style::new(), &THEMES[0]);
    let s: String = spans.iter().map(|s| s.content.as_ref()).collect();
    assert_eq!(s, "see outbox-pattern and  order_log  WHEN done");
}

#[test]
fn bold_italic_underline_and_tables_render() {
    let t = crate::ui::theme::THEMES[0];
    let spans = inline("a *b* _c_ snake_case **d** ***e*** <u>f</u>", Style::new(), &t);
    let style_of = |w: &str| spans.iter().find(|s| s.content == w).map(|s| s.style.add_modifier).unwrap_or_else(|| panic!("{w}: {spans:?}"));
    assert!(style_of("b").contains(Modifier::ITALIC) && style_of("c").contains(Modifier::ITALIC));
    assert!(style_of("d").contains(Modifier::BOLD) && style_of("e").contains(Modifier::BOLD | Modifier::ITALIC));
    assert!(style_of("f").contains(Modifier::UNDERLINED));
    assert!(spans.iter().any(|s| s.content.contains("snake_case")), "a _ inside a word stays");
    let r = render(&plain("| a | b |\n|---|--:|\n| one | 2 |\n| two words here | 30 |"), 30, &t, false);
    let text: Vec<String> = r.lines.iter().map(|l| l.spans.iter().map(|s| s.content.as_ref()).collect()).collect();
    assert!(text[0].starts_with('┌') && text.last().unwrap().starts_with('└'), "{text:#?}");
    assert!(text.iter().all(|l| l.width() <= 30), "fits: {text:#?}");
    assert!(text.iter().any(|l| l.contains("│  2 │")), "right aligned: {text:#?}");
    let wide = render(&plain("| a | text |\n|---|---|\n| x | When a visit that counts ends, the system shall keep the text |"), 30, &t, false);
    let rows: Vec<String> = wide.lines.iter().map(|l| l.spans.iter().map(|s| s.content.as_ref()).collect()).collect();
    let w0 = rows[0].width();
    assert!(rows.iter().all(|l| l.width() == w0), "every row of a table is as wide as its border: {rows:#?}");
    assert_eq!(r.rows, vec![1, 2, 3, 4], "each source row knows its rendered row");
    // Inline code that wraps keeps the borders in place.
    for w in [30, 34, 40, 46, 52] {
        let code = render(&plain("| Run | How |\n|---|---|\n| one | LangGraph receives it as `configurable.thread_id`. For a Job, `jobs.thread_id` stores the Execution thread ID. |"), w, &t, false);
        let rows: Vec<String> = code.lines.iter().map(|l| l.spans.iter().map(|s| s.content.as_ref()).collect()).collect();
        let w0 = rows[0].width();
        assert!(rows.iter().all(|l| l.width() == w0 && l.ends_with(['┐', '┤', '│', '┘'])), "width {w}: {rows:#?}");
        assert!(rows.iter().all(|l| !l.contains("│  c") && !l.contains("│  j")), "no pad space at the start of a cell row: {rows:#?}");
    }
}

#[test]
fn hard_wrapped_paragraphs_and_items_are_rewrapped() {
    let src = plain("Most tools push you\nto the extremes. A doc\neditor leaves you alone.\n\n- one item that\n  goes on here\n- two\n\n> a quote\n> more\n\nhard break  \nnext line\n\n| a | b |\n| c | d |");
    let r = render(&src, 80, &THEMES[0], false);
    let lines: Vec<String> = r.lines.iter().map(text).collect();
    assert_eq!(lines[0], "Most tools push you to the extremes. A doc editor leaves you alone.");
    assert_eq!(lines[2], "• one item that goes on here");
    assert_eq!(lines[3], "• two");
    assert!(lines[5].ends_with("a quote more"), "{lines:?}");
    assert!(lines.iter().any(|l| l.trim_end() == "hard break"), "{lines:?}");
    assert!(lines.iter().any(|l| l == "next line"));
    assert!(lines.iter().any(|l| l.contains("| a | b |")) && lines.iter().any(|l| l.contains("| c | d |")));
    // Each source line still maps to a row: the row of its paragraph.
    assert_eq!(r.rows.len(), src.len());
    assert_eq!((r.rows[0], r.rows[1], r.rows[2]), (0, 0, 0));
}

#[test]
fn front_matter_and_code_stay_as_they_are() {
    let src = plain("---\ntitle: x\ndate: 2026\n---\n\n```\nlet a = 1;\nlet b = 2;\n```");
    let r = render(&src, 60, &THEMES[0], false);
    let lines: Vec<String> = r.lines.iter().map(text).collect();
    assert_eq!(&lines[..4], &["---", "title: x", "date: 2026", "---"]);
    assert!(lines.iter().any(|l| l.contains("let a = 1;")) && lines.iter().any(|l| l.contains("let b = 2;")));
}

#[test]
fn diff_marks_added_lines() {
    let d = diff_lines("a\nb\n", "a\nx\nb\n");
    assert_eq!(d, vec![("a".into(), Mark::None), ("x".into(), Mark::Added), ("b".into(), Mark::None)]);
}

// NOTE-32: each row of a joined paragraph knows where it starts in the source: line and char.
#[test]
fn rows_know_their_start_in_the_source() {
    let src = plain("aaa bbb\nccc **ddd**\neee\n\n- one two three four");
    let r = render_live(&src, 12, &THEMES[0], Some(1));
    let lines: Vec<String> = r.lines.iter().map(text).collect();
    assert_eq!(lines[0].trim_end(), "aaa bbb ccc", "the cursor line does not break the paragraph: {lines:?}");
    assert_eq!(r.starts[0], Some((0, 0)));
    assert_eq!(r.starts[1], Some((1, 4)), "the markup goes with its word: {lines:?}");
    let item = lines.iter().position(|l| l.starts_with("• one")).unwrap();
    assert_eq!(r.starts[item], Some((4, 0)));
    assert_eq!(r.starts[item + 1].map(|s| s.0), Some(4));
}
