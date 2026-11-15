use super::*;

fn taken(names: &[&str]) -> Vec<String> {
    names.iter().map(|s| s.to_string()).collect()
}

// NAMES-1, NAMES-2: the first free word of one to 12 letters, in lower case.
#[test]
fn pick_takes_the_first_free_word() {
    assert_eq!(pick("Parser, lexer, tokens", &[]).as_deref(), Some("parser"));
    assert_eq!(pick("parser lexer tokens", &taken(&["parser"])).as_deref(), Some("lexer"));
    assert_eq!(pick("Here are the names:\nhooks, x, layout", &[]).as_deref(), Some("hooks"), "the last line");
    assert_eq!(pick("averyveryverylongword short", &[]).as_deref(), Some("short"));
    assert_eq!(pick("mock", &taken(&["mock"])), None, "no free word");
    assert_eq!(pick("", &[]), None);
}

// NAMES-1: only turns with an answer count.
#[test]
fn answered_counts_the_turns_with_an_answer() {
    let items = vec![Item::Info("x".into()), Item::User("a".into()), Item::Agent("b".into()), Item::User("c".into())];
    assert_eq!(answered(&items), 1);
    let mut more = items.clone();
    more.push(Item::Agent("d".into()));
    assert_eq!(answered(&more), 2);
}
