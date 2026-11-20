//! The command line of zendo.
use super::doctor;
use anyhow::{Context, Result, bail};
use zendo::setup::config;

pub(crate) const HELP: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/resources/help.txt"));

/// The command line.
#[derive(Default)]
pub(crate) struct Args {
    pub sub: &'static str,
    pub notes: String,
    pub notes_set: bool,
    pub agent: Option<String>,
    pub live: bool,
    pub fix: bool,
    pub with_agent: bool,
    pub offline: bool,
    pub notes_repo: Option<String>,
    pub action: String,
    pub fonts: bool,
    /// `--profile NAME`: the profile of the agents (PROFILE-1).
    pub profile: Option<String>,
}

pub(crate) fn read_args() -> Result<Args> {
    let mut o = Args { notes: std::env::var("ZENDO_NOTES").unwrap_or_else(|_| "docs/notes".into()), notes_set: std::env::var("ZENDO_NOTES").is_ok(), ..Default::default() };
    let mut args = std::env::args().skip(1).peekable();
    if let Some(first) = args.peek() {
        o.sub = match first.as_str() {
            "setup" => "setup",
            "init" => "init",
            "doctor" => "doctor",
            "upgrade" => "upgrade",
            "keymap" => "keymap",
            "notes" => "notes",
            _ => "",
        };
        if !o.sub.is_empty() {
            args.next();
        }
    }
    let sub = o.sub;
    while let Some(a) = args.next() {
        match a.as_str() {
            "--live" if sub == "doctor" => o.live = true,
            "--fix" if sub == "doctor" => o.fix = true,
            "--agent" if sub == "doctor" => o.with_agent = true,
            "--force" if sub == "upgrade" => o.live = true,
            "--offline" if sub.is_empty() => o.offline = true,
            "-V" | "--version" | "version" => {
                println!("{}", doctor::version_line());
                std::process::exit(0);
            }
            "-h" | "--help" => {
                println!("{HELP}");
                std::process::exit(0);
            }
            "--notes" => {
                o.notes = args.next().context("--notes needs a folder")?;
                o.notes_set = true;
            }
            "fonts" if sub == "setup" && !o.fonts => o.fonts = true,
            "--notes-repo" if sub == "setup" => o.notes_repo = Some(args.next().context("--notes-repo needs a folder or a git URL")?),
            "move" | "consolidate" if sub == "notes" && o.action.is_empty() => o.action = a.clone(),
            "--profile" if sub != "upgrade" => o.profile = Some(args.next().context("--profile needs a name")?),
            "--agent-cmd" => o.agent = Some(args.next().context("--agent-cmd needs a command")?),
            "--agent" => bail!("`--agent CMD` is now `--agent-cmd CMD`\n\n{HELP}"),
            "--mock" => o.agent = Some(mock_agent()?),
            other => bail!("unknown argument: {other}\n\n{HELP}"),
        }
    }
    Ok(o)
}

/// The mock agent next to this binary.
fn mock_agent() -> Result<String> {
    let exe = std::env::current_exe()?;
    let mock = exe.with_file_name(format!("zendo-mock-agent{}", std::env::consts::EXE_SUFFIX));
    if !mock.exists() {
        bail!("mock agent not found next to zendo: {}", mock.display());
    }
    Ok(config::quote(&mock))
}
