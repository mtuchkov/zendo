---
status: "draft"
revision: "1"
date: "2026-11-18"
parent: "zendo.md"
prev: "fonts.md"
level: "how the user finds the recommended fonts, and how the agent sets the font of the terminal"
note: "Agent draft of the decisions of the user on 2026-11-18, split from fonts.md revision 2. Not reviewed by the user. The requirement IDs are proposals until the user reviews this document."
---

# Font tip

Sources: the requests and answers of the user on 2026-11-18; the theme picker in zendo 0.92.0. The
fonts and the command `zendo setup fonts` are in [Fonts](fonts.md).

## What the user sees and does

**In the theme picker.** `t` in read mode opens the theme picker. While a recommended font is not
on the machine, the picker shows under the themes: run `zendo setup fonts` to get the recommended
fonts, or `[ ask the agent · a ]`. While all of them are on the machine, it shows
`[ change the font · a ]`. Without a connected agent, it shows only the command.

**The question.** On `a` or a click, a float asks whether zendo may ask the agent. Only on yes,
zendo sends the request to the agent of the current session. The agent runs `zendo setup fonts`
when a font is missing, asks the user which font to use, and sets it in the terminal only after the
user agrees.

**The tip.** The end of `zendo setup fonts` tells how to open the theme picker and that the font
button there changes the font of the terminal with the agent. The key comes from the keymap of the
user.

**`zendo setup` and the installer.** The installer runs `zendo setup`. When the list of recommended
fonts changed since the last time that `zendo setup` named it, setup names `zendo setup fonts` and
the tip at its end. Otherwise it says nothing about fonts. The first run of setup names them.

## Architecturally significant requirements

| ID     | Pattern    | Requirement | Source |
| ------ | ---------- | ----------- | ------ |
| FONT-5 | Event      | When the user asks for the fonts in the theme picker, the system shall send the request to the agent only after the user answers yes. | User, 2026-11-18 |
| FONT-6 | Ubiquitous | The system shall not change the config of the terminal; the request to the agent shall tell the agent to change it only after the user agrees. | User, 2026-11-18 |

### Acceptance tests

**`the agent gets the font request after yes`** (mock agent; verifies FONT-5, FONT-6, FONT-14,
FONT-15)

- Given the theme picker with the tip and a connected mock agent.
- When the user presses `a`, then `n`, then `a`, then `y`.
- Then the agent gets no request after `n`, and one request after `y`, which tells it to run
  `zendo setup fonts` and to ask the user before it changes the terminal.

**`the theme picker shows the font tip`** (unit test; verifies FONT-13, FONT-16, FONT-21)

- Given a machine without one of the recommended fonts, and no connected agent.
- When the user opens the theme picker.
- Then zendo shows the tip with `zendo setup fonts` and no button for the agent; with an agent and
  all fonts on the machine, it shows `[ change the font · a ]`.

**`setup names the fonts when the list changed`** (unit test; verifies FONT-18, FONT-19, FONT-20)

- Given a config folder in the test folder, without a list of fonts that setup named.
- When `zendo setup` runs two times, and then once with one more recommended font.
- Then the first and the third output name `zendo setup fonts` and the key of the theme picker; the
  second output says nothing about fonts.

## Detailed requirements

| ID      | Pattern  | Requirement | Source |
| ------- | -------- | ----------- | ------ |
| FONT-13 | State    | While a recommended font is not on the machine, the theme picker shall show a tip with the command `zendo setup fonts` and the button `[ ask the agent · a ]`. | User, 2026-11-18 |
| FONT-14 | Event    | When the user presses `a` or clicks the font button of the theme picker, the system shall show a float that asks whether zendo may ask the agent. | User, 2026-11-18 |
| FONT-15 | Event    | When the user answers yes, the system shall send the agent of the current session a request to run `zendo setup fonts` if a font is missing, and then to help the user set the font of the terminal. | User, 2026-11-18 |
| FONT-16 | Unwanted | If no agent is connected, then the theme picker shall show the command without a button for the agent. | Design |
| FONT-18 | Event    | When `zendo setup` ends and the list of recommended fonts differs from the list that `zendo setup` named the last time, the system shall name `zendo setup fonts` and the tip of FONT-20, and keep the list. | User, 2026-11-18 |
| FONT-19 | Unwanted | If the list of recommended fonts is the same as the list that `zendo setup` named the last time, then `zendo setup` shall say nothing about fonts. | User, 2026-11-18 |
| FONT-20 | Ubiquitous | The tip shall tell the key that opens the theme picker, from the keymap of the user, and that the font button there changes the font of the terminal with the agent. | User, 2026-11-18 |
| FONT-21 | State    | While all recommended fonts are on the machine and an agent is connected, the theme picker shall show the button `[ change the font · a ]`. | Design |

## Open points

| Open question | Affects | Answered by |
| ------------- | ------- | ----------- |
| Where zendo keeps the list that `zendo setup` named the last time. | FONT-18 | The developer, in the code |
