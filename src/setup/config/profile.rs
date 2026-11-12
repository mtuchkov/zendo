//! Agent profiles: Claude and Codex, and how to tell which one a command is.
use super::*;

/// The kind of an agent. What zendo can do with it depends on the kind and on the ACP capabilities
/// that the agent reports; a feature that an agent cannot do turns off only its keys.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Claude,
    Codex,
    /// Any other ACP agent command.
    Other,
}

/// An agent that zendo knows: its ACP adapter, pinned, and its command-line tool.
pub struct Profile {
    pub kind: Kind,
    /// The name for `--agent` and `agent = ...` in the config.
    pub id: &'static str,
    pub name: &'static str,
    pub package: &'static str,
    pub version: &'static str,
    /// The adapter entry point in its package.
    pub entry: &'static str,
    /// The folder of the adapter in the data folder of zendo.
    pub dir: &'static str,
    /// The command-line tool of the agent, for the login check.
    pub cli: &'static str,
    /// Hand the thread to the command-line tool (`ctrl-o`): the arguments before the session ID.
    pub handoff: Option<&'static [&'static str]>,
}

pub const CLAUDE: Profile = Profile {
    kind: Kind::Claude,
    id: "claude",
    name: "Claude",
    package: "@agentclientprotocol/claude-agent-acp",
    version: "0.106.0",
    entry: "dist/index.js",
    dir: "agent",
    cli: "claude",
    handoff: Some(&["--resume"]),
};

pub const CODEX: Profile = Profile {
    kind: Kind::Codex,
    id: "codex",
    name: "Codex",
    package: "@agentclientprotocol/codex-acp",
    version: "2.6.0",
    entry: "dist/index.js",
    dir: "agent-codex",
    cli: "codex",
    // Not known: whether the ACP session ID is a Codex session ID. The hand-off stays off.
    handoff: None,
};

impl Profile {
    pub fn by_id(id: &str) -> Option<&'static Profile> {
        PROFILES.into_iter().find(|p| p.id == id.trim())
    }

    pub fn by_kind(kind: Kind) -> Option<&'static Profile> {
        PROFILES.into_iter().find(|p| p.kind == kind)
    }

    pub fn install_dir(&self) -> PathBuf {
        data_dir().join(self.dir)
    }

    pub fn entry_path(&self) -> PathBuf {
        self.install_dir().join("node_modules").join(self.package).join(self.entry)
    }

    pub fn installed_version(&self) -> Option<String> {
        let pkg = self.install_dir().join("node_modules").join(self.package).join("package.json");
        let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(pkg).ok()?).ok()?;
        v["version"].as_str().map(str::to_string)
    }

    pub fn installed(&self) -> bool {
        self.installed_version().as_deref() == Some(self.version) && self.entry_path().exists()
    }

    /// The command: the installed adapter, else npx.
    pub fn command(&self) -> (String, Source) {
        if self.entry_path().exists() {
            (format!("node {}", quote(&self.entry_path())), Source::Installed)
        } else {
            (format!("npx -y {}@{}", self.package, self.version), Source::Npx)
        }
    }
}

/// The kind of an agent command: by the name of its adapter package, or by a profile name.
pub fn kind_of(cmd: &str) -> Kind {
    if let Some(p) = Profile::by_id(cmd) {
        return p.kind;
    }
    let c = cmd.replace('\\', "/");
    if c.contains("claude-agent-acp") || c.contains("claude-code-acp") {
        Kind::Claude
    } else if c.contains("codex-acp") {
        Kind::Codex
    } else {
        Kind::Other
    }
}
