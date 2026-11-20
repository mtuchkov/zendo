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

// CLAUDE-28, ADR 0027: the commands of zendo show with the commands of the agent, and win over a
// command of the agent with the same name.
#[test]
fn the_commands_of_zendo_come_with_the_ones_of_the_agent() {
    let list = with_own(vec![cmd("review"), cmd("new")]);
    let names: Vec<&str> = list.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names, vec!["new", "review"]);
    assert_ne!(list[0].description, "about new", "the `/new` of zendo, not the one of the agent");
    assert_eq!(with_own(Vec::new()).len(), 1, "`/new` shows before the agent sends its list");
}
