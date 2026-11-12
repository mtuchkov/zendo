use super::*;

#[test]
fn title_and_front_matter_go_in_only_when_missing() {
    let answer = "TITLE: Retention of audit logs\nFRONT MATTER:\nstatus: draft\ndate: 2026-11-12\nlevel: the retention rules\nEND";
    let (t, f) = parse_meta(answer);
    assert_eq!(t.as_deref(), Some("Retention of audit logs"));
    assert_eq!(f.as_deref(), Some("status: draft\ndate: 2026-11-12\nlevel: the retention rules"));
    let both = Meta { title: true, front: true };
    assert_eq!(with_meta("Keep logs for 90 days.\n", both, t.as_deref(), f.as_deref()).unwrap(), "---\nstatus: draft\ndate: 2026-11-12\nlevel: the retention rules\n---\n\n# Retention of audit logs\n\nKeep logs for 90 days.\n");
    let fm = "---\nstatus: approved\n---\nText\n";
    assert_eq!(with_meta(fm, both, t.as_deref(), f.as_deref()).unwrap(), "---\nstatus: approved\n---\n# Retention of audit logs\n\nText\n", "the old front matter stays");
    assert_eq!(with_meta("# Has one\n", both, t.as_deref(), None), None, "nothing to add");
    assert_eq!(parse_meta("I cannot.").0, None);
    let plain = Meta { title: true, front: false };
    assert_eq!(with_meta("Rate limits\n\nThe API shall allow 100.\n", plain, Some("Rate limits"), None).unwrap(), "# Rate limits\n\nThe API shall allow 100.\n", "the first line becomes the title");
}
