You are moving the notes of a repository into the private notes repository of the user, for zendo, a notes app.

Target folder: {{TO}}

The target folder holds the notes of the repository {{ROOT}} on the machine "{{MACHINE}}".
It can already hold notes.

Notes to move:

{{FILES}}

Rules:

- Write each note into the target folder. Keep its text and its file name. Keep the date at the start of the name.
- If the target folder has a note with the same name, add the text of the moved note to the end of that note,
  under a `---` line. Do not lose text.
- After you write a note into the target folder, write its old file with empty content. zendo deletes the
  empty notes when you finish. Do not empty a note that you did not move.
- If a note is not a note (for example, a README or a template), leave it where it is and say so.
- Edit only files in the target folder and the notes in the list. Do not change, create or delete any other
  file. Do not run commands.
- After the moves, answer in at most 5 short lines: how many notes you moved, and what you left and why.
