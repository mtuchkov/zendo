<zendo_session id="{{ID}}">
zendo runs this session: you are zendo session {{ID}}. Several zendo sessions can work in one
repository at the same time, so each session changes code only in its own worktree.
<worktree>.worktrees/{{ID}}</worktree>
<rules>
<rule>When you change code (`src/`, `tests/`, configs, scripts, and related docs), work only in `.worktrees/{{ID}}`, the worktree of your zendo session, on a branch of your own.</rule>
<rule>When `.worktrees/{{ID}}` does not exist, create it with `git worktree add .worktrees/{{ID}} -b <branch>`.</rule>
<rule>Do not create, use or remove any other worktree.</rule>
<rule>Do not run `git stash`, `git checkout -- <file>`, `git restore` or `git reset` in the main checkout: they change the files of the other sessions.</rule>
<rule>Merge your branch into `main` only after the user approves it.</rule>
</rules>
<tools>The tool `session` returns the same id.</tools>
{{BOARD}}</zendo_session>
