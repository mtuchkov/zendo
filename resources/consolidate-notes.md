You are consolidating the notes of one project in the private notes repository of the user, for zendo, a notes app.

The project is {{ORIGIN}}. Its notes are in more than one folder: one folder for each machine and for each
place where the repository is on that machine.

Target folder: {{TO}}

The target folder holds the notes of {{ROOT}} on the machine "{{MACHINE}}". Collect the notes of the other
folders into it.

Other folders and their notes:

{{SOURCES}}

Rules:

- Write each note of the other folders into the target folder. Keep its text and its file name.
- If two notes are about the same thing, merge them into one note: keep every decision, requirement and
  open question of both, remove only text that is the same in both. Keep the earliest date in the file name.
  When the notes do not agree, keep both versions, one after the other, and add the line
  `Open: the notes do not agree.` between them.
- After you write the text of a note into the target folder, write the old file with empty content. zendo
  deletes the empty notes when you finish.
- Edit only files in the target folder and the notes in the list. Do not change, create or delete any other
  file. Do not run commands.
- After the work, answer in at most 5 short lines: how many notes you moved, which notes you merged, and
  where the notes do not agree.
