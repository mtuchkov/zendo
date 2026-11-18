---
status: "approved"
approved: "2026-11-18"
revision: "3"
date: "2026-11-18"
parent: "zendo.md"
prev: "checklist.md"
next: "live-status.md"
level: "what the transcript of a thread shows around its messages: the time of each message, and the question of the turn in the view at the top of the pane"
note: "Agent draft from the request of the user on 2026-11-18 in chat: show the time of each message of the user and of Claude on the right, and keep the question of the user at the top of the transcript. Revision 2: the answers of the user on 2026-11-18 (no time for earlier sessions, the question of the turn in the view, 24-hour time). Revision 3: the checklist of the turn shows above the question at the top (User, 2026-11-18). Approved by the user on 2026-11-18."
---

# Transcript

The transcript is the list of the messages of a thread in the chat pane. Its keys are in
[Chat](chat.md); what the chat shows during a turn is in [Activity of the agent](activity.md). This
document adds two things: the time of each message, and the question of a turn at the top of the pane.

**Time of a message.** Each question of the user and each message of Claude shows its time at the
right edge of its first line, in the dim color of the theme, in 24-hour form, such as `23:41`. A question shows the
time when zendo sent it. A message of Claude shows the time of its first part. A message of an
earlier day also shows its date, such as `Nov 17 23:41`. The time takes no line of its own: when the
first line is long, the text wraps before the time. The steps of a turn show how long they took
(ACT-1), not a time. A message of a Claude Code session shows the time when zendo read it
([Claude Code messages in the chat](cli-messages.md)). When zendo does not know the time of a
message, for example of a message of an earlier session (CLAUDE-27), the message shows no time.
`y` copies the text without the time (CLAUDE-29).

**The question at the top.** The answer can push the question out of the view, and then the user
does not see what the agent answers. The turn in the view is the turn of the top line of the
transcript. While the question of that turn is above the top, because the answer grew or because
the user scrolled, the top of the pane shows the question: at most two lines, with `…` when it is
longer, its time on the right, and a rule under it. The transcript scrolls under it. When the user
scrolls into another turn, its question takes the place (User, 2026-11-18). When the question comes
into the view again, the copy at the top goes away. A click on the copy scrolls to the question.
While a turn with a plan runs, its checklist shows above the copy ([Checklist](checklist.md)).

## Architecturally significant requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| TRANS-1 | Ubiquitous | The system shall show the time of each question of the user and of each message of Claude at the right edge of its first line, in 24-hour form. | User, 2026-11-18 |
| TRANS-2 | State      | While the question of the turn at the top line of the transcript is above the top, the system shall show that question at the top of the transcript pane, in at most two lines, with its time. | User, 2026-11-18 |

### Acceptance tests

Tests: `tests/transcript.rs`, with the mock agent and a fixed clock.

**`each_message_shows_its_time_on_the_right`** (mock agent; verifies TRANS-1)

- Given a question sent at 23:41, and an answer of the mock agent that starts at 23:42.
- When the screen draws, and the user copies both messages with `y`.
- Then the first line of the question ends with `23:41`, the first line of the answer ends with
  `23:42`, and the copied text has no time.

**`the_last_question_sticks_to_the_top`** (mock agent; verifies TRANS-2)

- Given a transcript pane of 30 lines, and two turns, each with an answer of 200 lines.
- When the second answer pushes its question out of the view, and later the user scrolls up into
  the first answer, and then up to the first question.
- Then the top line shows the second question, then the first question, and then no copy while
  the first question is in the view.

## Detailed requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| TRANS-3 | Ubiquitous | The system shall show for a question the time when it went to the agent, and for a message of Claude the time of its first part. | Design |
| TRANS-4 | Ubiquitous | The system shall show the date with the time of a message of an earlier day. | Design |
| TRANS-5 | Unwanted   | If the system does not know the time of a message, such as a message of an earlier session, then the system shall show no time for it, and shall not read it from the files of Claude Code. | User, 2026-11-18 |
| TRANS-6 | Ubiquitous | The system shall wrap the first line of a message before its time, and shall copy a message without its time. | Design |
| TRANS-7 | Event      | When the user clicks the question at the top of the transcript, the system shall scroll the transcript to that question. | Design |
| TRANS-8 | Event      | When the turn at the top line of the transcript changes, the system shall show the question of the new turn at the top, while TRANS-2 applies. | User, 2026-11-18 |

## Open points

None. The user answered the open points on 2026-11-18.
