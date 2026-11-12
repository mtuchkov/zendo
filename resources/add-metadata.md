Write metadata for the markdown document below. Do not use tools and do not edit files.

Answer in exactly this form, and nothing else:

{{PARTS}}END

Rules:

- TITLE: one line, at most 8 words, in the language of the document, no `#`, no quotes. Name what the
  document is about, not its form ("Retention of audit logs", not "Notes"). When the first line already
  names the document but has no `#`, give that line as it is.
- FRONT MATTER: YAML lines without the `---` lines: `status: draft`, `date: {{TODAY}}`, and
  `level:` with one sentence that names the one level of detail the document keeps.

Document {{FILE}}:

{{TEXT}}
