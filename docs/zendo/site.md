---
status: "draft"
revision: "1"
date: "2026-11-13"
parent: "zendo.md"
prev: "commands.md"
next: "lofi.md"
level: "what a visitor of the landing page sees, hears and does, and the decisions of the user that shaped it"
note: "Agent draft from the requests of the user on 2026-11-13, after site/index.html was built. Not reviewed by the user. The requirement IDs are proposals until the user reviews this document."
---

# Landing page

Sources: the requests of the user on 2026-11-13, with the hero of omarchy.org as the model, and
`site/index.html` in zendo 0.49.1.

## What the visitor sees

The page is a quiet room. A field of square pixels moves slowly, in the spirit of lofi music. The
wordmark ZENDO stands in the middle in pixel letters of sumi ink, dark at the top and pale at the
bottom, with a vermilion block caret like a seal. A raked gravel garden with stones and moss lies at
the bottom. From time to time a glyph forms from loose pixels, stays, and falls apart into the garden.

The page behaves like zendo: terminal buttons, a status bar with the mode and the keys, a note, a
spec and the agent chat. When the visitor does nothing, the page shows how `a` works.

## Main design choices

- The colors are traditional Japanese colors, not the colors of the zendo UI: washi paper, sumi ink,
  moss, indigo, and vermilion.
- The page is one HTML file with no build step. It loads only fonts from Google Fonts.
- The music is made in the browser, so no one owns it. A CC0 track in `site/lofi.mp3` replaces it. See [Lofi music](lofi.md).
- The chat on the page is a demo. No agent answers, and the page says so when the visitor asks.

## Architecturally significant requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| SITE-1  | Ubiquitous | The landing page shall be one self-contained HTML file in `site/` that needs no build step. | User, 2026-11-13 |
| SITE-2  | Ubiquitous | The page shall use one pixel size for all of the field: the dust, the garden, the stones and the glyphs. | User, 2026-11-13 |
| SITE-3  | Ubiquitous | The page shall choose the largest pixel size at which the widest glyph fits beside the wordmark. | User, 2026-11-13 |
| SITE-4  | Ubiquitous | The field shall move slowly and in steps, with few changes per second. | User, 2026-11-13 |
| SITE-5  | Ubiquitous | The page shall play only music that no one owns: music made in the browser, or a CC0 track. | User, 2026-11-13 |
| SITE-6  | Unwanted   | If the visitor asks for less motion, then the page shall show one still frame. | Design |

### Acceptance tests

**`site renders`** (browser check; verifies SITE-1, SITE-2, SITE-3)

- Given `site/index.html` opened from disk in headless Chrome at 1280×900 and at 1680×1000.
- When ten seconds pass.
- Then the page shows the wordmark, the garden and at least one glyph, and the console shows no
  script error.

**`site music toggles`** (browser check; verifies SITE-5, MUSIC-1, MUSIC-2)

- Given the page in headless Chrome.
- When the visitor clicks the page, then clicks `[ m lofi ]`, then presses `m`.
- Then the button shows on, off, and on.

## Detailed requirements

| ID       | Pattern    | Requirement | Source |
| -------- | ---------- | ----------- | ------ |
| FIELD-1  | Ubiquitous | The dust pixels shall be dense at the sides and sparse in the middle. | User, 2026-11-13 |
| FIELD-2  | Ubiquitous | The page shall show a small pixel arrow in the old style as the pointer. | User, 2026-11-13 |
| FIELD-3  | Event      | When the pointer moves, the field shall leave a short trail of moss pixels and light the dust near the pointer. | User, 2026-11-13 |
| FIELD-4  | Ubiquitous | The garden surface shall curve up at the sides, and the rake lines shall follow it. | User, 2026-11-13 |
| FIELD-5  | Unwanted   | If a ring appears in the gravel, then a visible falling pixel or a click of the visitor shall have caused it. | User, 2026-11-13 |
| GLYPH-1  | Ubiquitous | The page shall show only these glyphs: a file, a terminal, the ask bubble, a git branch, and 禅 in vermilion without a frame. | User, 2026-11-13 |
| GLYPH-2  | Ubiquitous | Glyphs shall appear from time to time, at most two at once, and cover no text. | User, 2026-11-13 |
| GLYPH-3  | Event      | When the visitor clicks the field, a glyph shall form at the pointer. | Design |
| UI-1     | Ubiquitous | Each button shall look like a terminal button: brackets, the key in reverse video, and reverse video on hover. | User, 2026-11-13 |
| UI-2     | Ubiquitous | The page shall show no version banner and no product name in the top left corner. | User, 2026-11-13 |
| UI-3     | Event      | When the visitor presses `i` or an install button, the page shall show the install command with a copy button. | Design |
| UI-4     | Event      | When the visitor presses `n`, the page shall open a new note, type a short funny note, and let the agent add a title. | User, 2026-11-13 |
| UI-5     | Event      | When the visitor presses `s`, the page shall open the spec "zendo ergonomics", written in EARS. | User, 2026-11-13 |
| UI-6     | Event      | When the visitor presses `a`, the page shall open the agent chat beside the content in a wide window and below it in a narrow window. | User, 2026-11-13 |
| UI-7     | Event      | When the visitor presses `a` with selected text, the chat shall quote the selection. | Design |
| UI-8     | Event      | When the visitor presses `esc`, the page shall close the chat, and at the next `esc` the document. | Design |
| DEMO-1   | State      | While the visitor does nothing, the page shall from time to time select "See it in action", press `a`, ask "how?", and let the agent answer with the install command. | User, 2026-11-13 |
| DEMO-2   | Event      | When the visitor clicks "See it in action", the page shall run the demo at once. | Design |
| DEMO-3   | Unwanted   | If the chat of the demo is open, then a move of the mouse shall not close it or stop the answer. | User, 2026-11-13 |
| DEMO-4   | Event      | When the visitor started the demo, the chat shall stay open until the visitor closes it. | User, 2026-11-13 |
| MUSIC-1  | Ubiquitous | The music shall be on by default, and it shall start at the first click, touch or key that the browser allows. | User, 2026-11-13 |
| MUSIC-2  | Event      | When the visitor presses `m` or the music button, the page shall stop the music if it plays, and start it otherwise. | User, 2026-11-13 |
| MUSIC-3  | Ubiquitous | The music shall play without breaks until the visitor stops it, also in a background tab. | User, 2026-11-13 |
| MUSIC-4  | State      | While the music plays, the caret shall blink on the beat and a drop shall fall beside a stone on each bar. | User, 2026-11-13 |

## Open points

| Open question | Affects | Answered by |
| ------------- | ------- | ----------- |
| Which host serves `site/`: GitHub Pages from a workflow, or another host? | SITE-1 | User |
| Does the page get a CC0 track in `site/lofi.mp3`, and which one? | SITE-5 | User |
| Should a hint tell the visitor that a click starts the music? | MUSIC-1 | User |
| No test in `cargo test` covers the page. Is a browser check in CI enough? | SITE-1 to SITE-6 | User |
