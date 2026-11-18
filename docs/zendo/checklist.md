---
status: "draft"
revision: "1"
date: "2026-11-18"
parent: "zendo.md"
prev: "activity.md"
next: "transcript.md"
level: "the checklist of a turn: where it shows, closed and open, and where it goes when the turn ends"
note: "Agent draft from the requests of the user on 2026-11-18 in chat. Moved from activity.md (revision 4) to keep that document within two pages; the IDs stay. Not reviewed by the user. The requirement IDs are proposals until the user reviews this document."
---

# Checklist

The agent writes a plan of its work, a to-do list, and changes it while it works. zendo shows it as
the checklist of the turn, so that the user sees how far the agent is. The rest of the activity is
in [Activity of the agent](activity.md); a Claude Code session has the same checklist
([Activity of a Claude Code session](cli-activity.md)).

**Where it shows.** While the turn runs, the checklist shows at the top of the transcript pane,
above the question at the top ([Transcript](transcript.md)). It is closed: one line, such as
`plan 3/7`, the count of the done items and of all items. A click opens it with its items and their
states (to do, in progress, done), and `[x]` on its right closes it again (User, 2026-11-18).

**Changes.** A new version of the plan replaces the earlier one in place, open or closed as it was.

**The end of the turn.** When the turn ends, the checklist goes away from the top, and its last
version goes into the transcript once.

## Architecturally significant requirements

| ID     | Pattern    | Requirement | Source |
| ------ | ---------- | ----------- | ------ |
| ACT-3  | Event      | When the agent sends a new version of its plan, the system shall show the plan as a checklist with the state of each item, in place of the earlier version. | User, 2026-11-18 |

### Acceptance tests

Tests: `tests/activity.rs`, with the mock agent.

**`the_checklist_changes_in_place`** (mock agent; verifies ACT-3)

- Given a plan of three items from the mock agent.
- When the agent sends the plan again with the first item done, the user clicks the checklist, and
  then clicks `[x]`.
- Then the top of the transcript shows one line `plan 1/3`, then the three items with the first
  one done, and then the line `plan 1/3` again.

## Detailed requirements

| ID     | Pattern  | Requirement | Source |
| ------ | -------- | ----------- | ------ |
| ACT-10 | Event    | When the turn ends, the system shall remove the checklist from the top of the transcript pane and put its last version into the transcript once. | Design |
| ACT-14 | State    | While a turn with a plan runs, the system shall show the checklist at the top of the transcript pane, above the question of TRANS-2, as one line with the count of done items and of all items. | User, 2026-11-18 |
| ACT-15 | Event    | When the user clicks the closed checklist, the system shall open it with its items; when the user clicks `[x]` on the open checklist, the system shall close it to one line. | User, 2026-11-18 |
