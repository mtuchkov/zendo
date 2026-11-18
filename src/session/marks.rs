//! The OSC 133 marks of the shells of the terminals (TERM-34): zendo loads a short script into bash,
//! zsh, fish and the POSIX shells when it starts them, and does not change the files of the user.
use portable_pty::CommandBuilder;
use std::path::Path;

/// bash: `--rcfile`; the file reads `~/.bashrc`, then adds the marks.
const BASH: &str = r#"# zendo: the OSC 133 marks of the terminals of zendo (docs/zendo/terminal/terminal-output.md).
[ -f "$HOME/.bashrc" ] && . "$HOME/.bashrc"
if [ -z "$__zendo_marks" ]; then
  __zendo_marks=1
  __zendo_end() { local s=$?; printf '\033]133;D;%s\007' "$s"; return $s; }
  PROMPT_COMMAND="__zendo_end${PROMPT_COMMAND:+;$PROMPT_COMMAND}"
  PS1="\[\033]133;A\007\]${PS1}\[\033]133;B\007\]"
  if (( BASH_VERSINFO[0] * 100 + BASH_VERSINFO[1] >= 404 )); then
    PS0="${PS0}"$'\033]133;C\007'
  else
    # bash before 4.4 (macOS /bin/bash 3.2) has no PS0: the DEBUG trap marks the first command
    # after the prompt; the end of PROMPT_COMMAND arms it again.
    __zendo_pre() { [ -n "$__zendo_ready" ] || return 0; __zendo_ready=; printf '\033]133;C\007'; }
    trap '__zendo_pre' DEBUG
    PROMPT_COMMAND="${PROMPT_COMMAND};__zendo_ready=1"
  fi
fi
"#;

/// zsh: `ZDOTDIR` points here; the files read the files of the user, then add the marks.
const ZSHENV: &str = r#"# zendo: reads the .zshenv of the user (docs/zendo/terminal/terminal-output.md).
__zendo_zdotdir=$ZDOTDIR
ZDOTDIR=${ZENDO_ZDOTDIR:-$HOME}
[[ -f $ZDOTDIR/.zshenv ]] && source $ZDOTDIR/.zshenv
ZENDO_ZDOTDIR=$ZDOTDIR
ZDOTDIR=$__zendo_zdotdir
"#;

const ZSHRC: &str = r#"# zendo: reads the .zshrc of the user, then adds the OSC 133 marks.
ZDOTDIR=$ZENDO_ZDOTDIR
[[ -f $ZDOTDIR/.zshrc ]] && source $ZDOTDIR/.zshrc
if [[ -z $__zendo_marks ]]; then
  __zendo_marks=1
  __zendo_precmd() { local s=$?; print -n "\e]133;D;$s\a\e]133;A\a"; }
  __zendo_preexec() { print -n "\e]133;C\a"; }
  # Prompt expansion does not read \e and \a: the mark needs the bytes. The last precmd adds it,
  # also to a prompt that a theme sets again in its own precmd.
  __zendo_b=$'\e]133;B\a'
  __zendo_prompt() { [[ $PS1 == *$__zendo_b* ]] || PS1="$PS1%{$__zendo_b%}"; }
  precmd_functions=(__zendo_precmd $precmd_functions __zendo_prompt)
  preexec_functions+=(__zendo_preexec)
fi
"#;

/// fish: `-C` sources this file after the configuration of the user.
const FISH: &str = r#"# zendo: the OSC 133 marks of the terminals of zendo.
if not set -q __zendo_marks
    set -g __zendo_marks 1
    function __zendo_prompt --on-event fish_prompt
        printf '\e]133;A\a'
    end
    function __zendo_preexec --on-event fish_preexec
        printf '\e]133;C\a'
    end
    function __zendo_postexec --on-event fish_postexec
        printf '\e]133;D;%s\a' $status
    end
end
"#;

/// The POSIX shells (sh, dash, ash, ksh, mksh): `ENV` points here; the file reads the `ENV` file
/// of the user, then puts the marks into `PS1`. These shells have no hook before a command, so they
/// send no C mark: the prompt sends D with the exit code (`$?` in `PS1`), then A and B.
const SH: &str = r#"# zendo: the OSC 133 marks of the terminals of zendo (docs/zendo/terminal/terminal-output.md).
if [ -n "$ZENDO_ENV" ]; then ENV=$ZENDO_ENV; else unset ENV; fi
unset ZENDO_ENV
[ -n "$ENV" ] && [ -f "$ENV" ] && . "$ENV"
if [ -z "$__zendo_marks" ]; then
  __zendo_marks=1
  __zendo_d=$(printf '\033]133;D;')
  __zendo_a=$(printf '\007\033]133;A\007')
  __zendo_b=$(printf '\033]133;B\007')
  if [ -n "$BASH_VERSION" ]; then
    PS1="\[$__zendo_d\$?$__zendo_a\]${PS1:-\$ }\[$__zendo_b\]"
  else
    PS1="$__zendo_d\$?$__zendo_a${PS1:-\$ }$__zendo_b"
  fi
fi
"#;

/// Add the script of the marks to the start of `program`, with the files in `dir`. A shell that is
/// not bash, zsh, fish or a POSIX shell starts as it is.
pub fn prepare(b: &mut CommandBuilder, program: &str, dir: &Path) -> std::io::Result<()> {
    let name = Path::new(program).file_name().and_then(|n| n.to_str()).unwrap_or("").trim_start_matches('-');
    match name {
        "bash" => {
            std::fs::create_dir_all(dir)?;
            std::fs::write(dir.join("bashrc"), BASH)?;
            b.arg("--rcfile");
            b.arg(dir.join("bashrc"));
        }
        "zsh" => {
            let z = dir.join("zsh");
            std::fs::create_dir_all(&z)?;
            std::fs::write(z.join(".zshenv"), ZSHENV)?;
            std::fs::write(z.join(".zshrc"), ZSHRC)?;
            if let Ok(user) = std::env::var("ZDOTDIR") {
                b.env("ZENDO_ZDOTDIR", user);
            }
            b.env("ZDOTDIR", &z);
        }
        "fish" => {
            std::fs::create_dir_all(dir)?;
            std::fs::write(dir.join("marks.fish"), FISH)?;
            b.arg("-C");
            b.arg(format!("source '{}'", dir.join("marks.fish").display()));
        }
        "sh" | "dash" | "ash" | "ksh" | "mksh" => {
            std::fs::create_dir_all(dir)?;
            std::fs::write(dir.join("marks.sh"), SH)?;
            if let Ok(user) = std::env::var("ENV") {
                b.env("ZENDO_ENV", user);
            }
            b.env("ENV", dir.join("marks.sh"));
        }
        _ => {}
    }
    Ok(())
}
