use super::*;

fn cmd(name: &str) -> Command {
    Command { name: name.into(), description: format!("about {name}"), hint: None }
}

// CLAUDE-28: the commands that match what follows `/`.
#[test]
fn a_name_that_starts_with_the_query_comes_first() {
    let all = [cmd("review"), cmd("init"), cmd("pr-review"), cmd("compact")];
    let names: Vec<String> = matches(&all, "rev").into_iter().map(|c| c.name).collect();
    assert_eq!(names, vec!["review", "pr-review"]);
    assert_eq!(matches(&all, "").len(), 4, "`/` alone shows every command");
    assert!(matches(&all, "zz").is_empty());
}
