use super::*;

const PS: &str = "    1     0     1 S  3-01:02:03 /sbin/init
  100     1   100 S       10:00 zendo
  200   100   200 S       09:59 npx claude-agent-acp
  210   200   210 S       09:58 claude --output-format stream-json --verbose
  220   210   210 S       09:58 node mcp-server.js
  230   210   230 S       05:00 /bin/bash -c -l eval 'npm run dev' < /dev/null
  231   230   230 S       04:59 npm run dev
  240   210   240 S          12 sh -c sleep 60; :
  241   240   240 S          12 sleep 60
  250   210   250 S          30 /home/u/.nvm/versions/node/v24.15.0/bin/node /home/u/.local/lib/node_modules/codegraph/mcp.js
  251   250   250 S          30 /home/u/.local/lib/node_modules/codegraph/node_modules/.bin/indexer
  300   100   300 S       08:00 /bin/zsh
  310   300   310 S       07:00 cargo watch
  400     1   400 S       01:00 orphan-of-another-zendo
";

// BG-2: `ps` gives the pid, the parent, the group, the age and the command of each process.
#[test]
fn ps_output_parses() {
    let p = parse(PS);
    assert_eq!(p.len(), 14);
    assert_eq!((p[0].pid, p[0].ppid, p[0].pgid, p[0].secs), (1, 0, 1, 3 * 86400 + 3600 + 2 * 60 + 3));
    assert_eq!((p[6].secs, p[6].args.as_str()), (299, "npm run dev"));
    assert_eq!(p[7].secs, 12);
    assert_eq!(p[13].pid, 400);
    assert!(parse("garbage\n  1 2\n").is_empty());
    assert!(parse("  9 1 9 Z 0:01 [sleep] <defunct>\n").is_empty(), "a zombie does not count");
}

// BG-2, BG-5: a background process is a process group of its own under zendo; the agent, the MCP
// server in the group of the agent, and the subtree of a terminal do not count.
#[test]
fn background_groups_are_found_under_the_root() {
    let p = parse(PS);
    let g = groups(&p, 100, &[300]);
    let ids: Vec<u32> = g.iter().map(|g| g.pgid).collect();
    assert_eq!(ids, vec![230, 240], "{g:?}");
    assert_eq!(g[0].under, 200, "the direct child of zendo that it runs under");
}

// BG-1: a shell names the command that it runs; a plain process names itself.
#[test]
fn a_group_is_named_by_its_command() {
    let p = parse(PS);
    let g = groups(&p, 100, &[300]);
    assert_eq!(g[0].label, "npm run dev");
    assert_eq!(g[1].label, "sleep 60");
}

// BG-1: a shell without a child names the command of its `eval` or of its `-c`.
#[test]
fn a_shell_without_a_child_names_its_command() {
    let p = parse("100 1 100 S 1:00 zendo\n210 100 210 S 1:00 claude --output-format stream-json\n230 210 230 S 0:10 /bin/bash -c -l eval 'make watch' < /dev/null\n250 210 250 S 0:10 sh -c tail -f log\n");
    let g = groups(&p, 100, &[]);
    assert_eq!(g.iter().map(|g| g.label.as_str()).collect::<Vec<_>>(), vec!["make watch", "tail -f log"]);
}

// BG-5: the processes of an agent do not count, also in a group of their own; the background
// terminal of Claude Code names the command that it runs.
#[test]
fn the_agent_itself_does_not_count() {
    let p = parse("100 1 100 S 1:00 zendo\n200 100 200 S 1:00 node zendo-mock-agent\n210 200 210 S 1:00 codex-acp\n220 200 220 S 1:00 claude bg-pty-host\n221 220 221 S 0:59 vite\n");
    let g = groups(&p, 100, &[]);
    assert_eq!(g.iter().map(|g| (g.pgid, g.label.as_str())).collect::<Vec<_>>(), vec![(220, "vite")]);
}

// BG-10: a group stays alive while any process of it runs, also outside the tree.
#[test]
fn a_group_is_alive_while_a_process_of_it_runs() {
    let p = parse(PS);
    assert!(alive(&p, 230) && alive(&p, 400));
    assert!(!alive(&p, 999));
}

// BG-3, BG-9: zendo signals only the group of a background process: never group 0 (the group of
// the caller), group 1 (every process), or the group of zendo itself; and nothing while the group of
// zendo is unknown.
#[test]
fn zendo_never_signals_its_own_group() {
    assert!(may_stop(230, Some(100)));
    assert!(!may_stop(100, Some(100)), "the group of zendo");
    assert!(!may_stop(0, Some(100)) && !may_stop(1, Some(100)));
    assert!(!may_stop(230, None), "the group of zendo is unknown");
}


// BG-10: a group that runs is known to the kernel; a group that does not run is not.
#[test]
fn the_kernel_knows_a_running_group() {
    let own = own_group().expect("the group of the test");
    assert!(group_exists(own));
    assert!(!group_exists(0) && !group_exists(u32::MAX));
}
