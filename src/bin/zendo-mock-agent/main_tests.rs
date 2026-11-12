use super::propose;

#[test]
fn proposal_goes_after_last_bullet() {
    let out = propose("# T\n\n- a\n- b\n\nend\n");
    assert!(out.lines().nth(4).unwrap().starts_with("- WHEN"));
    assert!(out.ends_with("end\n"));
}
