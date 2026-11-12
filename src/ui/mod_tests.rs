use super::*;

/// Hide panes one by one: `fits` allows `n` panes.
fn hide(p: &[Focus], focus: Focus, n: usize) -> Vec<Focus> {
    hide_to_fit(p.to_vec(), focus, |ps| ps.len() <= n)
}

#[test]
fn panes_hide_from_the_right_then_the_left_by_turns() {
    use Focus::*;
    let all = [Tree, Note, Claude, Delta];
    assert_eq!(hide(&all, Note, 3), vec![Tree, Note, Claude], "rightmost first");
    assert_eq!(hide(&all, Note, 2), vec![Note, Claude], "then leftmost");
    assert_eq!(hide(&all, Note, 1), vec![Note], "then rightmost again");
}

#[test]
fn the_pane_in_focus_stays() {
    use Focus::*;
    let all = [Tree, Note, Claude, Delta];
    assert_eq!(hide(&all, Delta, 3), vec![Tree, Note, Delta]);
    assert_eq!(hide(&all, Delta, 2), vec![Note, Delta]);
    assert_eq!(hide(&all, Delta, 1), vec![Delta]);
    assert_eq!(hide(&all, Tree, 2), vec![Tree, Note]);
}

#[test]
fn standard_layout_with_delta_in_place_of_the_note() {
    use Focus::*;
    assert_eq!(hide(&[Tree, Delta, Claude], Delta, 2), vec![Tree, Delta]);
    assert_eq!(hide(&[Tree, Delta, Claude], Delta, 1), vec![Delta]);
}

// CLAUDE-24: the messages of the user and of the agent look different: the user's on a block of
// their own color, the agent's on the panel; inline code stands out from the panel.
#[test]
fn user_and_agent_messages_look_different() {
    let t = THEMES[0];
    let user = user_lines("say hi", &t, 30);
    assert!(user.iter().all(|l| l.style.bg == Some(t.sel) && l.width() == 30), "{user:?}");
    let agent = agent_lines("see `README.md` now", &t, 30);
    assert!(agent.iter().all(|l| l.style.bg != Some(t.sel)));
    let code = agent.iter().flat_map(|l| l.spans.iter()).find(|s| s.content.contains("README")).expect("the code span");
    assert_ne!(code.style.bg, Some(t.panel), "inline code on the panel color would not show");
}
